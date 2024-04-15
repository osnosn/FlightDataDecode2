#![allow(non_snake_case)]
//! 使用mmap把raw文件映射到内存，然后再解码多个参数中的一个。  
//! 多个解码配置写死在程序中。一次只能解码一个参数。  
//! 增加命令行参数，显示内存占用，增加同步字顺序警告。  
//! 增加subframe判断，处理了符号位。  
//!    author: osnosn@126.com   

use memmap2::Mmap;
use memmap2::MmapOptions;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;

#[path = "../CmdLineArgs4.rs"]
mod CmdLineArgs;

fn main() {
    let args;
    match CmdLineArgs::parse_args() {
        Ok(tmp) => args = tmp,
        Err(err) => {
            println!("Command line parse ERR.\r\n{err}");
            return ();
        }
    }
    if args.help || args.help2 {
        showHelp(args.bin_name);
        return ();
    }

    // 读取的文件名
    let filename_read;
    if args.rawfile.len() < 2 {
        filename_read = "data/raw.dat";
    } else {
        filename_read = args.rawfile.as_str();
    }
    // 写入的文件名
    let filename_write = args.csvfile.as_str();
    // 打开数据文件
    let rfile: File = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(filename_read)
        .expect("无法打开文件");
    println!("文件打开成功：{:?}", filename_read);

    // 使用mmap映射
    let buf = unsafe { MmapOptions::new().map(&rfile).expect("mmap映射创建失败") };
    let buflen = buf.len();

    // 创建一个文件，用于写入csv格式的数据
    let mut wfile = File::create(filename_write).expect("创建文件失败");
    wfile
        .write_all("time,value\r\n".as_bytes())
        .expect("写入失败");

    // vrtg 的配置
    let vrtg_words = vec![
        // [ subframe,word,lsb,msb,],
        vec![0, 2, 1, 12],
        vec![0, 34, 1, 12],
        vec![0, 66, 1, 12],
        vec![0, 98, 1, 12],
        vec![0, 130, 1, 12],
        vec![0, 162, 1, 12],
        vec![0, 194, 1, 12],
        vec![0, 226, 1, 12],
        vec![0, 258, 1, 12],
        vec![0, 290, 1, 12],
        vec![0, 322, 1, 12],
        vec![0, 354, 1, 12],
        vec![0, 386, 1, 12],
        vec![0, 418, 1, 12],
        vec![0, 450, 1, 12],
        vec![0, 482, 1, 12],
    ];
    //signed, resA, resB
    let vrtg_res = [0.0, -3.37538, 0.00228938];
    // altstd 的配置
    let altstd_words = vec![
        // [ subframe,word,lsb,msb,],
        vec![0, 47, 3, 11, 0, 46, 5, 12],
        vec![0, 175, 3, 11, 0, 174, 5, 12],
        vec![0, 303, 3, 11, 0, 302, 5, 12],
        vec![0, 431, 3, 11, 0, 430, 5, 12],
    ];
    //signed, resA, resB
    let altstd_res = [1.0, 0.0, 1.0];
    // gs3 的配置
    let gs3_words = vec![
        vec![0, 49, 2, 12],
        vec![0, 177, 2, 12],
        vec![0, 305, 2, 12],
        vec![0, 433, 2, 12],
    ];
    let gs3_res = [0.0, 0.0, 0.5];
    // pitch 的配置
    let pitch_words = vec![
        vec![0, 3, 3, 12],
        vec![0, 131, 3, 12],
        vec![0, 259, 3, 12],
        vec![0, 387, 3, 12],
    ];
    let pitch_res = [1.0, 0.0, 0.1757813];
    // N11 的配置
    let n11_words = vec![vec![0, 110, 3, 12]];
    let n11_res = [0.0, 0.0, 0.125];
    // N21 的配置
    let n21_words = vec![vec![0, 251, 3, 12]];
    let n21_res = [0.0, 0.0, 0.125];
    // SAT 的配置
    let sat_words = vec![vec![3, 249, 3, 12]];
    let sat_res = [1.0, 0.0, 0.25];
    // AILACTL 的配置
    let aileron_words = vec![
        vec![0, 82, 3, 12],
        vec![0, 210, 3, 12],
        vec![0, 338, 3, 12],
        vec![0, 466, 3, 12],
    ];
    let aileron_res = [1.0, 0.0, 0.03756054];
    // GMT_HOUR 的配置
    let gmth_words = vec![vec![1, 256, 8, 12]];
    let gmth_res = [0.0, 0.0, 1.0];
    // GMT_MIN 的配置
    let gmtm_words = vec![vec![1, 256, 2, 7]];
    let gmtm_res = [0.0, 0.0, 1.0];
    // GMT_SEC 的配置
    let gmts_words = vec![vec![1, 257, 1, 6]];
    let gmts_res = [0.0, 0.0, 1.0];

    // 参数的配置,
    let prm_words; //取值位置的配置
    let res_A; //系数A
    let res_B; //系数B
    let signed; //是否带符号位 0.0=N, 1.0=Y,
    let prm_name; //参数名称
    match &args.cmd as &str {
        "" | "1" => {
            prm_name = "VRTG";
            prm_words = vrtg_words;
            [signed, res_A, res_B] = vrtg_res;
        }
        "2" => {
            prm_name = "ALTSTD";
            prm_words = altstd_words;
            [signed, res_A, res_B] = altstd_res;
        }
        "3" => {
            prm_name = "GS3";
            prm_words = gs3_words;
            [signed, res_A, res_B] = gs3_res;
        }
        "4" => {
            prm_name = "PITCH";
            prm_words = pitch_words;
            [signed, res_A, res_B] = pitch_res;
        }
        "5" => {
            prm_name = "N11";
            prm_words = n11_words;
            [signed, res_A, res_B] = n11_res;
        }
        "6" => {
            prm_name = "N21";
            prm_words = n21_words;
            [signed, res_A, res_B] = n21_res;
        }
        "7" => {
            prm_name = "SAT";
            prm_words = sat_words;
            [signed, res_A, res_B] = sat_res;
        }
        "8" => {
            prm_name = "AILERON_ACTUATOR_POSN_LT";
            prm_words = aileron_words;
            [signed, res_A, res_B] = aileron_res;
        }
        "h" => {
            prm_name = "GMT HOUR";
            prm_words = gmth_words;
            [signed, res_A, res_B] = gmth_res;
        }
        "m" => {
            prm_name = "GMT MIN";
            prm_words = gmtm_words;
            [signed, res_A, res_B] = gmtm_res;
        }
        "s" => {
            prm_name = "GMT SEC";
            prm_words = gmts_words;
            [signed, res_A, res_B] = gmts_res;
        }
        _ => {
            showHelp(args.bin_name);
            return ();
        }
    };

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
    let mut word_cnt: usize = 0; //word计数，16bit字计数, (这个计数没什么用)
    let mut byte_cnt: usize = 0; //byte计数，字节计数。根据单/双数,也能确定word拼接时的位置。
    let mut value: f32; //解码后的工程值
    let mut frame_time: f32; //frame时间轴
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
        let mut rate_cnt: f32 = 0.0;
        let mut dword_raw: i32;
        'SFrame: for prm_set in &prm_words {
            dword_raw = 0;
            let mut sign_bit = 0; //符号位,如果有的话

            //为了倒序循环,计算最后一组配置的值
            let mut ii: usize = (prm_set.len() / 4 - 1) * 4; //整数 乘除.
            loop {
                //倒序循环
                //配置中 是否 指定了 subframe
                if prm_set[ii] > 0 && prm_set[ii] != subframe_idx {
                    //None 就是subframe不匹配，所以无需 rate_cnt += 1.0;
                    continue 'SFrame;
                }
                let bits_cnt = prm_set[ii + 3] - prm_set[ii + 2] + 1;
                //所有bit的最高位,假设为 符号位
                sign_bit += bits_cnt;
                let bits_mask: i32 = (1 << bits_cnt) - 1;
                dword_raw <<= bits_cnt;
                dword_raw |= (((buf[byte_cnt + (prm_set[ii + 1] - 1) * 2 + 1] as i32) << 8
                    | buf[byte_cnt + (prm_set[ii + 1] - 1) * 2] as i32)
                    >> (prm_set[ii + 2] - 1))
                    & bits_mask;
                if ii > 0 {
                    ii -= 4; //step
                } else {
                    break;
                }
            }
            frame_time = (subframe_cnt as f32) + (rate_cnt / param_rate);
            //如果有符号位，并且，最高位为1
            if signed > 0.0 && dword_raw & (1 << (sign_bit - 1)) > 0 {
                //计算补码
                dword_raw -= 1 << sign_bit;
            }
            value = (dword_raw as f32) * res_B + res_A;
            if word_cnt < 128000 {
                println!(
                    "subframe:{}, time:{:.5}, val:{:?}",
                    subframe_idx, frame_time, value
                );
            }

            // 以csv格式写入文件
            wfile
                .write_all(format!("{:.5},{:?}\r\n", frame_time, value).as_bytes())
                .expect("写入失败");
            rate_cnt += 1.0;
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
fn showHelp(bin_name: String) {
    println!(
        "Usage: {bin_name} [-r data/raw.dat] [-w data/output_data.csv] [-h | --help] [1|2|3|4|5|6|7|8|h|m|s]"
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
    println!("      author: osnosn@126.com");
}
