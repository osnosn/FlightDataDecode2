//!    author: osnosn@126.com   

use std::collections::HashMap;

pub struct PrmConf {
    pub param: HashMap<String, Param>,
    pub WordPerSec: usize,
    pub SuperFramePerCycle: usize,
}
///单个记录参数的配置
pub struct Param {
    pub words: Vec<Vec<usize>>, // [ subframe,word,lsb,msb,targetBit]
    // subframe=0,为 ALL, 即,4个subframe都有记录
    // targetBit=0,为默认拼接方式
    pub superframe: usize, //0=非超级帧参数
    pub res: [f32; 2],     //系数 A,B; 转换公式, A+B*X
    pub signed: bool,      //true=1,有符号; false=0,无符号;
    pub RecFormat: RecFormat,
    pub ConvConfig: Vec<u8>, // 1443 BCD
    pub Unit: String,        //计量单位。解码过程未使用,可以不填写
    pub LongName: String,    //解码过程未使用,可以不填写
}
///记录格式/类型
pub enum RecFormat {
    BNR,
    BCD,
    DIS,
    _ISO,
    _BINGMT,
}
impl PrmConf {
    pub fn new() -> Self {
        let mut param = HashMap::new();

        // vrtg 的配置
        let vrtg = Param {
            words: vec![
                // [ subframe,word,lsb,msb,targetBit],
                // subframe=0,为 ALL, 即,4个subframe都有记录
                // targetBit=0,为默认拼接方式
                vec![0, 2, 1, 12, 0],
                vec![0, 34, 1, 12, 0],
                vec![0, 66, 1, 12, 0],
                vec![0, 98, 1, 12, 0],
                vec![0, 130, 1, 12, 0],
                vec![0, 162, 1, 12, 0],
                vec![0, 194, 1, 12, 0],
                vec![0, 226, 1, 12, 0],
                vec![0, 258, 1, 12, 0],
                vec![0, 290, 1, 12, 0],
                vec![0, 322, 1, 12, 0],
                vec![0, 354, 1, 12, 0],
                vec![0, 386, 1, 12, 0],
                vec![0, 418, 1, 12, 0],
                vec![0, 450, 1, 12, 0],
                vec![0, 482, 1, 12, 0],
            ],
            //resA, resB
            res: [-3.37538, 0.00228938],
            signed: false,
            superframe: 0, //0=非超级帧参数
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "G".to_string(),
            LongName: "VERTICAL ACCELERATION ACQ".to_string(),
        };
        param.insert("VRTG".to_string(), vrtg);
        // altstd 的配置
        let altstd = Param {
            words: vec![
                // [ subframe,word,lsb,msb,targetBit, subframe,word,lsb,msb,targetBit],
                vec![0, 47, 3, 11, 0, 0, 46, 5, 12, 0],
                vec![0, 175, 3, 11, 0, 0, 174, 5, 12, 0],
                vec![0, 303, 3, 11, 0, 0, 302, 5, 12, 0],
                vec![0, 431, 3, 11, 0, 0, 430, 5, 12, 0],
            ],
            // resA, resB
            res: [0.0, 1.0],
            signed: true,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "FEET".to_string(),
            LongName: "PRES ALTITUDE".to_string(),
        };
        param.insert("ALTSTD".to_string(), altstd);
        // gs3 的配置
        let gs3 = Param {
            words: vec![
                vec![0, 49, 2, 12, 0],
                vec![0, 177, 2, 12, 0],
                vec![0, 305, 2, 12, 0],
                vec![0, 433, 2, 12, 0],
            ],
            res: [0.0, 0.5],
            signed: false,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "KNTS".to_string(),
            LongName: "GROUND SPEED(IR-3)".to_string(),
        };
        param.insert("GS3".to_string(), gs3);
        // pitch 的配置
        let pitch = Param {
            words: vec![
                vec![0, 3, 3, 12, 0],
                vec![0, 131, 3, 12, 0],
                vec![0, 259, 3, 12, 0],
                vec![0, 387, 3, 12, 0],
            ],
            res: [0.0, 0.1757813],
            signed: true,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "DEGS".to_string(),
            LongName: "CAP DISP PITCH ATT".to_string(),
        };
        param.insert("PITCH".to_string(), pitch);
        // N11 的配置
        let n11 = Param {
            words: vec![vec![0, 110, 3, 12, 0]],
            res: [0.0, 0.125],
            signed: false,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "%RPM".to_string(),
            LongName: "SELTD N1 INDICATED 1".to_string(),
        };
        param.insert("N11".to_string(), n11);
        // N21 的配置
        let n21 = Param {
            words: vec![vec![0, 251, 3, 12, 0]],
            res: [0.0, 0.125],
            signed: false,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "%RPM".to_string(),
            LongName: "SELECTED N2 ACTUAL 1".to_string(),
        };
        param.insert("N21".to_string(), n21);
        // SAT 的配置
        let sat = Param {
            words: vec![vec![3, 249, 3, 12, 0]],
            res: [0.0, 0.25],
            signed: true,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "DEGC".to_string(),
            LongName: "STATIC AIR TEMP".to_string(),
        };
        param.insert("SAT".to_string(), sat);
        // AILACTL 的配置 (AILERON_ACTUATOR_POSN_LT)
        let aileron = Param {
            words: vec![
                vec![0, 82, 3, 12, 0],
                vec![0, 210, 3, 12, 0],
                vec![0, 338, 3, 12, 0],
                vec![0, 466, 3, 12, 0],
            ],
            res: [0.0, 0.03756054],
            signed: true,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "DEGS".to_string(),
            LongName: "AILERON ACTUATOR POSN LT".to_string(),
        };
        param.insert("AILERON".to_string(), aileron);
        // LDGSQTL 的配置 (左主轮空地电门)
        let aileron = Param {
            words: vec![
                vec![0, 5, 2, 2, 0],
                vec![0, 133, 2, 2, 0],
                vec![0, 261, 2, 2, 0],
                vec![0, 389, 2, 2, 0],
            ],
            res: [0.0, 1.0],
            signed: false,
            superframe: 0,
            RecFormat: RecFormat::DIS,
            ConvConfig: vec![],
            Unit: "DEGS".to_string(),
            LongName: "LEFT MAIN GEAR AIR/GND".to_string(),
        };
        param.insert("LDGSQTL".to_string(), aileron);
        // GMT_HOUR 的配置
        let gmth = Param {
            words: vec![vec![1, 256, 8, 12, 0]],
            res: [0.0, 1.0],
            signed: false,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "DEC".to_string(),
            LongName: "GMT HRS".to_string(),
        };
        param.insert("GMTH".to_string(), gmth);
        // GMT_MINUTES 的配置
        let gmtm = Param {
            words: vec![vec![1, 256, 2, 7, 0]],
            res: [0.0, 1.0],
            signed: false,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "DEC".to_string(),
            LongName: "GMT MIN".to_string(),
        };
        param.insert("GMTM".to_string(), gmtm);
        // GMT_SECONDS 的配置
        let gmts = Param {
            words: vec![vec![1, 257, 1, 6, 0]],
            res: [0.0, 1.0],
            signed: false,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "DEC".to_string(),
            LongName: "GMT SEC".to_string(),
        };
        param.insert("GMTS".to_string(), gmts);
        // SUP CNT 的配置
        let gmts = Param {
            words: vec![vec![1, 499, 9, 12, 0]],
            res: [0.0, 1.0],
            signed: false,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "".to_string(),
            LongName: "SUPER FRAME COUNTER".to_string(),
        };
        param.insert("SuperFrameCounter".to_string(), gmts);
        // CAP_CLOCK_DAY 的配置
        let gmts = Param {
            words: vec![vec![4, 257, 2, 7, 0]],
            res: [0.0, 1.0],
            signed: false,
            superframe: 4,
            RecFormat: RecFormat::BCD,
            ConvConfig: vec![2, 4],
            Unit: "".to_string(),
            LongName: "CAP CLOCK DAY".to_string(),
        };
        param.insert("DAY".to_string(), gmts);
        return Self {
            param,
            WordPerSec: 1024,
            SuperFramePerCycle: 16,
        };
    }
}
