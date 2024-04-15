#![allow(non_snake_case)]
//! 把raw文件全部读入内存，然后再解码一个参数。  
//! 一个解码配置写死在程序中。只能解码一个参数。  
//!    author: osnosn@126.com  

use std::fs::File;
use std::io::{BufReader, Read, Write};
//use std::process;

fn main() {
    // 读取的文件名
    let filename_read = "data/raw.dat";
    // 写入的文件名
    let filename_write = "data/output_data.csv";
    // 打开数据文件
    // let rfile: File = File::open("D:/HBH_QAR/decode/B-1446_20231212024257.wgl/raw.dat").expect("无法打开文件");
    let rfile: File = File::open(filename_read).expect("无法打开文件");
    println!("文件打开成功：{:?}", filename_read);

    // 使用缓冲读取器读取文件内容
    let mut reader: BufReader<File> = BufReader::new(rfile);

    // 读取文件内容到一个缓冲区
    let mut buf: Vec<u8> = Vec::new();
    let buflen = reader.read_to_end(&mut buf).expect("读取文件失败");

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
    let vrtg_words = [
        2, 34, 66, 98, 130, 162, 194, 226, 258, 290, 322, 354, 386, 418, 450, 482,
    ];
    let prm_words = vrtg_words;
    // 参数的 每秒记录个数
    let param_rate: f32 = prm_words.len() as f32;
    //let param_rate: f32 = 16.0;

    println!("Hexadecimal representation:");
    loop {
        if find_sync(&mut byte_cnt, &mut word_cnt, word_per_sec, buflen, &buf) == false {
            //process::exit(1);  //非正常退出.(未使用)
            break;
        }
        if word_cnt < 128000 {
            let mut rate_cnt: f32 = 0.0;
            for prm_w in prm_words {
                let test_16: u16 = ((buf[byte_cnt + (prm_w - 1) * 2 + 1] as u16) << 8
                    | buf[byte_cnt + (prm_w - 1) * 2] as u16)
                    & 0x0fff;
                frame_time = (subframe_cnt as f32) + (rate_cnt / param_rate);
                value = ((test_16 & 0xfff) as f32) * 0.00228938 - 3.37538;
                println!("subframe:{:.5}, val:{:?}", frame_time, value);
                // println!("{:0x}--{:?}  zhi  {:?}  {}  {:016b}", test_16, word_cnt, value,cycle_time , test_16);

                // 以csv格式写入文件
                wfile
                    .write_all(format!("{:.5},{:?}\r\n", frame_time, value).as_bytes())
                    .expect("写入失败");
                rate_cnt += 1.0;
            }
        } else {
            break;
        }
        byte_cnt += word_per_sec * 2;
        word_cnt += word_per_sec - 1;
        subframe_cnt += 1;
    }
    println!("");
    println!(" The length of data is {}.", buflen);
    println!(" Write CSV to file: \"{}\".", filename_write);
    println!("");
    println!(" 说明: ");
    println!("   把raw文件全部读入内存，然后再解码一个参数。");
    println!("   一个解码配置写死在程序中。只能解码一个参数。");
    println!("      author: osnosn@126.com");
}
fn find_sync(
    byte_cnt: &mut usize,
    word_cnt: &mut usize,
    word_per_sec: usize,
    buflen: usize,
    buf1: &Vec<u8>,
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
                println!(
                    "->找到sync字. wordCnt:{:?}---word:{:0x}",
                    *word_cnt, test_16
                );
                return true;
            }
        }
        *byte_cnt += 1;
    }
}
