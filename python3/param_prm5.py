#!/usr/bin/python3
# -*- coding: utf-8 -*-

'''
 来自 320.PRM 的配置
'''

param={}
param['VRTG']={
        'words':[
            [0, 9, 1, 12, 1],
            [0, 41, 1, 12, 1],
            [0, 73, 1, 12, 1],
            [0, 105, 1, 12, 1],
            [0, 137, 1, 12, 1],
            [0, 169, 1, 12, 1],
            [0, 201, 1, 12, 1],
            [0, 233, 1, 12, 1],
            ],
        'res': [0.0, 0.00390625],
        'signed': True,
        'signRecType': True,
        'superframe': 0, # 0=非超级帧参数
        'RecFormat': 'BNR',
        'ConvConfig': [],
        'Unit': "G",
        'LongName': "Normal acceleration",
        }
param["ALT_BARO"]={
        'words': [
            # [ subframe,word,lsb,msb,targetBit, subframe,word,lsb,msb,targetBit],
            [0, 716, 1, 12, 1, 0, 715, 8, 12, 0],
            ],
        # resA, resB
        'res': [0.0, 1.0], #默认值(无需换算)
        'signed': True,
        'signRecType': True,
        'superframe': 0,
        'RecFormat': 'BNR',
        'ConvConfig': [],
        'Unit': "ft",
        'LongName': "BARO ALTI CORRECTED #1",
        };
# gs3 的配置
param["GPS_GS_C"]={
        'words': [[0, 747, 1, 12, 17]],
        'res': [0.0, 0.25],
        'signed': True,
        'signRecType': False,
        'superframe': 0,
        'RecFormat': 'BNR',
        'ConvConfig': [],
        'Unit': "kts",
        'LongName': "GPS GROUND SPEED CAPT",
        };
# pitch 的配置
param["PITCH"]={
        'words': [
            [0, 44, 3, 12, 1],
            [0, 172, 3, 12, 1],
            [0, 300, 3, 12, 1],
            [0, 428, 3, 12, 1],
            [0, 556, 3, 12, 0],
            [0, 684, 3, 12, 0],
            [0, 812, 3, 12, 0],
            [0, 940, 3, 12, 0],
            ],
        'res': [0.0, 0.1757813],
        'signed': True,
        'signRecType': True,
        'superframe': 0,
        'RecFormat': 'BNR',
        'ConvConfig': [],
        'Unit': "Â°",
        'LongName': "Pitch attitude CA",
        };
# N11 的配置
param["N1_1"]={
        'words': [[0, 369, 1, 12, 1]],
        'res': [0.0, 0.03125],
        'signed': True,
        'signRecType': False,
        'superframe': 0,
        'RecFormat': 'BNR',
        'ConvConfig': [],
        'Unit': "RPM",
        'LongName': "N1 Actual Eng 1",
        };
# SAT 的配置
param["SAT"]={
        'words': [[1, 521, 3, 12, 16], [3, 521, 3, 12, 16]],
        'res': [0.0, 0.25],
        'signed': True,
        'signRecType': True,
        'superframe': 0,
        'RecFormat': 'BNR',
        'ConvConfig': [],
        'Unit': "Â°C",
        'LongName': "SAT_CA",
        };
# CAS 的配置 
param["CAS"]={
        'words': [[0, 74, 1, 12, 1], [0, 586, 1, 12, 0]],
        'res': [0.0, 0.125],
        'signed': False,
        'signRecType': False,
        'superframe': 0,
        'RecFormat': 'BNR',
        'ConvConfig': [],
        'Unit': "kts",
        'LongName': "Computed airspeed CAPT",
        };
# UTC_HOUR 的配置
param["UTCH"]={
        'words': [[4, 429, 6, 12, 0]],
        'res': [0.0, 1.0],
        'signed': False,
        'signRecType': False,
        'superframe': 0,
        'RecFormat': 'BCD',
        'ConvConfig': [3, 4],
        'Unit': "",
        'LongName': "UTC_HOUR_SYS2",
        };
# UTC_MINUTES 的配置
param["UTCM"]={
        'words': [[4, 225, 7, 12, 1]],
        'res': [0.0, 1.0],
        'signed': False,
        'signRecType': False,
        'superframe': 0,
        'RecFormat': 'BNR',
        'ConvConfig': [],
        'Unit': "",
        'LongName': "UTC Minutes",
        };
# UTC_SECONDS 的配置
param["UTCS"]={
        'words': [[4, 225, 1, 6, 1]],
        'res': [0.0, 1.0],
        'signed': False,
        'signRecType': False,
        'superframe': 0,
        'RecFormat': 'BNR',
        'ConvConfig': [],
        'Unit': "",
        'LongName': "UTC Seconds",
        };
# SUP CNT 的配置
param["SuperFrameCounter"]={
        'words': [[2, 225, 1, 4, 0]],
        'res': [0.0, 1.0],
        'signed': False,
        'signRecType': False,
        'superframe': 0,
        'RecFormat': 'BNR',
        'ConvConfig': [],
        'Unit': "",
        'LongName': "SUPER FRAME COUNTER",
        };
# CAP_CLOCK_DAY 的配置
param["DAY"]={
        'words': [[1, 17, 1, 6, 0]],
        'res': [0.0, 1.0],
        'signed': False,
        'signRecType': False,
        'superframe': 4,
        'RecFormat': 'BCD',
        'ConvConfig': [2, 4],
        'Unit': "",
        'LongName': "DAY",
        };
PrmConf={
        'param':param,
        'WordPerSec': 1024,
        'SuperFramePerCycle': 16,
        };

