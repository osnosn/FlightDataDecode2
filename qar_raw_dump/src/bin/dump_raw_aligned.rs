#![allow(non_snake_case)]

//! 扫描raw文件, 通过sync同步字出现的顺序和间隔, 判断是否 aligned 格式。  
//!    author: osnosn@126.com  OR  LLGZ@csair.com  

use std::fs::File;
use std::io::BufReader;
use std::io::Read;

#[path = "../CmdLineArgs_aligned.rs"]
mod CmdLineArgs_aligned;

fn main() {
    let args;
    match CmdLineArgs_aligned::parse_args() {
        Ok(tmp) => args = tmp,
        _ => {
            showHelp();
            return ();
        }
    }
    if args.help || args.help2 || args.rawfile.len() < 2 {
        showHelp();
        return ();
    }

    // 打开数据文件
    //let file = File::open("raw.dat").expect("无法打开文件");
    let file =
        File::open(&args.rawfile).expect(format!("无法打开文件:\"{}\"", args.rawfile).as_str());
    println!("文件打开成功：{:?}", args.rawfile);

    // 使用缓冲读取器读取文件内容
    let mut reader: BufReader<File> = BufReader::new(file);

    // 读取文件内容到一个缓冲区
    let mut buf: Vec<u8> = Vec::new();
    reader
        .read_to_end(&mut buf)
        .expect(format!("读取文件失败:\"{}\"", args.rawfile).as_str());

    let mut word_cnt = 0; //word 计数
    let mut word_cnt2 = 0; //前一个sync字的 word 计数
    let mut byte_cnt = 0; //奇数/偶数 字节标记
    let mut pre_byte = 0; //前一个字节
                          //let mut mark = -1; //#byte起始位置标记, 0 or 1

    //println!("Hexadecimal representation:");
    println!("  说明: word=(16bit), byte=0-1(8bit), len=(16bit word)间隔,");
    for byte in &buf {
        byte_cnt += 1;

        let test_16: u16 = (*byte as u16) << 8 | pre_byte as u16;
        pre_byte = *byte;
        // print!("{:0x}--", test_16);
        // 0x247,0x5B8,0xA47,0xDB8
        if test_16 == 0x247 {
            //mark = byte_cnt & 0x1;
            println!(
                "--->Mark sync1   .at 0x{:<5X}word + {:>01?}byte,len:0x{:<X}",
                word_cnt,
                byte_cnt & 0x1,
                word_cnt - word_cnt2,
            );
            word_cnt2 = word_cnt;
        } else if test_16 == 0x5B8 {
            //mark = byte_cnt & 0x1;
            println!(
                "--->Mark sync 2  .at 0x{:<5X}word + {:>01?}byte,len:0x{:<X}",
                word_cnt,
                byte_cnt & 0x1,
                word_cnt - word_cnt2,
            );
            word_cnt2 = word_cnt;
        } else if test_16 == 0xA47 {
            //mark = byte_cnt & 0x1;
            println!(
                "--->Mark sync  3 .at 0x{:<5X}word + {:>01?}byte,len:0x{:<X}",
                word_cnt,
                byte_cnt & 0x1,
                word_cnt - word_cnt2,
            );
            word_cnt2 = word_cnt;
        } else if test_16 == 0xDB8 {
            //mark = byte_cnt & 0x1;
            println!(
                "--->Mark sync   4.at 0x{:<5X}word + {:>01?}byte,len:0x{:<X}",
                word_cnt,
                byte_cnt & 0x1,
                word_cnt - word_cnt2,
            );
            word_cnt2 = word_cnt;
        }
        if byte_cnt > 0 && (byte_cnt & 0x1) == 0 {
            word_cnt += 1;
            if args.rawlen > 0 && byte_cnt > args.rawlen {
                println!("---文件已经扫描{} (bytes)字节, 结束---", args.rawlen);
                break;
            }
        } //#测试用，暂时读500k就结束
    }
    println!("  说明: word=(16bit), byte=0-1(8bit), len=(16bit word)间隔,");

    //println!("{:02x}", pre_byte);
    println!(
        "The length of data is {}, 0x{:X} (bytes).",
        buf.len(),
        buf.len()
    );
}
fn showHelp() {
    println!("Usage: dump_raw_aligned [-h | --help] [-c 50000] -f raw.dat");
    println!("   Detail:");
    println!("      -h        简略的命令行帮助");
    println!("      --help     详细的帮助信息");
    println!("      -f /path/raw.dat    指定raw文件");
    println!("      -c 50000    扫描的字节数, 0:扫描整个文件, 默认=50000,");
    println!("");
}
