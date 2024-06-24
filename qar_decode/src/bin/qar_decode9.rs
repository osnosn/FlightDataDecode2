#![allow(non_snake_case)]
//! 使用mmap把raw文件映射到内存，然后再解码参数。  
//!   解码过程,按照Frame为单位循环获取参数的值.如果参数内容来自不同subframe,也能处理。
//! 解码配置来自json文件。  
//!     通过 read_prm717.py    把PRM配置转换为json格式。  
//!     通过 VEC717_to_json.py 把VEC配置转换为json格式。  
//! 所有的解码配置集中在 prm_conf::PrmConf 中。  
//! 增加命令行参数，显示内存占用，增加同步字顺序警告。  
//! 增加subframe判断，处理了符号位。  
//! 更改使用hashmap保存配置。配置中增加targetBit。  
//! 增加superFrame配置。尝试解码超级帧参数。  
//! 增加BCD格式的处理。
//!    author: osnosn@126.com   

// 我的测试 Intel CPU i9,x64,主频3.3GHz, BogoMIPS:6600。
// 原始文件 raw.dat 21MB。有参数 1080 个, 航段170分钟。
//     解所有参数，写入单文件,bzip2压缩, 1.4MB，耗时0m6.2s.
// 原始文件 raw.dat 15MB。有参数 2350 个, 航段121分钟。
//     解所有参数，写入单文件,bzip2压缩, 1.5MB，耗时0m5.8s.
// 原始文件 raw 115MB，压缩包为23MB。有参数 2770 个, 航段960分钟。
//     解所有参数，写入单文件,bzip2压缩, 8.0MB，耗时1m10s. 内存占用130-153MB.
// python3 版,用时 58s到1m50s.

use memmap2::Mmap;
use memmap2::MmapOptions;
use serde::Serialize;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
#[cfg(target_os = "linux")]
use std::process;

use rust_embed_for_web::{EmbedableFile, RustEmbed};
#[derive(RustEmbed)]
#[folder = "embedFile/"]
#[gzip = true]
struct Asset;

#[path = "../CmdLineArgs8.rs"]
mod CmdLineArgs;

#[path = "../prm_conf_json.rs"]
mod prm_conf;

#[derive(Serialize, Debug)]
pub struct PrmDict<T> {
    t: f32,
    v: T,
}
#[derive(Serialize, Debug)]
pub struct PrmValue {
    val: PrmType,
    info: String,
}
#[derive(Serialize, Debug)]
pub enum PrmType {
    Float(Vec<PrmDict<f32>>),
    Int(Vec<PrmDict<i32>>),
    Str(Vec<PrmDict<String>>),
}

fn main() {
    let args;
    match CmdLineArgs::parse_args() {
        Ok(tmp) => args = tmp,
        Err(err) => {
            eprintln!("Command line parse ERR.\r\n{err}");
            return ();
        }
    }
    if args.help || args.help2 {
        showHelp(args.bin_name);
        return ();
    }

    if args.custom_detail {
        //显示Custom_DataFile_Description
        if let Some(embedFile) = Asset::get("Custom_DataFile_Format_Description.txt") {
            // embedFile.data() -> &[u8]
            let Custom_Detail = embedFile.data();
            //Custom_Detail = embedFile.data_gzip().expect("Custom_DataFile_Format_Description.txt ReadError"); //返回gzip压缩后的内容
            println!("{}", String::from_utf8_lossy(&Custom_Detail));
        } else {
            println!("Custom_DataFile_Format_Description.txt NotFound.");
        }
        return ();
    }

    //参数配置,创建
    if args.json.len() < 2 {
        // "prm_conf320.json"
        println!();
        showHelp(args.bin_name);
        eprintln!();
        eprintln!("没有指定\"解码配置\"json文件, Missing \"-j prm_conf.json\".");
        eprintln!("\r\n");
        return ();
    }
    let prm_conf = prm_conf::PrmConf::json(args.json.as_str());

    if args.paramlist {
        //列出配置中,所有参数名称
        paramlist(prm_conf);
        return ();
    }

    // 读取的文件名
    let filename_read;
    if args.rawfile.len() < 2 {
        println!();
        showHelp(args.bin_name);
        eprintln!();
        eprintln!("没有指定raw原始数据文件, Missing \"-r raw.dat\".");
        eprintln!();
        return ();
        //默认文件名
        //filename_read = "data/raw320.dat";
    } else {
        filename_read = args.rawfile.as_str();
    }

    if args.allparam {
        //解码所有参数名称
        allparam(filename_read, &prm_conf, &args);
    } else if args.param.len() < 2 {
        println!();
        showHelp(args.bin_name);
        eprintln!();
        eprintln!("没有指定参数名称 -p , --param");
        eprintln!();
        return ();
    } else {
        // 打开数据文件
        let rfile: File = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(filename_read)
            .expect(format!("无法打开RAW数据文件:\"{}\"", filename_read).as_str());
        println!("RAW数据文件打开成功：{}", filename_read);

        // 使用mmap映射
        let buf = unsafe { MmapOptions::new().map(&rfile).expect("Mmap映射创建失败") };
        let buflen = buf.len();

        //每次都要取值的参数配置
        let prm_superFrameCnt_prm = prm_conf
            .param
            .get("SuperFrameCounter")
            .expect("参数没找到:'SuperFrameCounter'");
        let prm_superFrameCnt = prm_superFrameCnt_prm.words[0].clone();

        // 解码一个参数
        let PrmValue = get_param(
            &buf,
            buflen,
            &prm_conf,
            &prm_superFrameCnt_prm,
            &prm_superFrameCnt,
            &args.param,
            0xff, //verbose
        );

        // 写入的文件名
        let filename_write = args.outfile.as_str();
        if filename_write.len() < 2 {
            eprintln!();
            eprintln!("需要写入的CSV文件名,未指定.");
            eprintln!("Using \"-w out.csv\" 把解码后的数据写入out.csv文件.\r\n");
            return ();
        } else {
            // 创建一个文件，用于写入csv格式的数据
            let mut wfile = File::create(filename_write)
                .expect(format!("创建文件失败:\"{}\"", filename_write).as_str());
            wfile
                .write_all("time,value\r\n".as_bytes())
                .expect("写入失败");

            match PrmValue.val {
                // 以csv格式写入文件
                PrmType::Float(val) => {
                    for vv in val {
                        wfile
                            .write_all(format!("{:.5},{:?}\r\n", vv.t, vv.v).as_bytes())
                            .expect("写入失败");
                    }
                }
                PrmType::Int(val) => {
                    for vv in val {
                        wfile
                            .write_all(format!("{:.5},{:?}\r\n", vv.t, vv.v).as_bytes())
                            .expect("写入失败");
                    }
                }
                PrmType::Str(val) => {
                    for vv in val {
                        wfile
                            .write_all(format!("{:.5},{:?}\r\n", vv.t, vv.v).as_bytes())
                            .expect("写入失败");
                    }
                }
            }

            println!();
            println!(" The length of data is {}.", buflen);
            println!(
                " Parameter \"{}\" write to CSV file: \"{}\".",
                args.param, filename_write
            );
        }
        println!();
    }
    show_mem(&args);
}
//对于Linux系统，显示内存占用情况
fn show_mem(args: &CmdLineArgs::Args) {
    #[cfg(target_os = "linux")]
    if args.mem {
        // --begin--查看内存占用(linux)
        use std::io::{BufRead, BufReader};
        //println!(" PID is {}.", process::id());
        let status_file = File::open(format!("/proc/{}/status", process::id()))
            .expect("读取'/proc/?/status'失败");
        let vm_keys = ["Name", "VmPeak", "VmSize", "VmHWM", "VmRSS", "VmData"];
        for line in BufReader::new(status_file).lines().flatten() {
            for vm_key in vm_keys {
                if line.contains(vm_key) {
                    println!(" {}.", line);
                    break;
                }
            }
        }
        // --end--查看内存占用(linux)
    }
    #[cfg(target_os = "windows")]
    if args.mem {
        // --begin--查看内存占用(windows)
        eprintln!(" Windows 不支持 --mem 参数。");
        eprintln!("   因为 Windows 不支持 /proc/pid/status 文件的查看。");
    }
}
#[derive(Serialize, Debug)]
pub struct OneParamTable {
    selfsize: u16,
    data_point: u64,
    data_size: u32,
    val_size: u16,
    rate: i16,
    start_frameid: f32,
    name: String,
    compress: String,
    data_type: String,
    info: String,
}
use std::io::Error;
//use bincode::Error;
//use bincode::{serialize, Error};
//use std::io::Write;
impl OneParamTable {
    //自定义struct的序列化.LittleEndian.
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&self.selfsize.to_le_bytes())?;
        writer.write_all(&self.data_point.to_le_bytes())?;
        writer.write_all(&self.data_size.to_le_bytes())?;
        writer.write_all(&self.val_size.to_le_bytes())?;
        writer.write_all(&self.rate.to_le_bytes())?;
        writer.write_all(&self.start_frameid.to_le_bytes())?;
        writer.write_all(&self.name.as_bytes())?;
        writer.write_all(&[0])?; //写入一个 0 作为字符串结束.
        writer.write_all(&self.compress.as_bytes())?;
        writer.write_all(&[0])?; //写入一个 0 作为字符串结束.
        writer.write_all(&self.data_type.as_bytes())?;
        writer.write_all(&[0])?; //写入一个 0 作为字符串结束.
        writer.write_all(&self.info.as_bytes())?;
        writer.write_all(&[0])?; //写入一个 0 作为字符串结束.
        Ok(())
    }
    fn length(&self) -> usize {
        let mut size = 2 + 8 + 4 + 2 + 2 + 4;
        size += &self.name.len() + 1;
        size += &self.compress.len() + 1;
        size += &self.data_type.len() + 1;
        size += &self.info.len() + 1;
        size
    }
}
//解码所有参数名称
fn allparam(filename_read: &str, prm_conf: &prm_conf::PrmConf, args: &CmdLineArgs::Args) {
    // 打开数据文件
    let rfile: File = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(filename_read)
        .expect(format!("无法打开RAW数据文件:\"{}\"", filename_read).as_str());
    println!("RAW数据文件打开成功：{}", filename_read);

    // 使用mmap映射
    let rawbuf = unsafe { MmapOptions::new().map(&rfile).expect("Mmap映射创建失败") };
    let rawbuflen = rawbuf.len();

    //每次都要取值的参数配置
    let prm_superFrameCnt_prm = prm_conf
        .param
        .get("SuperFrameCounter")
        .expect("参数没找到:'SuperFrameCounter'");
    let prm_superFrameCnt = prm_superFrameCnt_prm.words[0].clone();

    let mut PRM_data: Vec<Vec<u8>> = vec![];
    let mut PRM_table: Vec<OneParamTable> = vec![];
    let mut param_table_total_size: u32 = 0;

    let mut ii_prm = 0; //解码的参数计数
    for param in prm_conf.param.keys() {
        ii_prm += 1;
        // 解码一个参数
        let PrmValue = get_param(
            &rawbuf,
            rawbuflen,
            &prm_conf,
            &prm_superFrameCnt_prm,
            &prm_superFrameCnt,
            &param,
            0xf, //verbose
        );

        let mut buf: Vec<u8> = vec![]; //序列化的缓存
        let mut one_param_table = OneParamTable {
            selfsize: 3,
            data_point: 2,
            data_size: 0,
            val_size: 4,
            rate: 4,
            start_frameid: 1.0,
            name: param.clone(),
            compress: "bzip2".to_string(),
            data_type: "float".to_string(),
            info: PrmValue.info,
        };
        let mut param_rate: f32 = 0.0;
        // 以自定义格式写入dat文件
        match &PrmValue.val {
            PrmType::Float(val) => {
                /*
                let buf = serde_json::to_string(&val).expect("serde_json::to_string失败");
                wfile.write_all(buf.as_bytes()).expect("写入失败");
                */
                one_param_table.val_size = 4; //单个值的size
                one_param_table.data_type = "float".to_string();
                if val.len() > 2 {
                    param_rate = val[1].t - val[0].t;
                    one_param_table.start_frameid = val[0].t; //一定是个整数, 0.0, 1.0
                }
                //let mut pFloat: Vec<f32> = vec![];
                for vv in val {
                    buf.write_all(&vv.v.to_le_bytes()).unwrap(); //仅返回 Ok(()),不会出错
                                                                 //pFloat.push(vv.v);
                }
                //bincode序列化vec,最前面有8byte的vec size, 不合适使用.
                //buf = bincode::serialize(&pFloat).expect("bincode::serialize(f32)失败");

                //let opt=liblzma_sys::lzma_options_lzma;
                //let buf2:Vec<u8>= liblzma_sys::lzma_alone_encoder(buf,opt);
            }
            PrmType::Int(val) => {
                /*
                let buf = serde_json::to_string(&val).expect("serde_json::to_string失败");
                wfile.write_all(buf.as_bytes()).expect("写入失败");
                */
                one_param_table.val_size = 4; //单个值的size
                one_param_table.data_type = "int".to_string();
                if val.len() > 2 {
                    param_rate = val[1].t - val[0].t;
                    one_param_table.start_frameid = val[0].t; //一定是个整数, 0.0, 1.0
                }
                for vv in val {
                    buf.write_all(&vv.v.to_le_bytes()).unwrap(); //仅返回 Ok(()),不会出错
                }
            }
            PrmType::Str(val) => {
                one_param_table.val_size = 0; //单个值的size
                one_param_table.data_type = "str".to_string();
                if val.len() > 2 {
                    param_rate = val[1].t - val[0].t;
                    one_param_table.start_frameid = val[0].t; //一定是个整数, 0.0, 1.0
                }
                let mut data_arr: Vec<(f32, String)> = vec![];
                for vv in val {
                    data_arr.push((vv.t, vv.v.clone()));
                }
                buf = serde_json::to_string(&data_arr)
                    .expect("serde_json::to_string失败")
                    .into_bytes();
            }
        }
        if param_rate != 0.0 && param_rate <= 1.0 {
            one_param_table.rate = (1.0 / param_rate) as i16;
        } else {
            one_param_table.rate = (param_rate * -1.0) as i16;
        }
        use std::io::Read;
        //show_mem(&args);

        let mut buf2: Vec<u8> = Vec::new(); //压缩的缓存
        if false {
            //使用7zXZ压缩,lzma;
            //--压缩参数9:需占680-700MB内存.
            //--压缩参数7:需占200-220MB内存.
            //--压缩参数6:需占80-100MB内存.
            //--参数 6,7,9,耗时都比bz2多一倍。例如:bz2用6秒,xz要用10-11秒,
            //    -- 文件还比bz2的大一点,1.4M的文件大约多80k.
            //--压缩参数5:需占80-100MB内存. 耗时少了一点点, 但文件更大了.
            //----不知道xz的后门问题,是否受到影响----
            use liblzma::read::XzEncoder;
            one_param_table.compress = "xz".to_string();
            XzEncoder::new(buf.as_slice(), 6)
                .read_to_end(&mut buf2)
                .expect("xz(lzma)失败");
        } else {
            //使用bzip2压缩,bz2; (需占7-9MB内存,占内存较小)
            //use std::io::prelude::*;
            use bzip2::read::BzEncoder;
            use bzip2::Compression;
            one_param_table.compress = "bzip2".to_string();
            BzEncoder::new(buf.as_slice(), Compression::best())
                .read_to_end(&mut buf2)
                .expect("bzip2失败");
        }

        one_param_table.selfsize = one_param_table.length() as u16; //单个table自身的size
        param_table_total_size += one_param_table.selfsize as u32; //用于指向DATA的指针,加上param_table的长度
        one_param_table.data_size = buf2.len() as u32; //压缩数据的size
        PRM_data.push(buf2);
        PRM_table.push(one_param_table);

        //show_mem(&args);
    }
    show_mem(&args);
    println!("解码完成，准备写入dat文件.");

    // 写入的文件名
    let filename_write = args.outfile.as_str();
    if filename_write.len() < 2 {
        eprintln!();
        eprintln!("需要写入的dat文件名,未指定.");
        eprintln!("Using \"-w out.dat\" 把解码后的数据写入out.dat文件.\r\n");
        return ();
    }

    // 创建一个文件，用于写入自定义格式的数据
    let mut wfile = File::create(filename_write)
        .expect(format!("创建文件失败:\"{}\"", filename_write).as_str());

    //------ 写 Header ----------
    let header_tag = b"QAR_Decoded_DATA_V1.0\0"; //自定义数据文件的tag
    wfile.write_all(header_tag).expect("写入失败");
    let meta = r#" {
                "MetaData": {
                    "DataVersion":8888,
                    "ParamConfigFile":"prm.json",
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
                    "FileName":"data/raw.dat"
                    },
                "other":123,
                "info":"This is a test."
                } "#;
    //把json转为Value; 再把Value转为json,为了的到紧凑的json格式
    let mut meta_value: serde_json::Value =
        serde_json::from_str(meta).expect("serde_json::from_str失败");
    meta_value["MetaData"]["ParamConfigFile"] = serde_json::Value::String(args.json.clone());
    meta_value["MetaData"]["FileName"] = serde_json::Value::String(filename_read.to_string());
    let meta_bytes = serde_json::to_vec(&meta_value).expect("serde_json::to_string失败");
    //读取 自定义数据文件的的格式描述
    /*
    let Custom_Detail = std::fs::read("data/Custom_DataFile_Format_Description.txt")
        .expect("读取data/Custom_DataFile_Format_Description.txt失败");
    */
    let Custom_Detail;
    if let Some(embedFile) = Asset::get("Custom_DataFile_Format_Description.txt") {
        // embedFile.data() -> &[u8]
        Custom_Detail = embedFile.data();
    } else {
        Custom_Detail = br#"Custom_DataFile_Format_Description.txt NotFound"#;
        println!("embedFile not found.");
    }
    //println!("{}",String::from_utf8_lossy(&Custom_Detail));  //debug

    let header_size: u32 =
        header_tag.len() as u32 + 4 + meta_bytes.len() as u32 + 1 + Custom_Detail.len() as u32 + 1;
    wfile
        .write_all(&header_size.to_le_bytes())
        .expect("写入失败");
    wfile.write_all(&meta_bytes).expect("写入失败");
    wfile.write_all(&[0]).expect("写入失败"); //写入一个 0 作为字符串结束.
    wfile.write_all(&Custom_Detail).expect("写入失败");
    wfile.write_all(&[0]).expect("写入失败"); //写入一个 0 作为字符串结束.

    //------ 写 Parameter_Table ----------
    param_table_total_size += 4; //加上total_size本身的4bytes后,为param_table_total_size,
    wfile
        .write_all(&param_table_total_size.to_le_bytes())
        .expect("写入失败");
    let mut data_point: u64 = (header_size + param_table_total_size) as u64;
    //   --- 写,单个参数的table ---
    for ii in 0..PRM_table.len() {
        PRM_table[ii].data_point = data_point; //修改,指向DATA的指针
        data_point += PRM_table[ii].data_size as u64; //加上data_size,指向下一个DATA
        let mut buf: Vec<u8> = vec![];
        PRM_table[ii]
            .serialize(&mut buf)
            .expect("PRM_table serialize失败");
        wfile.write_all(&buf).expect("写入失败");
    }
    //------ 写 DATA ---------
    for ii in 0..PRM_data.len() {
        wfile.write_all(&PRM_data[ii]).expect("写入失败");
    }

    println!();
    println!(" Decoded Parameters Count: {}.", ii_prm);
    println!(
        " All Parameters write to DATA file: \"{}\".",
        filename_write
    );
    println!("     Header size: {}", header_size);
    println!("     Parameter table size: {}", param_table_total_size);
    println!();
}
//列出prm配置中的所有参数名称
fn paramlist(prm_conf: prm_conf::PrmConf) {
    let mut ii = 0;
    for param in prm_conf.param.keys() {
        if ii > 0 && ii % 10 == 0 {
            println!();
        }
        print!("{param}\t");
        ii += 1;
    }
    println!();
}
//获取一个参数
fn get_param(
    buf: &Mmap,
    buflen: usize,
    prm_conf: &prm_conf::PrmConf,
    prm_superFrameCnt_prm: &prm_conf::Param,
    prm_superFrameCnt: &Vec<usize>,
    prm_name: &String,
    VERBOSE: i16,
) -> PrmValue {
    // 参数的配置,
    let prm_words; //参数的取值位置的配置
    let prm_superframe; //参数的超级帧配置
    let _res_rangeMin;
    let _res_rangeMax;
    let res_A: f32; //系数A
    let res_B: f32; //系数B
    let _res_C: f32; //系数C

    let prm_param = prm_conf
        .param
        .get(prm_name.as_str())
        .expect(format!("参数没找到:\"{}\"", prm_name).as_str());
    prm_words = prm_param.words.clone();
    //prm_superframe = prm_param.superframe;

    //取words中,第一组第一个值的superframe
    prm_superframe = prm_words[0][0];

    if prm_param.res.len() > 0 {
        [_res_rangeMin, _res_rangeMax, res_A, res_B, _res_C] = prm_param.res[0];
    } else {
        [_res_rangeMin, _res_rangeMax, res_A, res_B, _res_C] = [0.0, 0.0, 0.0, 1.0, 0.0];
    }

    // 参数的 每秒记录个数
    // 这个值，算的很粗糙，可能会不正确 !!!!!
    let param_rate: f32;
    if prm_words[0][1] == 0 {
        //subframe[0]==0
        if prm_words[0][0] == 0 {
            //superF[0]==0
            param_rate = prm_words.len() as f32; //一个subframe中记录的个数
        } else {
            //如果superframe不为0
            param_rate = (prm_words.len() as f32) / (prm_conf.SuperFramePerCycle as f32);
        }
    } else {
        //subframe[0] >0
        if prm_words[0][0] == 0 {
            //superF[0]==0
            //找出相同subframe有几个
            let subframe = prm_words[0][1];
            let mut num = 0;
            for vv in &prm_words {
                if vv[1] == subframe {
                    num += 1;
                }
            }
            param_rate = num as f32; //一个subframe中记录的个数
        } else {
            //如果superframe不为0
            param_rate = 1.0 / (prm_conf.SuperFramePerCycle as f32);
        }
    }
    //let param_rate: f32 = 16.0;

    let word_per_sec = prm_conf.WordPerSec;

    //变量初始化
    let mut subframe_cnt: i32 = 0; //subframe计数，
    let mut supcount_idx: i32 = 0; //超级帧索引, 0-15
    let mut word_cnt: usize = 0; //word计数，16bit字计数, (这个计数没什么用)
    let mut byte_cnt: usize = 0; //byte计数，字节计数。根据单/双数,也能确定word拼接时的位置。
    let mut frame_time: f32; //frame时间轴

    let mut ValType = "str"; //默认为str
    let mut PrmDict_f32: Vec<PrmDict<f32>> = vec![];
    let PrmDict_i32: Vec<PrmDict<i32>> = vec![];
    let mut PrmDict_str: Vec<PrmDict<String>> = vec![];

    //用于返回值 PrmValue.info
    let mut info_json: serde_json::Value =
        serde_json::from_str("{}").expect("serde_json::from_str失败");
    info_json["RecFormat"] = serde_json::Value::String(prm_param.RecFormat.clone());
    if prm_param.RecFormat == "DIS" {
        info_json["Options"] =
            serde_json::to_value(prm_param.Options.clone()).expect("serde_json,Options失败");
    }

    println!("--- begin: {prm_name} ---");
    let mut dword_err: u8 = 0x0; //用于记录get_dword_raw()的错误标记
    loop {
        if find_frame(
            &mut byte_cnt,
            &mut word_cnt,
            word_per_sec,
            buflen,
            &buf,
            VERBOSE,
        ) == false
        {
            if VERBOSE & 0x2 > 0 {
                if dword_err & 0x2 > 0 {
                    //通常 SuperFrameCounter , targetBit==0
                    println!("--> INFO.targetBit !=0, 取值结果可能不正确,SuperFrameCounter");
                }
                if dword_err & 0x1 > 0 {
                    //targetBit !=0 不知道如何拼接，暂时忽略这个配置。只给出提示信息。
                    println!("--> INFO.targetBit !=0, 取值结果可能不正确,{prm_name}");
                }
            }
            let info_bytes = serde_json::to_string(&info_json).expect("serde_json::to_string失败");
            //返回参数解码后的值
            match ValType {
                "float" => {
                    return PrmValue {
                        val: PrmType::Float(PrmDict_f32),
                        info: info_bytes,
                    }
                }
                "int" => {
                    return PrmValue {
                        val: PrmType::Int(PrmDict_i32),
                        info: info_bytes,
                    }
                }
                _ => {
                    return PrmValue {
                        val: PrmType::Str(PrmDict_str),
                        info: info_bytes,
                    }
                }
            }
        }
        /*
        //是否有丢帧
        if word_cnt / word_per_sec != subframe_cnt.try_into().unwrap() {
            println!("-> wordCnt/(word/sec):{:?}---subframeCnt:{:?}", word_cnt/word_per_sec, subframe_cnt);
        }
        */
        if let Some(val) = get_dword_raw(
            prm_superFrameCnt_prm,
            &prm_superFrameCnt,
            byte_cnt,
            word_per_sec,
            0, //subframe_idx,获取SuperFrameCount时设0
            &buf,
            &mut dword_err,
        ) {
            supcount_idx = val; //超级帧索引
            if VERBOSE & 0x80 > 0 && word_cnt < 128000 {
                println!("  --->超级帧索引:{}", supcount_idx);
            }
        }

        //超级帧判断
        if prm_superframe <= 0 || (prm_superframe as i32) == (supcount_idx + 1) {
            //按4个subframe循环
            for subframe_idx in 1..=4 {
                let mut rate_cnt: f32 = 0.0; //同一个subframe中,参数值的计数
                let mut dword_raw: i32;
                //按记录组循环. 单个记录组为一个完整的记录
                'SFrame: for prm_set in &prm_words {
                    match get_dword_raw(
                        prm_param,
                        &prm_set,
                        byte_cnt,
                        word_per_sec,
                        subframe_idx,
                        &buf,
                        &mut dword_err,
                    ) {
                        Some(val) => dword_raw = val,
                        None => {
                            //None 就是subframe不匹配，所以无需 rate_cnt += 1.0;
                            continue 'SFrame;
                        }
                    }
                    frame_time =
                        (subframe_cnt + (subframe_idx - 1) as i32) as f32 + (rate_cnt / param_rate);

                    // 处理BCD, 即,十进制数值
                    // 从get_dword_raw()中,移动到这里处理。
                    if "BCD" == prm_param.RecFormat {
                        ValType = "float";
                        let mut value_i32: i32 = 0;
                        let mut ii = 0;
                        //借用，倒序
                        for bcd_bits in (&prm_param.ConvConfig).iter().rev() {
                            let bits_mask: i32 = (1 << bcd_bits) - 1;
                            value_i32 += (bits_mask & dword_raw) * (10_i32.pow(ii));
                            dword_raw >>= bcd_bits;
                            ii += 1;
                        }
                        //BCD也有转换系数
                        let value_f32: f32 = (value_i32 as f32) * res_B + res_A; //通过系数，转换为工程值
                                                                                 //println!("=>BCD:{value_f32}");
                        PrmDict_f32.push(PrmDict {
                            t: frame_time,
                            v: value_f32,
                        });
                    } else if prm_param.RecFormat.starts_with("ISO")
                        || prm_param.RecFormat.starts_with("CHAR")
                    {
                        ValType = "str";
                        let mut value_str = "".to_string();
                        //借用，倒序
                        for bcd_bits in (&prm_param.ConvConfig).iter().rev() {
                            let bits_mask: i32 = (1 << bcd_bits) - 1;
                            //value_str.push(std::char::from_u32((bits_mask & dword_raw) as u32).unwrap());
                            value_str.push(((bits_mask & dword_raw) as u8) as char);
                            dword_raw >>= bcd_bits;
                        }
                        //println!("=>ISO:{value_str}");
                        PrmDict_str.push(PrmDict {
                            t: frame_time,
                            v: value_str,
                        });
                    } else if "UTC" == prm_param.RecFormat {
                        ValType = "str";
                        let ss = (dword_raw & 0x3f) as u8;
                        let mm = ((dword_raw >> 6) & 0x3f) as u8;
                        let hh = ((dword_raw >> 12) & 0x3f) as u8;
                        let value_str = format!("{hh:02}:{mm:02}:{ss:02}");
                        PrmDict_str.push(PrmDict {
                            t: frame_time,
                            v: value_str,
                        });
                    } else {
                        //处理BNR,DIS; PACKED_BITS,DISCRETE,
                        ValType = "float";
                        let value_f32: f32 = (dword_raw as f32) * res_B + res_A; //通过系数，转换为工程值
                        if VERBOSE & 0x80 > 0 && word_cnt < 128000 {
                            println!(
                                "subframe:{}, time:{:.5}, val:{:?}",
                                subframe_idx, frame_time, value_f32
                            );
                        }
                        PrmDict_f32.push(PrmDict {
                            t: frame_time,
                            v: value_f32,
                        });
                    }

                    //一个subframe, 仅一个记录组.就是一秒一记录
                    //一个subframe, 有多个记录组.就是一秒多记录
                    rate_cnt += 1.0;
                }
            }
        }
        byte_cnt += word_per_sec * 2 * 4;
        word_cnt += word_per_sec * 4 - 1;
        subframe_cnt += 4;
    }
}
//获取参数，一组位置的原始值
// --增加 param_prm 参数，可以获取RecFormat,ConvConfig， 为了处理BCD格式.
fn get_dword_raw(
    param_prm: &prm_conf::Param,
    prm_set: &Vec<usize>,
    byte_cnt: usize,
    word_per_sec: usize,
    subframe_idx: usize,
    buf: &Mmap,
    dword_err: &mut u8, //用于返回错误标记
) -> Option<i32> {
    let mut dword_raw: i32 = 0;
    let mut ttl_bit = 0; //总bit计数

    //保存第一个值的superframe
    //vec的超级帧参数，同组的值,分别取自不同的superframe位置,不同的subframe位置,TODO,
    let _superframe_idx = prm_set[0];
    //为了倒序循环,计算最后一组配置的值
    let mut ii: usize = (prm_set.len() / 6 - 1) * 6; //整数 乘除.
    loop {
        //倒序循环
        //配置中 是否 指定了 subframe
        //subframe_idx, prm_set[ii + 1] 取值为 0,1,2,3,4;
        if subframe_idx > 0 && prm_set[ii + 1] > 0 && prm_set[ii + 1] != subframe_idx {
            //普通参数,同组的subframe应该相同。 但超级帧参数,就会不同，TODO
            return None;
        }
        if prm_set[ii + 5] != 0 {
            //targetBit !=0 不知道如何拼接，暂时忽略这个配置。
            //    把错误标记返回，一个参数只给出提示信息一次。
            //println!("--> INFO.targetBit !=0, 取值结果可能不正确");
            if subframe_idx == 0 {
                //获取SuperFrameCounter时
                *dword_err |= 0x2;
            } else {
                //获取参数时
                *dword_err |= 0x1;
            }
        }
        let bits_cnt = prm_set[ii + 4] - prm_set[ii + 3] + 1;
        ttl_bit += bits_cnt; //总bit位数
        let bits_mask: i32 = (1 << bits_cnt) - 1;
        dword_raw <<= bits_cnt;

        let byte_pos: usize;
        if prm_set[ii + 1] > 0 {
            //subframe
            byte_pos =
                byte_cnt + (prm_set[ii + 1] - 1) * word_per_sec * 2 + (prm_set[ii + 2] - 1) * 2;
        } else {
            //subframe_idx==0,即:获取SuperFrameCounter时的subframe不会为0
            //反之,参数的subframe==0时，subframe_idx不会为0
            //即, 参数的subframe 与 subframe_idx 不会同时为0
            byte_pos = byte_cnt + (subframe_idx - 1) * word_per_sec * 2 + (prm_set[ii + 2] - 1) * 2;
        }
        dword_raw |= (((buf[byte_pos + 1] as i32) << 8 | buf[byte_pos] as i32)
            >> (prm_set[ii + 3] - 1))
            & bits_mask;

        if ii > 0 {
            ii -= 6; //step
        } else {
            break;
        }
    }
    //如果有符号位，并且，最高位为1 . 默认最高bit为符号位.
    //if param_prm.signRecType == true && dword_raw & (1 << (ttl_bit - 1)) > 0
    if param_prm.signed == true && dword_raw & (1 << (ttl_bit - 1)) > 0 {
        //计算补码
        dword_raw -= 1 << ttl_bit;
        //println!("--> INFO.signed=true, 计算补码");
    }
    // 处理BCD, 即,十进制数值
    // 原本在这里，现在移动到 get_param() 中处理。
    //if "BCD" == param_prm.RecFormat {
    //...
    //}
    return Some(dword_raw);
}
//寻找同步字,只找sync1: 0x247
//判断Frame长度完整,并判断后续三个同步字(0x5B8,A47,DB8)正确
fn find_frame(
    byte_cnt: &mut usize,
    word_cnt: &mut usize,
    word_per_sec: usize,
    buflen: usize,
    buf1: &Mmap,
    VERBOSE: i16,
) -> bool {
    let buf = buf1;
    let mut pre_word_cnt = *word_cnt; //保存上一个位置
    if *byte_cnt > 0 {
        //非文件头，需要+1
        pre_word_cnt += 1;
    }
    loop {
        if *byte_cnt >= buflen - 2 {
            //差一个字节到文件尾
            println!("文件结束。");
            return false;
        }
        if *byte_cnt > 0 && *byte_cnt & 0x1 == 0 {
            //是偶数
            *word_cnt += 1;
        }
        //两个字节拼为一个word, littleEndin, 低位在前。
        let word16: u16 = ((buf[*byte_cnt + 1] as u16) << 8 | (buf[*byte_cnt] as u16)) & 0x0fff;
        if word16 == 0x247 {
            if *byte_cnt + word_per_sec * 2 * 4 > buflen {
                //下一个Frame超出文件末尾
                if VERBOSE & 0x1 > 0 {
                    println!(
                        "->last Frame 不完整,放弃. last Frame len:0x{:X} bytes.",
                        buflen - (*byte_cnt),
                    );
                }
                return false; //Frame不完整,放弃
            }
            let diff_word_cnt = *word_cnt - pre_word_cnt; //word_cnt的差值

            //加一个subframe长度
            let mut byte_pos: usize = *byte_cnt + word_per_sec * 2;
            // word_per_sec 之后，是同步字2,0x5B8
            let word16_next: u16 =
                ((buf[byte_pos + 1] as u16) << 8 | buf[byte_pos] as u16) & 0x0fff;
            if word16_next != 0x5B8 {
                if VERBOSE & 0x1 > 0 {
                    println!(
                        "--->INFO.sync2 错误.0x{:X} wordCnt:0x{:X}, byteCnt0x{:X} 但next不是sync字, len:0x{:X}",
                        word16, *word_cnt, *byte_cnt, diff_word_cnt,
                        );
                }
                *byte_cnt += 1;
                continue;
            }
            //再加一个subframe长度
            byte_pos += word_per_sec * 2;
            // word_per_sec 之后，是同步字3,0xA47
            let word16_next: u16 =
                ((buf[byte_pos + 1] as u16) << 8 | buf[byte_pos] as u16) & 0x0fff;
            if word16_next != 0xA47 {
                if VERBOSE & 0x1 > 0 {
                    println!(
                        "--->INFO.sync3 错误.0x{:X} wordCnt:0x{:X}, byteCnt0x{:X} 但next不是sync字, len:0x{:X}",
                        word16, *word_cnt, *byte_cnt, diff_word_cnt,
                        );
                }
                *byte_cnt += 1;
                continue;
            }
            //再加一个subframe长度
            byte_pos += word_per_sec * 2;
            // word_per_sec 之后，是同步字4,0xDB8
            let word16_next: u16 =
                ((buf[byte_pos + 1] as u16) << 8 | buf[byte_pos] as u16) & 0x0fff;
            if word16_next != 0xDB8 {
                if VERBOSE & 0x1 > 0 {
                    println!(
                        "--->INFO.sync4 错误.0x{:X} wordCnt:0x{:X}, byteCnt0x{:X} 但next不是sync字, len:0x{:X}",
                        word16, *word_cnt, *byte_cnt, diff_word_cnt,
                        );
                }
                *byte_cnt += 1;
                continue;
            }

            if diff_word_cnt > 0 {
                if VERBOSE & 0x1 > 0 {
                    if pre_word_cnt == 0 && *byte_cnt > 1 {
                        println!(
                            "--->!!!Warning!!! First SYNC 0x{:X} at 0x{:X}b 0x{:X}w, not beginning of DATA.",
                            word16, *byte_cnt, *word_cnt,
                            );
                    }
                }
                if VERBOSE & 0x20 > 0 {
                    println!(
                        "--->INFO.找到sync字.0x{:X} wordCnt:0x{:X}, len:0x{:X}",
                        word16, *word_cnt, diff_word_cnt
                    );
                }
                if *byte_cnt & 0x1 != 0 {
                    println!("--->INFO.word 错位一个 byte.",);
                }
            } else {
                if VERBOSE & 0x80 > 0 && *word_cnt < 128000 {
                    //超过这个值，就不再打印
                    println!("--->找到sync1字.0x{:X} wordCnt:0x{:X}", word16, *word_cnt);
                }
            }

            /*
            //再加一个subframe长度
            byte_pos += word_per_sec * 2;
            if byte_pos >= buflen {
                //下一个Frame是文件末尾
                if VERBOSE & 0x1 > 0 {
                    println!(
                        "->找到last Frame. wordCnt:0x{:X}---word:0x{:X}",
                        *word_cnt, word16
                    );
                }
                //return true;
            }
            */

            return true;
        }
        *byte_cnt += 1;
    }
}
fn showHelp(bin_name: String) {
    println!("Usage: {bin_name} [-r raw.dat] [-w out.csv] [-a|-p GS] [-h | --help]");
    println!("   Detail:");
    println!("      -h        简略的命令行帮助");
    println!("      --show    显示 自定义格式文件的格式说明");
    println!("      -j, --json  /path/prm.json   指定json配置文件路径");
    println!("      -l, --paramlist          列出配置中所有的的参数名");
    println!("      -a, --all                解码所有的参数名");
    println!("      -p, --param  VRTG        解码一个参数名");
    println!("      -r /path/raw.dat         指定读取raw原始文件");
    println!("      -w /path/out.csv     如有 -p, 解码单个参数,写入csv文件");
    println!("      -w /path/out.dat     如有 -a, 解码所有参数,写入自定义格式的dat文件");
    println!(
        "           自定义格式的out.dat文件,可以用ALL_read_datafile.py读取,并导入pd.DataFrame()"
    );
    println!("      --mem        打印内存占用情况");
    println!(" 说明: ");
    println!("   使用mmap把raw文件映射到内存，然后再解码参数。");
    println!(
        "     解码过程,按照Frame为单位循环获取参数的值.如果参数内容来自不同subframe,也能处理。"
    );
    println!("   解码配置来自json文件。");
    println!("       通过 read_prm717.py    把PRM配置转换为json格式。");
    println!("       通过 VEC717_to_json.py 把VEC配置转换为json格式。");
    println!("   所有的解码配置集中在 prm_conf::PrmConf 中。");
    println!("   增加命令行参数，显示内存占用，增加同步字顺序警告。");
    println!("   增加subframe判断，处理了符号位。");
    println!("   更改使用hashmap保存配置。配置中增加targetBit。");
    println!("   增加superFrame配置。尝试解码超级帧参数。");
    println!("   可以处理BNR,DIS,BCD,ISO格式的参数。");
    println!("      author: osnosn@126.com");
}
