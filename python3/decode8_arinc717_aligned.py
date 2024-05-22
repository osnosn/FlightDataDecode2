#!/usr/bin/python3
# -*- coding: utf-8 -*-

"""
读取 wgl 中 raw.dat 。
解码一个参数。
仅支持 ARINC 573/717 Aligned 格式
"""
import json

class ARINC717():
    '''
    从 ARINC 573/717 ALIGNED 格式文件中，获取参数
    '''
    def __init__(self,fname):
        '''
        用来保存配置参数的实例变量
        '''
        self.prm=None
        self.raw=None
        self.rawlen=0
        self.qar_filename=''
        if len(fname)>0:
            self.qar_file(fname)
        if self.prm is None:
            with open('prm_conf320.json','r') as fp:
                self.prm=json.load(fp)   #320.PRM 的配置

    def qar_file(self,qar_filename):
        #----------读取raw.dat文件-----------
        if self.raw is None or self.qar_filename != qar_filename:
            with open(FNAME,'rb') as fp:
                self.raw=fp.read()
            self.rawlen=len(self.raw)
            self.qar_filename=qar_filename

    def get_param(self, prm_name, filename_write):
        #  (frame_pos, param_set, word_per_sec ):

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
            return
        prm_words = prm_param['words']
        prm_superframe = prm_param['superframe']
        if len(prm_param['res'])>0:
            [_, _, res_A, res_B, res_C] = prm_param['res'][0]
        else:
            [_, _, res_A, res_B, res_C] = [0, 0, 0.0, 1.0, 0.0]

        #每次都要取值的参数配置
        if "SuperFrameCounter" in self.prm['param']:
            prm_superFrameCnt_prm = self.prm['param']["SuperFrameCounter"]
        else:
            print("参数没找到:\"{}\"".format( "SuperFrameCounter"))
            return
        prm_superFrameCnt = prm_superFrameCnt_prm['words'][0]
        if "UTC_HOUR" in self.prm['param']:
            frame_hour_prm = self.prm['param']["UTC_HOUR"]
        else:
            print("参数没找到:\"{}\"".format("UTC_HOUR"))
            return
        frame_hour = frame_hour_prm['words'][0]
        if "UTC_MIN" in self.prm['param']:
            frame_min_prm = self.prm['param']["UTC_MIN"]
        else:
            print("参数没找到:\"{}\"".format("UTC_MIN"))
            return
        frame_min = frame_min_prm['words'][0]
        if "UTC_SEC" in self.prm['param']:
            frame_sec_prm = self.prm['param']["UTC_SEC"]
        else:
            print("参数没找到:\"{}\"".format("UTC_SEC"))
            return
        frame_sec = frame_sec_prm['words'][0]

        # 参数的 每秒记录个数
        # 这个值，算的很粗糙，可能会不正确 !!!!!
        #param_rate: f32
        if prm_words[0][0] == 0 :
            param_rate = len(prm_words)
        else :
            param_rate = 1
        #param_rate = 16.0

        fp_write=None
        if filename_write is not None:
            fp_write=open(filename_write,'w')
            fp_write.write("frametime,value,utc_time\r\n")

        #变量初始化
        self.subframe_cnt = 0 #subframe计数，
        self.subframe_idx = 1 #subframe索引, 1-4
        self.supcount_idx = 0 #超级帧索引, 0-15
        self.word_cnt = 0 #word计数，16bit字计数, (这个计数没什么用)
        self.byte_cnt = 0 #byte计数，字节计数。根据单/双数,也能确定word拼接时的位置。
        #value; #解码后的工程值
        #frame_time #frame时间轴
        frame_time_string='' #frame时间
        while True :
            if self.find_SYNC() == False:
                break
            val = self.get_dword_raw(prm_superFrameCnt_prm, prm_superFrameCnt)
            if val is not None :
                self.supcount_idx = val #超级帧索引
                if self.word_cnt < 128000 :
                    print("  --->超级帧索引:{}".format( self.supcount_idx))
            #超级帧判断
            if prm_superframe <= 0 or (prm_superframe == self.supcount_idx + 1) :
                #取UTC时间，H:M:S
                frame_time_string = ""
                val=self.get_dword_raw(frame_hour_prm, frame_hour)
                if val is not None:
                    frame_time_string += "{:02}:".format( val)
                val=self.get_dword_raw(frame_min_prm, frame_min)
                if val is not None:
                    frame_time_string += "{:02}:".format( val)
                val=self.get_dword_raw(frame_sec_prm, frame_sec)
                if val is not None:
                    frame_time_string += "{:02}".format( val)

                rate_cnt = 0.0
                #dword_raw: i32
                #按记录组循环. 单个记录组为一个完整的记录
                for prm_set in prm_words :
                    val=self.get_dword_raw(prm_param, prm_set)
                    if val is None:
                        continue
                    else:
                        dword_raw = val
                    value = float(dword_raw) * res_B + res_A #通过系数，转换为工程值
                    frame_time = float(self.subframe_cnt) + (rate_cnt / param_rate)
                    if self.word_cnt < 128000 :
                        print( "subframe:{}, frametime:{:.5f}, val:{}, UTC:{}".format( self.subframe_idx, frame_time, value, frame_time_string))

                    if fp_write is not None:
                        # 以csv格式写入文件
                        fp_write.write("{:.5f},{},{}\r\n".format( frame_time, value, frame_time_string))

                    #一个subframe只有一个记录，输出一次即可
                    frame_time_string = "" #输出一次后，就清除,
                    rate_cnt += 1.0

            self.byte_cnt += self.word_per_sec * 2
            self.word_cnt += self.word_per_sec - 1
            self.subframe_cnt += 1

        print("")
        print(" The length of data is {}.".format( self.rawlen))
        if fp_write is not None:
            fp_write.close()
            print(" Parameter \"{}\", write to CSV file: \"{}\".".format(prm_name, filename_write))
        else:
            print(" Parameter \"{}\", 用参数 -w myfile.csv 把结果写入文件。".format(prm_name))
        print("")

    def get_dword_raw(self, param_prm, prm_set):
        #(param_prm, prm_set, byte_cnt, suframe_idx)
        '''
            author: osnosn@126.com  
        '''
        dword_raw = 0
        ttl_bit = 0 #总bit计数
        #为了倒序循环,计算最后一组配置的值
        ii = (len(prm_set) // 5 - 1) * 5 #整数 乘除.
        while True:
            #倒序循环
            #配置中 是否 指定了 subframe
            if prm_set[ii] > 0 and prm_set[ii] != self.subframe_idx :
                return None
            if prm_set[ii + 4] != 0 :
                #targetBit !=0 不知道如何拼接，暂时忽略这个配置。只给出提示信息。
                print("--> INFO.targetBit !=0, 取值结果可能不正确")
            bits_cnt = prm_set[ii + 3] - prm_set[ii + 2] + 1
            ttl_bit += bits_cnt #总bit位数
            bits_mask = (1 << bits_cnt) - 1
            dword_raw <<= bits_cnt
            #dword_raw |= (((self.raw[self.byte_cnt + (prm_set[ii + 1] - 1) * 2 + 1]) << 8 | self.raw[self.byte_cnt + (prm_set[ii + 1] - 1) * 2]) >> (prm_set[ii + 2] - 1)) & bits_mask
            dword_raw |= (( self.getWord(self.byte_cnt + (prm_set[ii + 1] - 1) * 2) ) >> (prm_set[ii + 2] - 1)) & bits_mask
            if ii > 0 :
                ii -= 5 #step
            else :
                break

        #如果有符号位，并且，最高位为1 . 默认最高bit为符号位.
        #if param_prm.signed == true && dword_raw & (1 << (ttl_bit - 1)) > 0 {
        if param_prm['signRecType'] == True and dword_raw & (1 << (ttl_bit - 1)) > 0 :
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
        return dword_raw

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
                        print("->INFO.找到sync字.0x{:X} wordCnt:0x{:X}, len:0x{:X}".sormat(word16, self.word_cnt, diff_word_cnt))
                    else:
                        if self.word_cnt <128000:
                            #超过12800就不打印了
                            print("->找到sync字.0x{:X} wordCnt:0x{:X}".format(word16, self.word_cnt))
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

    def close(self):
        '清除,保留的所有配置和数据'
        self.prm=None
        self.raw=None
        self.rawlen=0
        self.qar_filename=''

def paramlist():
    myQAR=ARINC717('')
    print(' 配置中的参数名:')
    for vv in myQAR.prm['param']:
        print('    ',vv)
    print()
def main():
    global PARAM,FNAME,WFNAME
    myQAR=ARINC717('')
    myQAR.qar_file(FNAME)
    if PARAM is None:
        print("Use -p , 比如 -p VRTG")
    else:
        myQAR.get_param(PARAM,WFNAME)

import os,sys,getopt
def usage():
    print(u'Usage:')
    print(u'   命令行工具。')
    print(u' 读取 wgl中 raw320.dat,根据参数编码规则,解码一个参数。')

    print(sys.argv[0]+' [-h|--help]')
    print('   * (必要参数)')
    print('   -h, --help                 print usage.')
    print(' * -f, --file raw320.dat      "raw.dat" filename')
    print(' * -p, --param ALT_STD        show "ALT_STD" param.')
    print('   -l, --paramlist            list all param name.')
    print('   -w xxx.csv            参数写入文件"xxx.csv"')
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
        opts, args = getopt.gnu_getopt(sys.argv[1:],'hlw:df:p:',['help','file=','paramlist','param=',])
    except getopt.GetoptError as e:
        print(e)
        usage()
        exit(2)
    FNAME=None
    WFNAME=None
    DUMPDATA=False
    PARAMLIST=False
    PARAM=None
    for op,value in opts:
        if op in ('-h','--help'):
            usage()
            exit()
        elif op in('-f','--file'):
            FNAME=value
        elif op in('-w',):
            WFNAME=value
        elif op in('-d',):
            DUMPDATA=True
        elif op in('-l','--paramlist',):
            PARAMLIST=True
        elif op in('-p','--param',):
            PARAM=value
    if len(args)>0:  #命令行剩余参数
        FNAME=args[0]  #只取第一个
    if PARAMLIST:
        paramlist()
        exit()
    if FNAME is None:
        usage()
        exit()
    if os.path.isfile(FNAME)==False:
        print(FNAME,'Not a file')
        exit()

    main()

