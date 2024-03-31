#![allow(non_snake_case)]
//! 使用mmap把raw文件映射到内存，然后再解码多个参数中的一个。  
//! 多个解码配置写死在程序中。一次只能解码一个参数。  
//! 增加命令行参数，显示内存占用，增加同步字顺序警告。  
//! 增加subframe判断，处理了符号位。  
//! 更改使用hashmap保存配置。配置中增加targetBit。  
//! 增加superFrame配置。尝试解码超级帧参数。  
//!    author: osnosn@126.com  OR  LLGZ@csair.com  

use memmap2::Mmap;
use memmap2::MmapOptions;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;

#[path = "../CmdLineArgs_aligned4.rs"]
mod CmdLineArgs_aligned;

#[path = "../prm_conf5.rs"]
mod prm_conf;

fn main() {
    let args;
    match CmdLineArgs_aligned::parse_args() {
        Ok(tmp) => args = tmp,
        _ => {
            showHelp();
            return ();
        }
    }
    if args.help || args.help2 {
        showHelp();
        return ();
    }

    // 读取的文件名
    let filename_read = args.rawfile.as_str();
    // 写入的文件名
    let filename_write = args.csvfile.as_str();
    // 打开数据文件
    let rfile: File = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(filename_read)
        .expect(format!("无法打开文件:\"{}\"", filename_read).as_str());
    println!("文件打开成功：{:?}", filename_read);

    // 使用mmap映射
    let buf = unsafe { MmapOptions::new().map(&rfile).expect("mmap映射创建失败") };
    let buflen = buf.len();

    // 创建一个文件，用于写入csv格式的数据
    let mut wfile = File::create(filename_write)
        .expect(format!("创建文件失败:\"{}\"", filename_write).as_str());
    wfile
        .write_all("time,value,gmt_time\r\n".as_bytes())
        .expect("写入失败");

    //参数配置,创建
    let prm_conf = prm_conf::PrmConf::new();

    // 参数的配置,
    let prm_words; //取值位置的配置
    let prm_supframe; //超级帧
    let res_A; //系数A
    let res_B; //系数B
    let signed; //是否带符号位 0.0=N, 1.0=Y,
    let prm_name; //参数名称
    prm_name = match &args.cmd as &str {
        "" | "1" => "VRTG",
        "2" => "ALTSTD",
        "3" => "GS3",
        "4" => "PITCH",
        "5" => "N11",
        "6" => "N21",
        "7" => "SAT",
        "8" => "AILERON",
        "h" => "GMTH",
        "m" => "GMTM",
        "s" => "GMTS",
        "sup" => "SUP_COUNTER",
        "day" => {
            println!("这个参数是BCD编码，本程序未实现，取值正确，但转换后的工程值不正确");
            "DAY"
        }
        _ => {
            showHelp();
            return ();
        }
    };
    prm_words = prm_conf
        .get(prm_name)
        .expect(format!("参数没找到:\"{}\"", prm_name).as_str())
        .words
        .clone();
    prm_supframe = prm_conf
        .get(prm_name)
        .expect(format!("参数没找到:\"{}\"", prm_name).as_str())
        .supframe;
    [signed, res_A, res_B] = prm_conf
        .get(prm_name)
        .expect(format!("参数没找到:\"{}\"", prm_name).as_str())
        .res;

    //每次都要取值的参数配置
    let sup_counter = prm_conf
        .get("SUP_COUNTER")
        .expect("参数没找到:'SUP_COUNTER'")
        .words[0]
        .clone();
    let frame_hour = prm_conf.get("GMTH").expect("参数没找到:'GMTH'").words[0].clone();
    let frame_min = prm_conf.get("GMTM").expect("参数没找到:'GMTM'").words[0].clone();
    let frame_sec = prm_conf.get("GMTS").expect("参数没找到:'GMTS'").words[0].clone();

    // 参数的 每秒记录个数
    // 这个值，算的很粗糙，可能会不正确 !!!!!
    let param_rate: f32;
    if prm_words[0][0] == 0 {
        param_rate = prm_words.len() as f32;
    } else {
        param_rate = 1.0;
    }
    //let param_rate: f32 = 16.0;

    //变量初始化
    let mut subframe_cnt: i32 = 0; //subframe计数，
    let mut subframe_idx: usize = 1; //subframe索引, 1-4
    let mut supcount_idx: i32 = 0; //超级帧索引, 0-15
    let mut word_cnt: usize = 0; //word计数，16bit字计数, (这个计数没什么用)
    let mut byte_cnt: usize = 0; //byte计数，字节计数。根据单/双数,也能确定word拼接时的位置。
    let mut value: f32; //解码后的工程值
    let mut frame_time: f32; //frame时间轴
    let mut frame_time_string: String; //frame时间
    let word_per_sec = 1024;
    println!("Hexadecimal representation 十六进制表示:");
    loop {
        if find_sync(
            &mut byte_cnt,
            &mut word_cnt,
            word_per_sec,
            &mut subframe_idx,
            buflen,
            &buf,
        ) == false
        {
            break;
        }
        /*
        //是否有丢帧
        if word_cnt / word_per_sec != subframe_cnt.try_into().unwrap() {
            println!("-> wordCnt/(word/sec):{:?}---subframeCnt:{:?}", word_cnt/word_per_sec, subframe_cnt);
        }
        */
        if let Some(val) = get_dword_raw(&sup_counter, 0.0, byte_cnt, subframe_idx, &buf) {
            supcount_idx = val; //超级帧索引
            if word_cnt < 128000 {
                println!("  --->超级帧索引:{}", supcount_idx);
            }
        }

        //超级帧判断
        if prm_supframe <= 0 || (prm_supframe as i32) == (supcount_idx + 1) {
            //取GMT时间，H:M:S
            frame_time_string = String::from("");
            if let Some(val) = get_dword_raw(&frame_hour, 0.0, byte_cnt, subframe_idx, &buf) {
                frame_time_string = format!("{}:", val);
            }
            if let Some(val) = get_dword_raw(&frame_min, 0.0, byte_cnt, subframe_idx, &buf) {
                frame_time_string.push_str(format!("{}:", val).as_str());
            }
            if let Some(val) = get_dword_raw(&frame_sec, 0.0, byte_cnt, subframe_idx, &buf) {
                frame_time_string.push_str(format!("{}", val).as_str());
            }

            let mut rate_cnt: f32 = 0.0;
            let mut dword_raw: i32;
            'SFrame: for prm_set in &prm_words {
                match get_dword_raw(prm_set, signed, byte_cnt, subframe_idx, &buf) {
                    Some(val) => dword_raw = val,
                    None => {
                        //None 就是subframe不匹配，所以无需 rate_cnt += 1.0;
                        continue 'SFrame;
                    }
                }
                value = (dword_raw as f32) * res_B + res_A; //通过系数，转换为工程值
                frame_time = (subframe_cnt as f32) + (rate_cnt / param_rate);
                if word_cnt < 128000 {
                    println!(
                        "subframe:{}, time:{:.5}, val:{:?}, GMT:{}",
                        subframe_idx, frame_time, value, frame_time_string
                    );
                }

                // 以csv格式写入文件
                wfile
                    .write_all(
                        format!("{:.5},{:?},{}\r\n", frame_time, value, frame_time_string)
                            .as_bytes(),
                    )
                    .expect("写入失败");
                //一个subframe只有一个记录，输出一次即可
                frame_time_string = String::from(""); //输出一次后，就清除,
                rate_cnt += 1.0;
            }
        }
        byte_cnt += word_per_sec * 2;
        word_cnt += word_per_sec - 1;
        subframe_cnt += 1;
    }
    println!("");
    println!(" The length of data is {}.", buflen);
    println!(
        " Parameter \"{}\" write to CSV file: \"{}\".",
        prm_name, filename_write
    );
    println!("");

    #[cfg(target_os = "linux")]
    if args.mem {
        // --begin--查看内存占用(linux)
        use std::io::{BufRead, BufReader};
        use std::process;
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
        println!(" Windows 不支持 --mem 参数。");
        println!("   因为 Windows 不支持 /proc/pid/status 文件的查看。");
    }
}
//获取参数，一组位置的原始值
fn get_dword_raw(
    prm_set: &Vec<usize>,
    signed: f32,
    byte_cnt: usize,
    subframe_idx: usize,
    buf: &Mmap,
) -> Option<i32> {
    let mut dword_raw: i32 = 0;
    let mut ttl_bit = 0; //总bit计数
                         //为了倒序循环,计算最后一组配置的值
    let mut ii: usize = (prm_set.len() / 5 - 1) * 5; //整数 乘除.
    loop {
        //倒序循环
        //配置中 是否 指定了 subframe
        if prm_set[ii] > 0 && prm_set[ii] != subframe_idx {
            return None;
        }
        if prm_set[ii + 4] != 0 {
            //targetBit !=0 不知道如何拼接，暂时忽略这个配置。只给出提示信息。
            println!("--> INFO.targetBit !=0, 取值结果可能不正确");
        }
        let bits_cnt = prm_set[ii + 3] - prm_set[ii + 2] + 1;
        ttl_bit += bits_cnt; //总bit位数
        let bits_mask: i32 = (1 << bits_cnt) - 1;
        dword_raw <<= bits_cnt;
        dword_raw |= (((buf[byte_cnt + (prm_set[ii + 1] - 1) * 2 + 1] as i32) << 8
            | buf[byte_cnt + (prm_set[ii + 1] - 1) * 2] as i32)
            >> (prm_set[ii + 2] - 1))
            & bits_mask;
        if ii > 0 {
            ii -= 5; //step
        } else {
            break;
        }
    }
    //如果有符号位，并且，最高位为1 . 默认最高bit为符号位.
    if signed > 0.0 && dword_raw & (1 << (ttl_bit - 1)) > 0 {
        //计算补码
        dword_raw -= 1 << ttl_bit;
    }
    return Some(dword_raw);
}
//寻找同步字
fn find_sync(
    byte_cnt: &mut usize,
    word_cnt: &mut usize,
    word_per_sec: usize,
    subframe_idx: &mut usize,
    buflen: usize,
    buf1: &Mmap,
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
            /*
            if word_cnt > 21139454 {
                println!("---文件已经扫描, 结束---");
                return false;
            }
            */
        }
        //两个字节拼为一个word, littleEndin, 低位在前。
        let word16: u16 = ((buf[*byte_cnt + 1] as u16) << 8 | (buf[*byte_cnt] as u16)) & 0x0fff;
        if word16 == 0x247 || word16 == 0x5B8 || word16 == 0xA47 || word16 == 0xDB8 {
            match word16 {
                0x247 => *subframe_idx = 1,
                0x5B8 => *subframe_idx = 2,
                0xA47 => *subframe_idx = 3,
                0xDB8 => *subframe_idx = 4,
                _ => (),
            }
            if *byte_cnt + word_per_sec * 2 >= buflen - 2 {
                //下一个sync是文件末尾
                println!(
                    "->找到last sync字. wordCnt:0x{:X}---word:0x{:X}",
                    *word_cnt, word16
                );
                return true;
            }
            // word_per_sec 之后，也是同步字
            let word16_next: u16 = ((buf[*byte_cnt + word_per_sec * 2 + 1] as u16) << 8
                | buf[*byte_cnt + word_per_sec * 2] as u16)
                & 0x0fff;
            let diff_word_cnt = *word_cnt - pre_word_cnt; //word_cnt的差值
            if word16_next == 0x247
                || word16_next == 0x5B8
                || word16_next == 0xA47
                || word16_next == 0xDB8
            {
                if diff_word_cnt > 0 {
                    println!(
                        "--->INFO.找到sync字.0x{:X} wordCnt:0x{:X}, len:0x{:X}",
                        word16, *word_cnt, diff_word_cnt
                    );
                    if *byte_cnt & 0x1 != 0 {
                        println!("--->INFO.word 错位一个 byte.",);
                    }
                } else {
                    if *word_cnt < 128000 {
                        //超过这个值，就不再打印
                        println!("--->找到sync字.0x{:X} wordCnt:0x{:X}", word16, *word_cnt);
                    }
                }
                if (*subframe_idx == 1 && word16_next != 0x5B8)
                    || (*subframe_idx == 2 && word16_next != 0xA47)
                    || (*subframe_idx == 3 && word16_next != 0xDB8)
                    || (*subframe_idx == 4 && word16_next != 0x247)
                {
                    println!(
                        "--->INFO.当前sync字.0x{:0X} wordCnt:0x{:X},NEXT.0x{:X},sync错误",
                        word16, *word_cnt, word16_next
                    );
                }
                return true;
            } else {
                println!(
                    "--->INFO.找到sync字.0x{:X} wordCnt:0x{:X}, 但next不是sync字, len:0x{:X}",
                    word16, *word_cnt, diff_word_cnt
                );
            }
        }
        *byte_cnt += 1;
    }
}
fn showHelp() {
    println!(
        "Usage: qar_decode [-r data/raw.dat] [-w data/output_data.csv] [-h | --help] [1|2|3|4|5|6|7|8|h|m|s|sup|day]"
    );
    println!("   Detail:");
    println!("      -h        简略的命令行帮助");
    println!("      -r /path/raw.dat   指定读取raw原始文件");
    println!("      -w /path/xxxx.csv  指定写入csv文件");
    println!("      --mem        打印内存占用情况");
    println!(" 说明: ");
    println!("   使用mmap把raw文件映射到内存，然后再解码多个参数中的一个。");
    println!("   多个解码配置写死在程序中。一次只能解码一个参数。");
    println!("   增加命令行参数，显示内存占用，增加同步字顺序警告。");
    println!("   增加subframe判断，处理了符号位。");
    println!("   更改使用hashmap保存配置。配置中增加targetBit。");
    println!("   增加superFrame配置。尝试解码超级帧参数。");
    println!("      author: osnosn@126.com  OR  LLGZ@csair.com");
}
