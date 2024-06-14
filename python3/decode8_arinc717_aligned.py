#!/usr/bin/python3
# -*- coding: utf-8 -*-

"""
读取 wgl 中 raw.dat 。
解码一个参数。
仅支持 ARINC 573/717 Aligned 格式

  我的测试 Intel CPU i9,x64,主频3.3GHz, BogoMIPS:6600。
  原始文件 raw.dat 21MB。有参数 1080 个, 航段170分钟。
      解所有参数，写入单文件,bzip2压缩, 1.4MB，耗时1m45s.
  原始文件 raw.dat 15MB。有参数 2350 个, 航段121分钟。
      解所有参数，写入单文件,bzip2压缩, 1.5MB，耗时0m57s.
  rust版的decode8, 耗时6s到7s.
"""
import json
import struct
import lzma
import bz2
import gzip

class ARINC717():
    '''
    从 ARINC 573/717 ALIGNED 格式文件中，获取参数
    '''
    def __init__(self,fname='',jsonconf=''):
        '''
        用来保存配置参数的实例变量
        '''
        self.prm=None
        self.raw=None
        self.rawlen=0
        self.qar_filename=''
        if len(fname)>0:
            self.qar_file(fname)
        if self.prm is None and len(jsonconf)>2:
            with open(jsonconf,'r') as fp:
                self.prm=json.load(fp)   #320.PRM 的配置

    def jsonconf(self,jsonconf):
        if self.prm is None and len(jsonconf)>2:
            with open(jsonconf,'r') as fp:
                self.prm=json.load(fp)   #320.PRM 的配置
    def qar_file(self,qar_filename):
        #----------读取raw.dat文件-----------
        if self.raw is None or self.qar_filename != qar_filename:
            with open(FNAME,'rb') as fp:
                self.raw=fp.read()
            self.rawlen=len(self.raw)
            self.qar_filename=qar_filename

    def get_param(self, prm_name):
        #  (frame_pos, param_set, word_per_sec ):
        pm_list=[]   #用于返回,解码后的数据

        # 参数的配置,
        #prm_words #取值位置的配置
        #prm_superframe #超级帧
        #res_A #系数A
        #res_B #系数B
        #signed #是否带符号位 false=N, true=Y,
        self.word_per_sec = self.prm['WordPerSec']
        if prm_name in self.prm['param']:
            prm_param = self.prm['param'][prm_name]
        else:
            print("参数没找到:\"{}\"".format( prm_name))
            return pm_list
        prm_words = prm_param['words']
        #prm_superframe = prm_param['superframe']
        prm_superframe = prm_words[0][0]
        if len(prm_param['res'])>0:
            [_, _, res_A, res_B, res_C] = prm_param['res'][0]
        else:
            [_, _, res_A, res_B, res_C] = [0, 0, 0.0, 1.0, 0.0]

        #每次都要取值的参数配置
        if "SuperFrameCounter" in self.prm['param']:
            prm_superFrameCnt_prm = self.prm['param']["SuperFrameCounter"]
        else:
            print("参数没找到:\"{}\"".format( "SuperFrameCounter"))
            return pm_list
        prm_superFrameCnt = prm_superFrameCnt_prm['words'][0]

        # 参数的 每秒记录个数
        # 这个值，算的很粗糙，可能会不正确 !!!!!
        #param_rate: f32
        if prm_words[0][1] == 0 :
            param_rate = len(prm_words)
        else :
            param_rate = 1
        #param_rate = 16.0

        #变量初始化
        self.subframe_cnt = 0 #subframe计数，
        self.subframe_idx = 1 #subframe索引, 1-4
        self.supcount_idx = 0 #超级帧索引, 0-15
        self.word_cnt = 0 #word计数，16bit字计数, (这个计数没什么用)
        self.byte_cnt = 0 #byte计数，字节计数。根据单/双数,也能确定word拼接时的位置。
        dword_error = 0
        #value; #解码后的工程值
        #frame_time #frame时间轴
        while True :
            if self.find_SYNC() == False:
                break
            val, dword_err = self.get_dword_raw(prm_superFrameCnt_prm, prm_superFrameCnt)
            #dword_error |= dword_err  #忽略这次的错误, 并且这个不会出错
            if val is not None :
                self.supcount_idx = val #超级帧索引
                #if self.word_cnt < 12800 :
                #    print("  --->超级帧索引:{}".format( self.supcount_idx))
            #超级帧判断
            if prm_superframe <= 0 or (prm_superframe == self.supcount_idx + 1) :
                rate_cnt = 0.0
                #dword_raw: i32
                #按记录组循环. 单个记录组为一个完整的记录
                for prm_set in prm_words :
                    val, dword_error=self.get_dword_raw(prm_param, prm_set)
                    dword_error |= dword_err
                    if val is None:
                        continue
                    else:
                        dword_raw = val
                    value = float(dword_raw) * res_B + res_A #通过系数，转换为工程值
                    frame_time = float(self.subframe_cnt) + (rate_cnt / param_rate)
                    pm_list.append({'t':round(frame_time,10),'v':round(value,10)})

                    #if self.word_cnt < 12800 :
                    #    print( "subframe:{}, frametime:{:.5f}, val:{}".format( self.subframe_idx, frame_time, value))

                    #一个subframe只有一个记录，输出一次即可
                    rate_cnt += 1.0

            self.byte_cnt += self.word_per_sec * 2
            self.word_cnt += self.word_per_sec - 1
            self.subframe_cnt += 1
        print("--> INFO.{},targetBit !=0, 取值结果可能不正确".format(prm_name))
        return pm_list

    def get_dword_raw(self, param_prm, prm_set):
        #(param_prm, prm_set, byte_cnt, suframe_idx)
        '''
            author: osnosn@126.com  
        '''
        dword_error=0 #错误信息
        dword_raw = 0
        ttl_bit = 0 #总bit计数
        #为了倒序循环,计算最后一组配置的值
        ii = (len(prm_set) // 6 - 1) * 6 #整数 乘除.
        while True:
            #倒序循环
            #配置中 是否 指定了 subframe
            if prm_set[ii+1] > 0 and prm_set[ii+1] != self.subframe_idx :
                return None,dword_error
            if prm_set[ii + 5] != 0 :
                #targetBit !=0 不知道如何拼接，暂时忽略这个配置。只给出提示信息。
                #print("--> INFO.targetBit !=0, 取值结果可能不正确")
                dword_error=0x1
            bits_cnt = prm_set[ii + 4] - prm_set[ii + 3] + 1
            ttl_bit += bits_cnt #总bit位数
            bits_mask = (1 << bits_cnt) - 1
            dword_raw <<= bits_cnt
            #dword_raw |= (((self.raw[self.byte_cnt + (prm_set[ii + 2] - 1) * 2 + 1]) << 8 | self.raw[self.byte_cnt + (prm_set[ii + 2] - 1) * 2]) >> (prm_set[ii + 3] - 1)) & bits_mask
            dword_raw |= (( self.getWord(self.byte_cnt + (prm_set[ii + 2] - 1) * 2) ) >> (prm_set[ii + 3] - 1)) & bits_mask
            if ii > 0 :
                ii -= 6 #step
            else :
                break

        #如果有符号位，并且，最高位为1 . 默认最高bit为符号位.
        #if param_prm['signRecType'] == True and dword_raw & (1 << (ttl_bit - 1)) > 0 :
        if param_prm['signed']      == True and dword_raw & (1 << (ttl_bit - 1)) > 0 :
            #计算补码
            dword_raw -= 1 << ttl_bit
            #println!("--> INFO.signed=true, 计算补码")
        # 处理BCD, 即,十进制数值
        if 'BCD' == param_prm['RecFormat'] :
            bcd_dword = 0
            ii = 0
            #倒序
            for bcd_bits in reversed(param_prm['ConvConfig']) :
                bits_mask = (1 << bcd_bits) - 1
                bcd_dword += (bits_mask & dword_raw) * (10 ** ii)
                dword_raw >>= bcd_bits
                ii += 1
            dword_raw = bcd_dword
        return dword_raw,dword_error

    def getWord(self, pos, word_len=1):
        '''
        word_len=1,取2字节; word_len=2,取4字节;
        读取两个字节，取12bit为一个word。低位在前。littleEndian,low-byte first.
        支持取 12bits,24bits,36bits,48bits,60bits
           author: osnosn@126.com
        '''
        buf=self.raw
        #print(type(buf), type(buf[pos]), type(buf[pos+1])) #bytes, int, int

        #读数据的时候,开始位置加上subframe和word的偏移，可能会超限
        if word_len==1:
            if pos+1 >= self.rawlen:
                return 0  #超限返回0
            else:
                return ((buf[pos +1] << 8 ) | buf[pos] ) & 0xFFF

        #word_len>1 //只有获取大于1个word 的同步字,才有用
        word=0
        for ii in range(0,word_len):
            if pos+ii*2+1 >= self.rawlen:
                high = 0
            else:
                high = ((buf[pos+ii*2+1] << 8 ) | buf[pos +ii*2] ) & 0xFFF
            word |= high << (12 * ii)
        return word

    def find_SYNC(self):
        #( byte_cnt, word_cnt, word_per_sec, subframe_idx, rawlen)
        '''
        判断 frame_pos 位置，是否满足同步字特征。如果不满足, 继续寻找下一个起始位置
        '''
        pre_word_cnt=self.word_cnt #保存上一个位置
        if self.byte_cnt >0:
            # 非文件头，需要加1
            pre_word_cnt +=1
        while True:
            if self.byte_cnt >= self.rawlen -2:
                print("文件结束")
                return False
            if self.byte_cnt >0 and self.byte_cnt & 0x1 ==0:
                #偶数
                self.word_cnt +=1
            #----似乎只判断连续两个同步字位置正确, 就够了-----
            word16=self.getWord(self.byte_cnt)
            if word16 == 0x247 or word16 == 0x5B8 or word16 == 0xA47 or word16 == 0xDB8 :
                if word16==0x247: self.subframe_idx=1
                elif word16==0x5B8: self.subframe_idx=2
                elif word16==0xA47: self.subframe_idx=3
                elif word16==0xDB8: self.subframe_idx=4
                if self.byte_cnt + self.word_per_sec*2 >= self.rawlen-2 :
                    print("->找到last sync字. wordCnt:0x{:X}---word:0x{:X}".format(self.word_cnt,word16))
                    return True

                # word_per_sec 之后，也是同步字
                word16_next=self.getWord(self.byte_cnt+self.word_per_sec*2)
                diff_word_cnt = self.word_cnt - pre_word_cnt #位置差值
                if word16_next == 0x247 or word16_next == 0x5B8 or word16_next == 0xA47 or word16_next == 0xDB8 :
                    if diff_word_cnt >0:
                        print("->INFO.找到sync字.0x{:X} wordCnt:0x{:X}, len:0x{:X}".format(word16, self.word_cnt, diff_word_cnt))
                    else:
                        pass
                        #if self.word_cnt <12800:
                        #    #超过12800就不打印了
                        #    print("->找到sync字.0x{:X} wordCnt:0x{:X}".format(word16, self.word_cnt))
                    if (self.subframe_idx ==1 and word16_next != 0x5B8) \
                        or (self.subframe_idx == 2 and word16_next != 0xA47) \
                        or (self.subframe_idx == 3 and word16_next != 0xDB8) \
                        or (self.subframe_idx == 4 and word16_next != 0x247):
                        print("--->INFO.当前sync字.0x{:0X} wordCnt:0x{:X},NEXT.0x{:X},sync错误".format( word16, self.word_cnt, word16_next))
                    return True
                else:
                    print( "--->INFO.找到sync字.0x{:X} wordCnt:0x{:X}, 但next不是sync字, len:0x{:X}".format( word16, self.word_cnt, diff_word_cnt))
            self.byte_cnt +=1
        pass

    def all_param(self,WFNAME):
        #准备Header
        datafile_header=bytearray(b"QAR_Decoded_DATA_V1.0\0")
        point=len(datafile_header)
        datafile_header.extend(b"\0\0\0\0")
        meta={
                "MetaData": {
                    "DataVersion":8888,
                    "ParamConfigFile":"",
                    "Tail":".B-8888",
                    "Type":"A320",
                    "FlightNum":"CXX8888",
                    "DepICAO":"ZGGL",
                    "DepRunway":"01",
                    "ArrICAO":"ZGSZ",
                    "ArrRunway":"15L",
                    "DepDateTime":"20240102T160555Z",
                    "ArrDateTime":"20240102T170922Z",
                    "TakeOffDateTime":"20240102T162359Z",
                    "LandingDateTime":"20240102T170101Z",
                    "AirborneDuration":161,
                    "FlightDuration":173,
                    "DecodeDateTime":"20240401T122359Z",
                    "FileName":"wgl.raw.dat",
                    },
                "other":123,
                "info":"This is a test.",
                }
        datafile_header.extend(json.dumps(meta,separators=(',',':'),ensure_ascii=False).encode())
        datafile_header.append(0)  #补'\0'
        with open('../data/Custom_DataFile_Format_Description.txt','rb') as fp:
            datafile_header.extend(fp.read())
        datafile_header.append(0)  #补'\0'
        #填入 Header size
        datafile_header[point:point+4]=struct.pack('<L',len(datafile_header)) #long,4byte,Little-Endion

        #准备Parameter_Table
        parameter_table=bytearray(b"\0\0\0\0") #Parameter_Table size

        parameter_data=bytearray() #存放 压缩/未压缩 的解码数据
        #-----------解码配置中的所有参数名称--------------
        total_pm=0
        for vv in self.prm['param']:
            one_param_table=bytearray(b"\0\0")  #Parameter01 size
            one_param_table.extend(b"\0\0\0\0\0\0\0\0") #Parameter01_DATA 指针
            one_param_table.extend(b"\0\0\0\0") #Parameter01_DATA size

            print(vv,flush=True)
            total_pm +=1
            #if total_pm==1: continue  #第一个不是参数
            pm_list=self.get_param(vv)
            pm_par=self.getPAR(vv)
            other_info=json.dumps(pm_par,separators=(',',':'),ensure_ascii=False).encode()+b'\0'
            pm_name="{}.{}".format(total_pm,vv)  #防止参数重名,加上序号
            data_len, data_type, value_size, compress_type=self.write_datafile(parameter_data,pm_name,pm_list)
            data_rate=pm_list[1]['t']-pm_list[0]['t']
            #print(pm_list[1]['t'],pm_list[0]['t'],data_rate,flush=True)
            if data_rate<=1:
                data_rate= 1/data_rate
            else:
                data_rate *= -1
            #print(data_rate,flush=True)

            one_param_table.extend(struct.pack('<h',value_size))     #value size
            one_param_table.extend(struct.pack('<h',int(data_rate))) #rate
            one_param_table.extend(b"\0\0\0\0")           #start FrameID
            one_param_table.extend(bytes(vv,'utf8')+b'\0')  #参数名称
            one_param_table.extend(compress_type)           #压缩算法
            one_param_table.extend(data_type)          #数据类型
            one_param_table.extend(other_info)         #其他信息
            #填入 Parameter01 size
            one_param_table[0:2]=struct.pack('<H',len(one_param_table)) #short,2byte,Little-Endion
            #填入Parameter01_DATA size
            one_param_table[10:14]=struct.pack('<L',data_len) #long,4byte,Little-Endion

            #加入Parameter_Table
            parameter_table.extend(one_param_table)
        print()
        print(" Count of parameters:{}".format(total_pm))

        #填入 Parameter_Table 的 size
        parameter_table[0:4]=struct.pack('<L',len(parameter_table)) #long,4byte,Little-Endion
        point=4
        data_point=len(datafile_header)+len(parameter_table)
        while point < len(parameter_table)-8:
            #填入 Parameter01_DATA 的起始位置
            parameter_table[point+2:point+10] = struct.pack('<Q',data_point)   #long long,8byte,Little-Endion
            data_point += struct.unpack('<L',parameter_table[point+10:point+14])[0]  #long,4byte
            one_param_size = struct.unpack('<H',parameter_table[point:point+2])[0]   #short,2byte
            point += one_param_size

        if WFNAME is not None and len(WFNAME)>0:
            with open(WFNAME,'wb') as mydatafile:
                mydatafile.write(datafile_header)
                mydatafile.write(parameter_table)
                mydatafile.write(parameter_data)

    def write_datafile(self,parameter_data,pm_name, pm_list):
        #-----------参数写入data bytearray--------------------
        data_len=0
        data_type=b'txt\0'
        value_size=0
        if parameter_data is None:
            #不写文件,就打印到终端
            if len(pm_list)>0:
                print([vv['v'] for vv in pm_list[0:10] ])
        else:
            if isinstance(pm_list[0]['v'], int) :
                #pm_data=[struct.pack('<fl',vv['t'],vv['v']) for vv in pm_list]
                pm_data=[struct.pack('<l',vv['v']) for vv in pm_list]
                tmp_str=b"".join(pm_data)
                data_type=b'int\0'
                value_size=4
            elif isinstance(pm_list[0]['v'], float) :
                #pm_data=[struct.pack('<ff',vv['t'],vv['v']) for vv in pm_list]
                pm_data=[struct.pack('<f',vv['v']) for vv in pm_list]
                tmp_str=b"".join(pm_data)
                data_type=b'float\0'
                value_size=4
            else:
                ### 获取解码参数的 json 数据
                df_pm=pd.DataFrame(pm_list)
                #tmp_str=df_pm.to_csv(None,sep='\t',index=False)
                #tmp_str=df_pm.to_json(None,orient='split',index=False)
                #tmp_str=df_pm.to_json(None,orient='records')
                #tmp_str=df_pm.to_json(None,orient='index')
                tmp_str=df_pm.to_json(None,orient='values')
                #tmp_str=df_pm.to_json(None,orient='table',index=False)
    
                tmp_str=bytes(tmp_str,'utf8')
                data_type=b'json\0'
                value_size=0
    
            ### 解码数据的压缩
            ### lzma占用内存大,bzip2占用内存小,两者压缩率在此场景下差不多
            #tmp_b=lzma.compress(tmp_str,format=lzma.FORMAT_XZ)    #有完整性检查
            #compress_type=b'xz\0'
            #tmp_b=lzma.compress(tmp_str,format=lzma.FORMAT_ALONE)  #无完整性检查
            #compress_type=b'lzma\0'
            tmp_b=bz2.compress(tmp_str,compresslevel=9)
            compress_type=b'bzip2\0'
            #tmp_b=gzip.compress(tmp_str,compresslevel=9)
            #compress_type=b'gzip\0'
    
            data_len=len(tmp_b)
            parameter_data.extend(tmp_b)
            print('mem:',sysmem())
        return data_len, data_type, value_size, compress_type

    def getPAR(self,param):
        if param in self.prm['param']:
            pm_find = self.prm['param'][param]
        else:
            print("参数没找到:\"{}\"".format( param))
            return pm_list
        info={}
        info["RecFormat"]= pm_find["RecFormat"]
        info["Unit"]=pm_find["Unit"]
        if "range" in pm_find:
            info["Range"]=pm_find["range"]
        if "Options" in pm_find and len(pm_find["Options"])>0:
            info["Options"]= pm_find["Options"]
        return info
    
    def close(self):
        '清除,保留的所有配置和数据'
        self.prm=None
        self.raw=None
        self.rawlen=0
        self.qar_filename=''

def paramlist(JSONCONF):
    myQAR=ARINC717('')
    myQAR.jsonconf(JSONCONF)
    print(' 配置中的参数名:')
    for vv in myQAR.prm['param']:
        print('    ',vv)
    print()
def main():
    global ALLPARAM,PARAM,FNAME,WFNAME,JSONCONF
    myQAR=ARINC717('')
    myQAR.jsonconf(JSONCONF)
    myQAR.qar_file(FNAME)
    if ALLPARAM:
        myQAR.all_param(WFNAME)
        print(" The sizeof RAW data is {}.".format( myQAR.rawlen))
        if WFNAME is not None:
            print(" ALL Parameter write to DAT file: \"{}\".".format(WFNAME))
        else:
            print(" 用参数 -w myfile.dat 把所有参数的解码结果写入文件。")
    elif PARAM is None:
        print("Use -p , 比如 -p VRTG")
    else:
        pm_list=myQAR.get_param(PARAM)
        print("")
        print(" The sizeof RAW data is {}.".format( myQAR.rawlen))
        if WFNAME is not None:
            with open(WFNAME,'w') as fp_write:
                fp_write.write("frametime,value\r\n")
                for vv in pm_list:
                    # 以csv格式写入文件
                    fp_write.write("{:.5f},{}\r\n".format( vv['t'], vv['v']))
            print(" Parameter \"{}\", write to CSV file: \"{}\".".format(PARAM, WFNAME))
        else:
            print(" Parameter \"{}\", 用参数 -w myfile.csv 把结果写入文件。".format(PARAM))

def showsize(size):
    '''
    显示，为了 human readable
    '''
    if size<1024.0*2:
        return '%.0f B'%(size)
    size /=1024.0
    if size<1024.0*2:
        return '%.2f K'%(size)
    size /=1024.0
    if size<1024.0*2:
        return '%.2f M'%(size)
    size /=1024.0
    if size<1024.0*2:
        return '%.2f G'%(size)
import psutil         #非必需库
def sysmem():
    '''
    获取本python程序占用的内存大小
    '''
    size=psutil.Process(os.getpid()).memory_info().rss #实际使用的物理内存，包含共享内存
    #size=psutil.Process(os.getpid()).memory_full_info().uss #实际使用的物理内存，不包含共享内存
    return showsize(size)


import os,sys,getopt
def usage():
    print(u'Usage:')
    print(u'   命令行工具。')
    print(u' 读取 wgl中 raw.dat,根据参数编码规则 prm.json, 解码一个参数。')

    print(sys.argv[0]+' [-h|--help]')
    print('   * (必要参数)')
    print('   -h, --help                print usage.')
    print(' * -j, --jsonconf prm.json   读取 "prm.json" 解码配置文件')
    print(' * -f, --file raw320.dat     "raw.dat" filename')
    print(' * -p, --param ALT_STD       show "ALT_STD" param.')
    print('     -w out.csv                with "-p",参数写入文件"out.csv"')
    print('   -l, --paramlist           list all param name.')
    print('   -a, --allparam            解码  all parameters.')
    print('     -w out.dat                with "-a",参数写入自定义格式文件"out.dat"')
    print(u'\n               author: osnosn@126.com')
    print(u' 认为此项目对您有帮助，请发封邮件给我，让我高兴一下.')
    print(u' If you think this project is helpful to you, please send me an email to make me happy.')
    print()
    return
if __name__=='__main__':
    if(len(sys.argv)<2):
        usage()
        exit()
    try:
        opts, args = getopt.gnu_getopt(sys.argv[1:],'hlw:j:f:p:a',['help','file=','jsonconf=','paramlist','allparam','param=',])
    except getopt.GetoptError as e:
        print(e)
        usage()
        exit(2)
    FNAME=None
    WFNAME=None
    JSONCONF=''
    PARAMLIST=False
    PARAM=None
    ALLPARAM=None
    if len(args)>0:  #命令行剩余参数
        FNAME=args[0]  #只取第一个
    for op,value in opts:
        if op in ('-h','--help'):
            usage()
            exit()
        elif op in('-f','--file'):
            FNAME=value
        elif op in('-w',):
            WFNAME=value
        elif op in('-j','--jsonconf'):
            JSONCONF=value
        elif op in('-l','--paramlist',):
            PARAMLIST=True
        elif op in('-a','--allparam',):
            ALLPARAM=True
        elif op in('-p','--param',):
            PARAM=value
    if PARAMLIST:
        paramlist(JSONCONF)
        exit()
    if FNAME is None:
        usage()
        print(' =>ERROR,需要解码的raw原始文件.')
        exit()
    if len(JSONCONF)<3:
        usage()
        print(' =>ERROR,json解码配置文件,未指定.')
        exit()
    if os.path.isfile(FNAME)==False:
        print('"{}" Not a file'.format(FNAME))
        exit()
    if os.path.isfile(JSONCONF)==False:
        print('"{}" Not a file'.format(JSONCONF))
        exit()

    main()

