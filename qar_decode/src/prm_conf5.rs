use std::collections::HashMap;

pub struct PrmConf {
    pub words: Vec<Vec<usize>>,
    pub res: [f32; 3],
    pub supframe: usize,
}
impl PrmConf {
    pub fn new() -> HashMap<String, PrmConf> {
        let mut param = HashMap::new();

        // vrtg 的配置
        let vrtg = PrmConf {
            words: vec![
                // [ subframe,word,lsb,msb,targetBit],
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
            //signed, resA, resB
            res: [0.0, -3.37538, 0.00228938],
            supframe: 0,
        };
        param.insert("VRTG".to_string(), vrtg);
        // altstd 的配置
        let altstd = PrmConf {
            words: vec![
                // [ subframe,word,lsb,msb,targetBit, subframe,word,lsb,msb,targetBit],
                vec![0, 47, 3, 11, 0, 0, 46, 5, 12, 0],
                vec![0, 175, 3, 11, 0, 0, 174, 5, 12, 0],
                vec![0, 303, 3, 11, 0, 0, 302, 5, 12, 0],
                vec![0, 431, 3, 11, 0, 0, 430, 5, 12, 0],
            ],
            //signed, resA, resB
            res: [1.0, 0.0, 1.0],
            supframe: 0,
        };
        param.insert("ALTSTD".to_string(), altstd);
        // gs3 的配置
        let gs3 = PrmConf {
            words: vec![
                vec![0, 49, 2, 12, 0],
                vec![0, 177, 2, 12, 0],
                vec![0, 305, 2, 12, 0],
                vec![0, 433, 2, 12, 0],
            ],
            res: [0.0, 0.0, 0.5],
            supframe: 0,
        };
        param.insert("GS3".to_string(), gs3);
        // pitch 的配置
        let pitch = PrmConf {
            words: vec![
                vec![0, 3, 3, 12, 0],
                vec![0, 131, 3, 12, 0],
                vec![0, 259, 3, 12, 0],
                vec![0, 387, 3, 12, 0],
            ],
            res: [1.0, 0.0, 0.1757813],
            supframe: 0,
        };
        param.insert("PITCH".to_string(), pitch);
        // N11 的配置
        let n11 = PrmConf {
            words: vec![vec![0, 110, 3, 12, 0]],
            res: [0.0, 0.0, 0.125],
            supframe: 0,
        };
        param.insert("N11".to_string(), n11);
        // N21 的配置
        let n21 = PrmConf {
            words: vec![vec![0, 251, 3, 12, 0]],
            res: [0.0, 0.0, 0.125],
            supframe: 0,
        };
        param.insert("N21".to_string(), n21);
        // SAT 的配置
        let sat = PrmConf {
            words: vec![vec![3, 249, 3, 12, 0]],
            res: [1.0, 0.0, 0.25],
            supframe: 0,
        };
        param.insert("SAT".to_string(), sat);
        // AILACTL 的配置 (AILERON_ACTUATOR_POSN_LT)
        let aileron = PrmConf {
            words: vec![
                vec![0, 82, 3, 12, 0],
                vec![0, 210, 3, 12, 0],
                vec![0, 338, 3, 12, 0],
                vec![0, 466, 3, 12, 0],
            ],
            res: [1.0, 0.0, 0.03756054],
            supframe: 0,
        };
        param.insert("AILERON".to_string(), aileron);
        // GMT_HOUR 的配置
        let gmth = PrmConf {
            words: vec![vec![1, 256, 8, 12, 0]],
            res: [0.0, 0.0, 1.0],
            supframe: 0,
        };
        param.insert("GMTH".to_string(), gmth);
        // GMT_MINUTES 的配置
        let gmtm = PrmConf {
            words: vec![vec![1, 256, 2, 7, 0]],
            res: [0.0, 0.0, 1.0],
            supframe: 0,
        };
        param.insert("GMTM".to_string(), gmtm);
        // GMT_SECONDS 的配置
        let gmts = PrmConf {
            words: vec![vec![1, 257, 1, 6, 0]],
            res: [0.0, 0.0, 1.0],
            supframe: 0,
        };
        param.insert("GMTS".to_string(), gmts);
        // SUP CNT 的配置
        let gmts = PrmConf {
            words: vec![vec![1, 499, 9, 12, 0]],
            res: [0.0, 0.0, 1.0],
            supframe: 0,
        };
        param.insert("SUP_COUNTER".to_string(), gmts);
        // CAP_CLOCK_DAY 的配置
        let gmts = PrmConf {
            words: vec![vec![4, 257, 2, 7, 0]],
            res: [0.0, 0.0, 1.0],
            supframe: 4,
        };
        param.insert("DAY".to_string(), gmts);
        return param;
    }
}
