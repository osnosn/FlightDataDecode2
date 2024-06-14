#![allow(non_snake_case)]
use std::fs::File;
use std::io::BufReader;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct PrmConf {
    pub param: HashMap<String, Param>,
    pub WordPerSec: usize,
    pub SuperFramePerCycle: usize,
}
///单个记录参数的配置
#[derive(Serialize, Deserialize, Debug)]
pub struct Param {
    pub words: Vec<Vec<usize>>, // [ superframe,subframe,word,lsb,msb,targetBit]
    // superframe=0,为非超级帧参数
    // subframe=0,为 ALL, 即,4个subframe都有记录
    // targetBit=0,为默认拼接方式
    pub superframe: usize,  //0=非超级帧参数,放弃使用,移到words中
    pub res: Vec<[f32; 5]>, //系数 A,B; 转换公式, A+B*X
    pub signed: bool,       //true=1,有符号; false=0,无符号;
    //符号位.PRM配置用signRecType,所以配置read_prm717.py把signRecType写为signed
    //pub signRecType: bool, //true=01,有符号; false=00,无符号;
    pub RecFormat: String,
    pub ConvConfig: Vec<u8>, // 1443 BCD
    pub Unit: String,        //计量单位。解码过程未使用,可以不填写
    pub LongName: String,    //解码过程未使用,可以不填写
    #[serde(default = "default_options")]
    pub Options: Vec<(i16, String)>, //DIS 的枚举值
}
//为Options提供默认值
fn default_options() -> Vec<(i16, String)> {
    vec![]
}

impl PrmConf {
    pub fn json(json_name: &str) -> Self {
        //let json_file = File::open("prm_conf320.json").expect("读取'prm_conf320.json'失败");
        let json_file =
            File::open(json_name).expect(format!("读取json配置文件 \"{json_name}\" 失败").as_str());
        let reader = BufReader::new(json_file);

        // 转换成 Person 结构
        let p: PrmConf = serde_json::from_reader(reader)
            .expect(format!("解析json配置文件 \"{json_name}\" 失败").as_str());

        println!("读取,解析json配置文件 \"{json_name}\" 成功.");

        // 通过方括号建立索引来访问部分数据
        //println!("{:#?}\n", p,);
        return p;
    }
}
