#![allow(non_snake_case)]
//! 使用mmap把raw文件映射到内存，然后再解码多个参数中的一个。  
//! 多个解码配置写死在程序中。一次只能解码一个参数。  
//! 使用mmap读取raw.dat。  
//!    author: osnosn@126.com  OR  LLGZ@csair.com  

use memmap2::Mmap;
use memmap2::MmapOptions;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
//use std::process;

#[path = "../CmdLineArgs.rs"]
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
    let filename_read = "data/raw.dat";
    // 写入的文件名
    let filename_write = "data/output_data.csv";
    // 打开数据文件
    //let rfile: File = File::open(filename_read).expect("无法打开文件");
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

    let mut subframe_cnt: i32 = 0; //subframe计数，
    let mut word_cnt: usize = 0; //word计数，16bit字计数, (这个计数没什么用)
    let mut byte_cnt: usize = 0; //byte计数，字节计数。根据单/双数,也能确定word拼接时的位置。
    let mut value: f32; //解码后的工程值
    let mut frame_time: f32; //frame时间轴
    let word_per_sec = 1024;
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
    let vrtg_res = [-3.37538, 0.00228938];
    // altstd 的配置
    let altstd_words = vec![
        // [ subframe,word,lsb,msb,],
        vec![0, 47, 3, 11, 0, 46, 5, 12],
        vec![0, 175, 3, 11, 0, 174, 5, 12],
        vec![0, 303, 3, 11, 0, 302, 5, 12],
        vec![0, 431, 3, 11, 0, 430, 5, 12],
    ];
    let altstd_res = [0.0, 1.0];
    // gs3 的配置
    let gs3_words = vec![
        vec![0, 49, 2, 12],
        vec![0, 177, 2, 12],
        vec![0, 305, 2, 12],
        vec![0, 433, 2, 12],
    ];
    let gs3_res = [0.0, 0.5];
    // pitch 的配置
    let pitch_words = vec![
        vec![0, 3, 3, 12],
        vec![0, 131, 3, 12],
        vec![0, 259, 3, 12],
        vec![0, 387, 3, 12],
    ];
    let pitch_res = [0.0, 0.1757813];
    // N11 的配置
    let n11_words = vec![vec![0, 110, 3, 12]];
    let n11_res = [0.0, 0.125];
    // N21 的配置
    let n21_words = vec![vec![0, 251, 3, 12]];
    let n21_res = [0.0, 0.125];

    // 参数的配置,
    let prm_words;
    let res_A;
    let res_B;
    match &args.cmd as &str {
        "" | "1" => {
            prm_words = vrtg_words;
            res_A = vrtg_res[0];
            res_B = vrtg_res[1];
        }
        "2" => {
            prm_words = altstd_words;
            res_A = altstd_res[0];
            res_B = altstd_res[1];
        }
        "3" => {
            prm_words = gs3_words;
            res_A = gs3_res[0];
            res_B = gs3_res[1];
        }
        "4" => {
            prm_words = pitch_words;
            res_A = pitch_res[0];
            res_B = pitch_res[1];
        }
        "5" => {
            prm_words = n11_words;
            res_A = n11_res[0];
            res_B = n11_res[1];
        }
        "6" => {
            prm_words = n21_words;
            res_A = n21_res[0];
            res_B = n21_res[1];
        }
        _ => {
            showHelp(args.bin_name);
            return ();
        }
    };

    // 参数的 每秒记录个数
    let param_rate: f32 = prm_words.len() as f32;
    //let param_rate: f32 = 16.0;

    println!("Hexadecimal representation:");
    loop {
        if find_sync(&mut byte_cnt, &mut word_cnt, word_per_sec, buflen, &buf) == false {
            //process::exit(1);  //非正常退出.(未使用)
            break;
        }
        /*
        if word_cnt / word_per_sec != subframe_cnt.try_into().unwrap() {
            println!("-> wordCnt/(word/sec):{:?}---subframeCnt:{:?}", word_cnt/word_per_sec, subframe_cnt);
        }
        */
        let mut rate_cnt: f32 = 0.0;
        let mut test_16: u16;
        for prm_set in &prm_words {
            test_16 = 0;
            let mut ii: usize = (prm_set.len() / 4 - 1) * 4; //整数 乘除
            loop {
                //倒序循环
                let bits_cnt = prm_set[ii + 3] - prm_set[ii + 2] + 1;
                let bits_mask: u16 = (1 << bits_cnt) - 1;
                test_16 <<= bits_cnt;
                test_16 |= (((buf[byte_cnt + (prm_set[ii + 1] - 1) * 2 + 1] as u16) << 8
                    | buf[byte_cnt + (prm_set[ii + 1] - 1) * 2] as u16)
                    >> (prm_set[ii + 2] - 1))
                    & bits_mask;
                if ii > 0 {
                    ii -= 4; //step
                } else {
                    break;
                }
            }
            frame_time = (subframe_cnt as f32) + (rate_cnt / param_rate);
            value = (test_16 as f32) * res_B + res_A;
            if word_cnt < 128000 {
                println!("subframe:{:.5}, val:{:?}", frame_time, value);
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
    println!(" Write CSV to file: \"{}\".", filename_write);
    println!("");
    /*
    // 查看内存占用(linux)
    use std::fs::read_to_string;
    println!(" PID is {}.", process::id());
    println!(" {}.", read_to_string(format!("/proc/{}/status",process::id())).expect("读取/proc/?/status失败"));
    */
}
fn find_sync(
    byte_cnt: &mut usize,
    word_cnt: &mut usize,
    word_per_sec: usize,
    buflen: usize,
    buf1: &Mmap,
) -> bool {
    let buf = buf1;
    loop {
        if *byte_cnt >= buflen - 2 {
            //差一个字节到文件尾
            println!("文件结束。");
            return false;
        }
        if *byte_cnt > 0 && *byte_cnt & 0x1 == 0 {
            //是偶数
            *word_cnt += 1;
            // if word_cnt > 21139454{
            //     println!("---文件已经扫描, 结束---" );
            //     return false;
            // }
        }
        //两个字节拼为一个word, littleEndin, 低位在前。
        let test_16: u16 = ((buf[*byte_cnt + 1] as u16) << 8 | (buf[*byte_cnt] as u16)) & 0x0fff;
        if test_16 == 0x247 || test_16 == 0x5B8 || test_16 == 0xA47 || test_16 == 0xDB8 {
            if *byte_cnt + word_per_sec * 2 >= buflen - 2 {
                //下一个sync是文件末尾
                println!(
                    "->找到last sync字. wordCnt:{:?}---word:{:0x}",
                    *word_cnt, test_16
                );
                return true;
            }
            // word_per_sec 之后，也是同步字
            let test_16_next: u16 = ((buf[*byte_cnt + word_per_sec * 2 + 1] as u16) << 8
                | buf[*byte_cnt + word_per_sec * 2] as u16)
                & 0x0fff;
            if test_16_next == 0x247
                || test_16_next == 0x5B8
                || test_16_next == 0xA47
                || test_16_next == 0xDB8
            {
                if *word_cnt < 128000 {
                    println!(
                        "->找到sync字. wordCnt:{:?}---word:{:0x}",
                        *word_cnt, test_16
                    );
                }
                return true;
            }
        }
        *byte_cnt += 1;
    }
}
fn showHelp(bin_name: String) {
    println!("Usage: {bin_name} [1|2|3|4|5|6] [-h | --help]");
    println!("   Detail:");
    println!("      -h        简略的命令行帮助");
    println!(" 说明: ");
    println!("   使用mmap把raw文件映射到内存，然后再解码多个参数中的一个。");
    println!("   多个解码配置写死在程序中。一次只能解码一个参数。");
    println!("   使用mmap读取raw.dat。");
    println!("      author: osnosn@126.com  OR  LLGZ@csair.com");
}
