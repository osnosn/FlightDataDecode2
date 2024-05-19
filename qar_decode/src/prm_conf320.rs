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
    //符号位不知道以那个为准.
    pub signRecType: bool, //true=01,有符号; false=00,无符号;
    pub RecFormat: RecFormat,
    pub ConvConfig: Vec<u8>, // 1443 BCD
    pub Unit: String,        //计量单位。解码过程未使用,可以不填写
    pub LongName: String,    //解码过程未使用,可以不填写
}
///记录格式/类型
#[derive(PartialEq)]
pub enum RecFormat {
    BNR,
    BCD,
    _DIS,
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
                vec![0, 9, 1, 12, 1],
                vec![0, 41, 1, 12, 1],
                vec![0, 73, 1, 12, 1],
                vec![0, 105, 1, 12, 1],
                vec![0, 137, 1, 12, 1],
                vec![0, 169, 1, 12, 1],
                vec![0, 201, 1, 12, 1],
                vec![0, 233, 1, 12, 1],
            ],
            //resA, resB
            res: [0.0, 0.00390625],
            signed: true,
            signRecType: true,
            superframe: 0, //0=非超级帧参数
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "G".to_string(),
            LongName: "Normal acceleration".to_string(),
        };
        param.insert("VRTG".to_string(), vrtg);
        // altstd 的配置
        let altstd = Param {
            words: vec![
                // [ subframe,word,lsb,msb,targetBit, subframe,word,lsb,msb,targetBit],
                vec![0, 716, 1, 12, 1, 0, 715, 8, 12, 0],
            ],
            // resA, resB
            res: [0.0, 1.0], //默认值(无需换算)
            signed: true,
            signRecType: true,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "ft".to_string(),
            LongName: "BARO ALTI CORRECTED #1".to_string(),
        };
        param.insert("ALT_BARO".to_string(), altstd);
        // gs3 的配置
        let gs3 = Param {
            words: vec![vec![0, 747, 1, 12, 17]],
            res: [0.0, 0.25],
            signed: true,
            signRecType: false,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "kts".to_string(),
            LongName: "GPS GROUND SPEED CAPT".to_string(),
        };
        param.insert("GPS_GS_C".to_string(), gs3);
        // pitch 的配置
        let pitch = Param {
            words: vec![
                vec![0, 44, 3, 12, 1],
                vec![0, 172, 3, 12, 1],
                vec![0, 300, 3, 12, 1],
                vec![0, 428, 3, 12, 1],
                vec![0, 556, 3, 12, 0],
                vec![0, 684, 3, 12, 0],
                vec![0, 812, 3, 12, 0],
                vec![0, 940, 3, 12, 0],
            ],
            res: [0.0, 0.1757813],
            signed: true,
            signRecType: true,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "Â°".to_string(),
            LongName: "Pitch attitude CA".to_string(),
        };
        param.insert("PITCH".to_string(), pitch);
        // N11 的配置
        let n11 = Param {
            words: vec![vec![0, 369, 1, 12, 1]],
            res: [0.0, 0.03125],
            signed: true,
            signRecType: false,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "RPM".to_string(),
            LongName: "N1 Actual Eng 1".to_string(),
        };
        param.insert("N1_1".to_string(), n11);
        // SAT 的配置
        let sat = Param {
            words: vec![vec![1, 521, 3, 12, 16], vec![3, 521, 3, 12, 16]],
            res: [0.0, 0.25],
            signed: true,
            signRecType: true,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "Â°C".to_string(),
            LongName: "SAT_CA".to_string(),
        };
        param.insert("SAT".to_string(), sat);
        // CAS 的配置
        let cas = Param {
            words: vec![vec![0, 74, 1, 12, 1], vec![0, 586, 1, 12, 0]],
            res: [0.0, 0.125],
            signed: false,
            signRecType: false,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "kts".to_string(),
            LongName: "Computed airspeed CAPT".to_string(),
        };
        param.insert("CAS".to_string(), cas);
        // UTC_HOUR 的配置
        let gmth = Param {
            words: vec![vec![4, 429, 6, 12, 0]],
            res: [0.0, 1.0],
            signed: false,
            signRecType: false,
            superframe: 0,
            RecFormat: RecFormat::BCD,
            ConvConfig: vec![3, 4],
            Unit: "".to_string(),
            LongName: "UTC_HOUR_SYS2".to_string(),
        };
        param.insert("UTCH".to_string(), gmth);
        // UTC_MINUTES 的配置
        let gmtm = Param {
            words: vec![vec![4, 225, 7, 12, 1]],
            res: [0.0, 1.0],
            signed: false,
            signRecType: false,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "".to_string(),
            LongName: "UTC Minutes".to_string(),
        };
        param.insert("UTCM".to_string(), gmtm);
        // UTC_SECONDS 的配置
        let gmts = Param {
            words: vec![vec![4, 225, 1, 6, 1]],
            res: [0.0, 1.0],
            signed: false,
            signRecType: false,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "".to_string(),
            LongName: "UTC Seconds".to_string(),
        };
        param.insert("UTCS".to_string(), gmts);
        // SUP CNT 的配置
        let gmts = Param {
            words: vec![vec![2, 225, 1, 4, 0]],
            res: [0.0, 1.0],
            signed: false,
            signRecType: false,
            superframe: 0,
            RecFormat: RecFormat::BNR,
            ConvConfig: vec![],
            Unit: "".to_string(),
            LongName: "SUPER FRAME COUNTER".to_string(),
        };
        param.insert("SuperFrameCounter".to_string(), gmts);
        // CAP_CLOCK_DAY 的配置
        let gmts = Param {
            words: vec![vec![1, 17, 1, 6, 0]],
            res: [0.0, 1.0],
            signed: false,
            signRecType: false,
            superframe: 4,
            RecFormat: RecFormat::BCD,
            ConvConfig: vec![2, 4],
            Unit: "".to_string(),
            LongName: "DAY".to_string(),
        };
        param.insert("DAY".to_string(), gmts);
        return Self {
            param,
            WordPerSec: 1024,
            SuperFramePerCycle: 16,
        };
    }
}
