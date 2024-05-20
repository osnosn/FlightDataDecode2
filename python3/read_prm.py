#!/usr/bin/python3
# -*- coding: utf-8 -*-

"""
读解码库，参数配置文件 xxx.PRM
仅支持 ARINC 573 PCM 格式
   author: osnosn@126.com
"""
import os
import gzip
from io import StringIO
import config_vec as conf
import json

def main():
    global FNAME,DUMPDATA
    global TOCSV
    global PARAMLIST
    global PARAM
    prm_conf=read_parameter_file(FNAME)
    if prm_conf is None:
        return

    #print(prm_conf)
    #print(prm_conf.keys())

    if PARAMLIST:
        #----------显示所有参数名-------------
        #print(prm_conf['2'].iloc[:,0].tolist())
        #---regular parameter
        print('------------regular-----------------------------')
        ii=0
        for kk,vv in prm_conf['PRM'].items():
            if 'superframe' not in vv:
                print(vv['name'], end=',\t')
                if ii % 10 ==0:
                    print()
                ii+=1
        print()
        #---superframe parameter
        print('-------------superframe-------------------------')
        ii=0
        for kk,vv in prm_conf['PRM'].items():
            if 'superframe' in vv:
                print(vv['name'], end=',\t')
                if ii % 10 ==0:
                    print()
                ii+=1
        print()
        #----写CSV文件--------
        if len(TOCSV)>4:
            print('Write to CSV file:',TOCSV)
            if TOCSV.endswith('.gz'):
                fp=gzip.open(TOCSV,'wt',encoding='utf8')
            else:
                fp=open(TOCSV,'w',encoding='utf8')
            fp.write('regular:\n')
            ii=0
            for kk,vv in prm_conf['PRM'].items():
                if 'superframe' not in vv:
                    fp.write(str(ii)+'\t'+vv['name']+'\n')
                ii+=1
            fp.write('super frame:\n')
            ii=0
            for kk,vv in prm_conf['PRM'].items():
                if 'superframe' in vv:
                    fp.write(str(ii)+'\t'+vv['name']+'\n')
                ii+=1
            fp.close()
        return

    if PARAMJSON:
        #----------显示所有参数名-------------
        #---regular parameter, superframe parameter
        PrmConf=prm_to_dict(prm_conf)
        #----写CSV文件--------
        if len(TOCSV)>4:
            print('Write to CSV file:',TOCSV)
            if TOCSV.endswith('.gz'):
                fp=gzip.open(TOCSV,'wt',encoding='utf8')
            else:
                fp=open(TOCSV,'w',encoding='utf8')
            fp.write(json.dumps(PrmConf, ensure_ascii=False, separators=(',',':')))
            fp.close()
        else:
            print(json.dumps(PrmConf, ensure_ascii=False, indent=3))
        return

    if PARAM is not None and len(PARAM)>0:  #显示单个参数名
        #----------显示单个参数的配置内容-------------
        param=PARAM.upper()
        #---regular parameter
        #---superframe parameter
        idx=[]
        for kk,vv in prm_conf['PRM'].items():
            if vv['name'].upper() == param:
                idx.append(kk)
                print('编号:',kk)
                print('  name:',vv['name'])
                print('  长名称:',vv['namelong'])
                if 'rate' in vv:
                    print('  每秒记录个数:',vv['rate'])
                else:
                    print('  [无: 每秒记录个数]')
                if 'RecFormat' in vv:
                    print('  采样类型:',vv['type1'])
                    print('  记录格式:',vv['RecFormat'])
                    print('  ConvConfig:',vv['ConvConfig'])
                    print('  signed:',vv['sign'])
                    print('  SignRecType:',vv['SignRecType'])
                    print('  FlagType:',vv['FlagType'])
                    print('  计量单位:',vv['unit'])
                else:
                    print('  [无: 记录格式]')
                if 'min' in vv:
                    print('  数值范围:',vv['min'],',',vv['max'])
                else:
                    print('  [无: 数值范围]')
                if 'superframe' in vv:
                    print('  超级帧位置:',vv['superframe'])
                print('  --记录位置--')
                if len(vv['map'])>0:
                    print('    编号   subframe  word  lsb(低位)  msb(高位) target')
                for mapk,mapv in vv['map'].items():
                    for mapv2 in mapv:
                        print('     {:<4}   {:>4}    {:>5}  {:>3}        {:>3}       {:>3}'.format(mapk,mapv2['subframe'],mapv2['word'],mapv2['lsb'],mapv2['msb'],mapv2['target']))
                print('  --数值转换--')
                if len(vv['res'])>0:
                    print('    编号 MinValue MaxValue EUConvType resI   resolutionA  resolutionB  resolutionC')
                for mapk,mapv in vv['res'].items():
                    print('    {:>4} {:>8} {:>8} {:>8}    {:>3}  {:>12} {:>12} {:>12}'.format(mapk,mapv['MinValue'],mapv['MaxValue'],mapv['EUConvType'],mapv['resI'],mapv['resolutionA'],mapv['resolutionB'],mapv['resolutionC']))
                print('  --离散枚举值--')
                for mapk,mapv in vv['enum'].items():
                    print('    编号:',mapk,' ( 数值=>枚举值)')
                    for mapv2 in mapv:
                        print('      ',mapv2['val'],'=>',mapv2['text'])
                print('  --参数类型--')
                if 'superframe' in vv:
                    print('    super frame 参数')
                else:
                    print('    regular 参数')
        if len(idx)<1:
            print('Parameter %s not found in Regular parameter.'%param)
            print('Parameter %s not found in Superframe parameter.'%param)
        print()
        if DUMPDATA:
            tmp=getPRM(FNAME,PARAM)
            for kk,vv in tmp.items():
                if kk == '2':
                    print('{}: ['.format(kk))
                    for v2 in vv:
                        print('  {}'.format(v2))
                    print(']')
                else:
                    print('{}: {}'.format(kk,vv))

        return

    prm_len=len(prm_conf['PRM'])
    print('---- basic ------------- recorder num:',prm_len,'-----')
    print('   Word/Sec:',prm_conf['DAT'][2])
    if len(prm_conf['SUP'])>0:
        print('   SuperFrame Count:')
        print('      frame/cycle:',prm_conf['SUP'][2])
        print('      SubFrame:',prm_conf['SUP'][3])
        print('      Word:',prm_conf['SUP'][4])
        print('      LSB:',prm_conf['SUP'][5])
        print('      MSB:',prm_conf['SUP'][6])
    else:
        print('   No Superframe Count.')

    reguar_len=0
    super_len=0
    for vv in prm_conf['PRM'].values():
        if 'superframe' in vv:
            super_len +=1
        else:
            reguar_len+=1
    print('------------ reguar recorder num:',reguar_len,'-----')
    print('------------ super  recorder num:',super_len,'-----')
    print()

    if len(TOCSV)>4:
        print('==>ERR,  There has 4 tables. Can not save to 1 CSV.')

def print_fra(prm_conf, frakey ):
    if frakey not in prm_conf:
        print('ERR, %s not in list' % frakey)
        return
    fra_len=len(prm_conf[frakey])-1
    print('----',frakey,'------------- recorder num:',fra_len,'-----')
    if fra_len>6:
        show_len=6
    else:
        show_len=fra_len
    for ii in range(prm_conf[frakey+'_items']):
        print(ii,end=',')
        for jj in range(1,show_len+1):
            print('\t',prm_conf[frakey][jj][ii], end=',')
        print('\t',prm_conf[frakey][0][ii])

def prm_to_dict(prm_conf):
    PrmConf={
            "WordPerSec": int(prm_conf['DAT'][2]),
            "SuperFramePerCycle": int(prm_conf['SUP'][2]),
            "param": {
                "SuperFrameCounter": {
                    #words: [ subframe,word,lsb,msb,targetBit]
                    "words": [[
                        int(prm_conf['SUP'][3]) if prm_conf['SUP'][3]!='ALL' else 0,
                        int(prm_conf['SUP'][4]),
                        int(prm_conf['SUP'][5]),
                        int(prm_conf['SUP'][6]),
                        0]],
                    #res: 系数 [A,B]; 转换公式, A+B*X
                    "res": [0.0, 1.0],
                    "signed": False,      #true=1,有符号; false=0,无符号; 
                    "signRecType": False, #true=01,有符号; false=00,无符号; 以这个为准.
                    "superframe": 0,   #superframe:0 非超级帧参数
                    "RecFormat": "BNR",
                    "ConvConfig": [],
                    "Unit": "",
                    "LongName": "SUPER FRAME COUNTER"
                    }
                }
            }
    ii=0
    for vv in prm_conf['PRM'].values():
        PrmConf["param"][vv['name']]={
                    #words: [ subframe,word,lsb,msb,targetBit]
                    "words": [],
                    #res: 系数 A,B,C; 转换公式, A+B*X+C*X*X
                    #res: [MinValue, MaxValue, resolutionA, resolutionB, resolutionC]
                    "res": [],
                    "signed": True if vv['sign']=='Y' else False,
                    "signRecType": True if int(vv['SignRecType'])!=0 else False,
                    "superframe": int(vv['superframe']) if 'superframe' in vv else 0,
                    "RecFormat": vv['RecFormat'],
                    #ConvConfig: 类型为BCD/ISO，每一位"数字/字符"占用的bit数
                    "ConvConfig": [],
                    "Unit": vv['unit'],
                    "LongName": vv['namelong'],

                    "rate": int(vv['rate']),
                    "FlagType": int(vv['FlagType']) if vv['FlagType']!='' else 0,
                    "range": [],
                    #Options: 类型为DIS，枚举值
                    "Options": [],
                    }
        if len(vv['ConvConfig'])>0:
            for onechar in vv['ConvConfig']:
                if onechar=='s':
                    onechar='1'
                    #print(vv['name'],vv['ConvConfig'])
                PrmConf["param"][vv['name']]["ConvConfig"].append( int(onechar) )
        if 'min' in vv:
            PrmConf["param"][vv['name']]["range"].append( float(vv['min']) )
            PrmConf["param"][vv['name']]["range"].append( float(vv['max']) )
        for mapv in vv['map'].values():
            for mapv2 in mapv:
                if mapv2['subframe'].find(',')>0:
                    subframeA=mapv2['subframe'].split(',')
                else:
                    subframeA=[mapv2['subframe'], ]
                for subframe in subframeA:
                    PrmConf["param"][vv['name']]["words"].append([
                        int(subframe) if subframe!='ALL' else 0,
                        int(mapv2['word']),
                        int(mapv2['lsb']),
                        int(mapv2['msb']),
                        int(mapv2['target']) if mapv2['target']!='' else 0,
                        ])
        #print('    编号 MinValue MaxValue EUConvType resI   resolutionA  resolutionB  resolutionC')
        #if len(vv['res'])>1:
        #    print(vv['name'],vv['res'])
        for mapv in vv['res'].values():
            PrmConf["param"][vv['name']]["res"].append([
                int(mapv['MinValue']),
                int(mapv['MaxValue']),
                float(mapv['resolutionA']),
                float(mapv['resolutionB']),
                float(mapv['resolutionC']),
                ])
        for mapv in vv['enum'].values():
            for mapv2 in mapv:
                #PrmConf["param"][vv['name']]["Options"][int(mapv2['val'])]=mapv2['text']
                PrmConf["param"][vv['name']]["Options"].append([int(mapv2['val']), mapv2['text'] ])
        ii+=1
    #print(json.dumps(PrmConf, ensure_ascii=False, separators=(',',':')))
    #print(json.dumps(PrmConf, ensure_ascii=False, indent=3))
    return PrmConf

def read_parameter_file(dataver):
    '''
    prm_conf={
         'HDR': [x,x,,....],
         'DAT': [x,x,,....],
         'SUP': [x,x,,....],
         'PRM': {
            '012345': {  //param_index
              'name':'...',
              'namelong':'...',
              'map': {
                '01': {
                  'subframe': '...',
                  'word': '...',
                  'lsb': '...',
                  'msb': '...',
                  'target': '...',
                },
                ...
              },
              'superframe':'...',
              'res': {
                '01': {
                  'MinValue': '..',
                  'MaxValue': '..',
                  'EUConvType': '..',
                  'resI': '..',
                  'resolutionA': '..',
                  'resolutionB': '..',
                  'resolutionC': '..',
                },
                ...
              },
              'enum': {
                '01': {
                  'val': '..',
                  'text': '..',
                },
                ...
              },
              'type1':'...',
              'RecFormat':'...',
              'sign':'...',
              'SignRecType':'...',
              'FlagType':'...',
              'ConvConfig':'...',
              'unit':'...',
              'rate':'...',
              'min':'...',
              'max':'...',
            },
            ...
         }
    }
    '''
    if not isinstance(dataver,str):
        dataver=str(dataver)

    filename_prm=dataver+'.PRM'     #.PRM文件名
    prm_fname=os.path.join(conf.prm,dataver+'.PRM')  #.PRM文件名

    if os.path.isfile(prm_fname)==False:
        print('ERR,PRM_FileNotFound',prm_fname,flush=True)
        raise(Exception('ERR,PRM_FileNotFound,%s'%(prm_fname)))

    prm_conf={'HDR':[],'DAT':[],'SUP':[],'PRM':{}}
    with open(prm_fname,'r',encoding='iso-8859-1') as fp:
        # DAT1 HDR03 PA11 PA12 PA17 PA21 PA22 PA31 PA32 PA41 PA50 PA62 PA70 SUP1
        PmName='EMPTY'
        for line in fp.readlines():
            line_tr=line.strip('\r\n')
            if line_tr.startswith('HDR03'):  #信息头
                prm_conf['HDR']=split_line(line_tr)
            elif line_tr.startswith('DAT1'): #信息头
                prm_conf['DAT']=split_line(line_tr)
            elif line_tr.startswith('SUP1'): #信息头
                prm_conf['SUP']=split_line(line_tr)
            elif line_tr.startswith('PA11'):   #记录参数
                row=split_line(line_tr)
                PmName=row[1]   #编号
                if PmName not in prm_conf['PRM']:
                    prm_conf['PRM'][PmName]={}
                else:
                    print('ERR,param 编号重复,',PmName)
                prm_conf['PRM'][PmName]['name']=row[3]
                prm_conf['PRM'][PmName]['namelong']=row[5]
                prm_conf['PRM'][PmName]['map']={}
                prm_conf['PRM'][PmName]['res']={}
                prm_conf['PRM'][PmName]['enum']={}
            elif line_tr.startswith('PA12'):  #记录参数
                row=split_line(line_tr)
                prm_conf['PRM'][PmName]['type1']=row[2]  #采样类型
                prm_conf['PRM'][PmName]['RecFormat']=row[3]
                prm_conf['PRM'][PmName]['sign']=row[4]
                prm_conf['PRM'][PmName]['unit']=row[5]
            elif line_tr.startswith('PA17'):  #记录参数
                row=split_line(line_tr)
                prm_conf['PRM'][PmName]['name']=row[1]
            elif line_tr.startswith('PA21'):  #记录参数
                row=split_line(line_tr)
                prm_conf['PRM'][PmName]['rate']=row[1]
                prm_conf['PRM'][PmName]['ConvConfig']=row[2]
                prm_conf['PRM'][PmName]['SignRecType']=row[4]
                prm_conf['PRM'][PmName]['FlagType']=row[5]
                #print(row)
                #if row[5] !='00':
                #    print(prm_conf['PRM'][PmName]['name'],row[5])
                #    print('==>',row[5])
            elif line_tr.startswith('PA22'):  #记录参数
                row=split_line(line_tr)
                prm_conf['PRM'][PmName]['min']=row[1]
                prm_conf['PRM'][PmName]['max']=row[2]
                #print(row)
            elif line_tr.startswith('PA31'):  #记录参数
                row=split_line(line_tr)
                if row[1] not in prm_conf['PRM'][PmName]['map']:
                    prm_conf['PRM'][PmName]['map'][row[1]]=[]
                for ii in range(3,len(row)-4,5):
                    if len(row[ii])>0:
                        prm_conf['PRM'][PmName]['map'][row[1]].append({
                            'subframe':row[ii],
                            'word':row[ii+1],
                            'lsb':row[ii+2],
                            'msb':row[ii+3],
                            'target':row[ii+4],
                            })
                #print(row)
            elif line_tr.startswith('PA32'):  #记录参数
                row=split_line(line_tr)
                if row[1] not in prm_conf['PRM'][PmName]['map']:
                    prm_conf['PRM'][PmName]['map'][row[1]]=[]
                for ii in range(3,len(row)-4,5):
                    if len(row[ii])>0:
                        prm_conf['PRM'][PmName]['map'][row[1]].append({
                            'subframe':row[ii],
                            'word':row[ii+1],
                            'lsb':row[ii+2],
                            'msb':row[ii+3],
                            'target':row[ii+4],
                            })
                #print(row)
            elif line_tr.startswith('PA41'):  #记录参数
                row=split_line(line_tr)
                #print(row)
                if row[1] not in prm_conf['PRM'][PmName]['res']:
                    prm_conf['PRM'][PmName]['res'][row[1]]={}
                else:
                    print('ERR,PA41 转换系数编号重复,',PmName)
                prm_conf['PRM'][PmName]['res'][row[1]]={
                        'MinValue':row[2],
                        'MaxValue':row[3],
                        'EUConvType':row[4],
                        'resI':row[5],
                        'resolutionA':row[6],
                        'resolutionB':row[7],
                        'resolutionC':row[8],
                        }
                #print('==>',row[5])
            elif line_tr.startswith('PA50'):  #记录参数
                row=split_line(line_tr)
                #print(row)
            elif line_tr.startswith('PA62'):  #记录参数
                row=split_line(line_tr)
                if row[1] not in prm_conf['PRM'][PmName]['enum']:
                    prm_conf['PRM'][PmName]['enum'][row[1]]=[]
                for ii in range(1,len(row)-1,2):
                    if len(row[ii])>0:
                        prm_conf['PRM'][PmName]['enum'][row[1]].append({
                            'val':row[ii],
                            'text':row[ii+1],
                            })
                #print(row)
            elif line_tr.startswith('PA70'):  #记录参数
                row=split_line(line_tr)
                prm_conf['PRM'][PmName]['superframe']=row[3]
                #print(row)
    return prm_conf       #返回list

def split_line(line):
    CONF={
            # DAT1 HDR03 PA11 PA12 PA17 PA21 PA22 PA31 PA32 PA41 PA50 PA62 PA70 SUP1
            'HDR03':[0,6,9,12,20,29],
            #HDR03, ?, ?, 日期, ?, FDR
            'DAT1':[0,5,8,13,16,19],
            #DAT1, ?, word/sec, ?, ?, ?
            'SUP1':[0,5,8,12,17,22,25,28],
            #SUP1, 超级帧计数器序号?, frame/cycle, subframe, word, LSB低位, MSB高位,targetBit
            'PA11':[0,5,12,19,28,32],
            #PA11, ?, ?, 参数名简称(会被截断), 参数名全称
            'PA12':[0,5,7,16,25,27,53,63],
            #PA12, ?, 采样类型(Type), 记录格式(RecFormat), 是否有符号位, 计量单位, 日期, ?
            #工程值是否有符号，似乎不是以这个为准，而是以SignRecType为准。
            'PA17':[0,5],
            #PA17, 参数名简称(如果PA11没显示完整)
            'PA21':[0,5,9,17,20,23,26,33],
            #PA21, 记录次数/4sec, BCD取值方式/空白, 单个记录分几段保存, ?, ?, ?, ?
            #PA21, Rate(sample/frame), ConvConfig/空白, 单个记录分几段, SignRecType, FlagType, ?, ?
            # 单个记录分几段: 通常指一行PA31中有几个分段。
            # SignRecType=00,01,只有这两个值。可能表示原始值中的最高位是否为符号位。
            #FlagType:只有''(空),0,4,5四种值。
            # 最后两个值，都是0; 即:'000000 00'
            'PA22':[0,5,19,33,47],
            #PA22, EU LowerOperRange, EU UpperOperRange, ?, ?
            #PA22, 取值范围最小, 取值范围最大, ?, ?
            # 最后两个值，都是1和0; 即:'1.000000e+00 0'; 怀疑是原始值的转换系数。
            'PA31':[0,5,9, 16,21,26,29,32, 35,40,45,48,51, 54,59,64,67,70 ],
            #(会有多行) PA31, 序号SampleNum,ComponentsNum,  SubFrame,WordNum,LSB低位,MSB高位,TargetBit,  
            # 同一行的多个部分，解码时需要拼接。如果TargetBit=0,则按顺序首尾拼接. 如有三组A,B,C, 则A在高位,C在低位。
            # 不同行的配置，应该是独立记录位置。
            # SubFrame: "ALL"表示四个子帧都有记录; 单个数字"2",表示仅一个子帧有记录; 逗号分割的数字"2,4",表示多个子帧有记录。通常用一行中subFrame是相同的，但有三四个参数例外。
            # TargetBit: 如果=0,默认方式拼接。>0 则为LSB, 1则LSB为0.
            'PA32':[0,5,9, 16,21,26,29,32, 35,40,45,48,51, 54,59,64,67,70],
            #PA32,  (与PA31相同) 
            # 如果有PA32，则应该与同序号的 PA31合并到 同一行。PA21中"分几段"也证实,需要合并。
            'PA41':[0,5,8,16,24,29,33,47,61],
            #(会有多行) PA41, 序号, MinValue, MaxValue, EU ConvType?, 系数编号resI, 转换系数0, 转换系数1, 转换系数2, (多行的情况,不知道怎么换算的)
            # EU=equation
            # EUConvType:只有0,1两种值; 系数编号resI:只有1,2两种值;
            # EUConvType=0; 无需转换
            # EUConvType=1,系数编号resI=1; 转换公式为 VAL=系数0 + 系数1 * X
            # EUConvType=1,系数编号resI=2; 转换公式为 VAL=系数0 + 系数1 * X + 系数2 * (X*X)
            'PA50':[0,5,18,31,44,57,59],
            #PA50, 参数名简称/空白, 参数名全称/空白, 单位/空白, 输出的显示格式, ?, ?
            'PA62':[0, 5,11, 24,30, 43,49, 62,68],
            #PA62, 数值,枚举值,  (两个一对，离散枚举值列表)
            'PA70':[0,5,8,16],
            #PA70, 编号, 空白, superframe位置
            }
    arr=[]
    pre=''
    for key in CONF.keys():
        if line.startswith(key):
            pre=key
            break
    if len(pre)<1:
        print('ERR,line header NOT found.',flush=True)
        return []
    num=len(CONF[pre])
    for ii in range(0,num-1):
        #arr.append( line[ CONF[pre][ii] : CONF[pre][ii+1] ] )
        arr.append( line[ CONF[pre][ii] : CONF[pre][ii+1] ].strip() )
    #arr.append( line[ CONF[pre][num-1] : ] )
    arr.append( line[ CONF[pre][num-1] : ].strip() )
    return arr

def getPRM(dataver,param):
    '''
    获取参数在arinc717的12bit word中的位置配置
    挑出有用的,整理一下,返回
       author: osnosn@126.com
    '''
    prm_conf=read_parameter_file(dataver)
    if len(prm_conf['DAT'])<1 or len(prm_conf['PRM'])<1:
        print('Empty dataVer.',flush=True)
        return {}

    ret2=[]  #for regular
    ret3=[]  #for superframe
    ret4=[]  #for superframe pm
    if len(param)>0:
        param=param.upper() #改大写
        #---find regular parameter----
        #---find superframe parameter----
        for kk,vv in prm_conf['PRM'].items():
            if vv['name'].upper() == param:
                if 'superframe' in vv:
                    print('  ---super frame 参数---')
                    for mapk,mapv in vv['map'].items():
                        for tmp in mapv:
                            tmp2=[ #superframe 单一参数记录
                                  int(mapk),   #part(1,2,3),会有多组记录,对应返回多个32bit word. 同一组最多3个part,3个part分别读出,写入同一个32bit word.
                                  int(vv['rate']), #period, 周期,每几个frame出现一次
                                  0, #superframe no, 对应"superframe全局配置"中的superframe no
                                  int(tmp['subframe']) if tmp['subframe']!='ALL' else 0,   #Frame,  位于第几个frame (由superframe counter,找出编号为1的frame)
                                  int(tmp['msb']),   #bitOut, 在12bit中,第几个bit开始
                                  int(tmp['msb'])-int(tmp['lsb'])+1,   #bitLen, 共几个bits
                                  int(tmp['target']),   #bitIn,  写入arinc429的32bits word中,从第几个bits开始写
                                  vv['res']['01']['resolutionB'] if len(vv['res'])>0 else 1,  #resolutionB, 未用到
                                  ]
                            ret4.append(tmp2)
                else:
                    print('  ---regular 参数---')
                    for mapk,mapv in vv['map'].items():
                        for tmp in mapv:
                            tmp2=[  #regular 参数配置
                                  int(mapk),   #part(1,2,3),会有多组记录,对应返回多个32bit word. 同一组最多3个part,3个part分别读出,写入同一个32bit word.
                                  int(vv['rate']),   #recordRate, 记录频率(记录次数/Frame)
                                  int(tmp['subframe']) if tmp['subframe']!='ALL' else 0,   #subframe, 位于哪个subframe(1-4)
                                  int(tmp['word']),   #word, 在subframe中第几个word(sync word编号为1)
                                  int(tmp['msb']),   #bitOut, 在12bit中,第几个bit开始
                                  int(tmp['msb'])-int(tmp['lsb'])+1,   #bitLen, 共几个bits
                                  int(tmp['target']),   #bitIn,  写入arinc429的32bits word中,从第几个bits开始写
                                  0,  #Occurence No
                                  'Imposed',   #Imposed,Computed
                                  ]
                            ret2.append(tmp2)
        tmp=prm_conf['SUP']
        tmp2=[ #superframe 全局配置
              0,   #superframe no
              int(tmp[3]),   #subframe, 位于哪个subframe(1-4)
              int(tmp[4]),   #word, 在subframe中第几个word(sync word编号为1)
              int(tmp[6]),   #bitOut, 在12bit中,第几个bit开始(通常=12)
              int(tmp[6])-int(tmp[6])+1,   #bitLen, 共几个bits(通常=12)
              1,   #superframe couter 1/2, 对应Frame总配置中的第几个counter
              ]
        ret3.append(tmp2)
    return { '1':
            [  #Frame 总配置, 最多两条记录(表示有两个counter)
                int(prm_conf['DAT'][2]),  #Word/Sec, 每秒的word数量,即 word/subframe
                12,  #sync length, 同步字长度(bits=12,24,36)
                '247',  #sync1, 同步字,前12bits
                '5B8',  #sync2
                'A47',  #sync3
                'DB8',  #sync4
                int(prm_conf['SUP'][3]),  #subframe, [superframe counter],每个frame中都有,这4项是counter的位置
                int(prm_conf['SUP'][4]),  #word,     [superframe counter]
                int(prm_conf['SUP'][6]),  #bitOut,   [superframe counter]
                int(prm_conf['SUP'][6])-int(prm_conf['SUP'][5])+1, #bitLen,   [superframe counter]
                1, #Value in 1st frame (0/1), 编号为1的frame,counter的值(counter的最小值)
                ],
             '2':ret2,
             '3':ret3,
             '4':ret4,
            }

import os,sys,getopt
def usage():
    print(u'Usage:')
    print(u'   命令行工具。')
    print(u' 读解码库，参数配置文件 xx.PRM ')
    print(sys.argv[0]+' [-h|--help]')
    print('   -h, --help        print usage.')
    print('   -v, --ver=10XXX      dataver 中的参数配置表')
    print('   --csv xxx.csv        save to "xxx.csv" file.')
    print('   --csv xxx.csv.gz     save to "xxx.csv.gz" file.')
    print('   -l,--paramlist       list all param name.')
    print('   -p,--param alt_std   show "alt_std" param.')
    print('   -j,--paramjson       dump all param config TO "json" format.')
    print(u'\n               author: osnosn@126.com')
    print()
    return
if __name__=='__main__':
    if(len(sys.argv)<2):
        usage()
        exit()
    try:
        opts, args = getopt.gnu_getopt(sys.argv[1:],'hlv:p:f:dj',['help','ver=','csv=','paramlist','paramjson','param='])
    except getopt.GetoptError as e:
        print(e)
        usage()
        exit(2)
    FNAME=None
    DUMPDATA=False
    TOCSV=''
    PARAMLIST=False
    PARAMJSON=False
    PARAM=None
    for op,value in opts:
        if op in ('-h','--help'):
            usage()
            exit()
        elif op in('-v','--ver'):
            FNAME=value
        elif op in('-d',):
            DUMPDATA=True
        elif op in('--csv',):
            TOCSV=value
        elif op in('-l','--paramlist',):
            PARAMLIST=True
        elif op in('-j','--paramjson',):
            PARAMJSON=True
        elif op in('--param','-p',):
            PARAM=value
    if len(args)>0:  #命令行剩余参数
        FNAME=args[0]  #只取第一个
    if FNAME is None:
        usage()
        exit()

    main()

