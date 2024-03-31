#![allow(non_snake_case)]

//! 扫描raw文件, 通过sync同步字出现的顺序和间隔, 判断是否 bitstream 格式。  
//! 显示12bit word计数.  
//!    author: osnosn@126.com  OR  LLGZ@csair.com  

use std::fs::File;
use std::io::BufReader;
use std::io::Read;

#[path = "../CmdLineArgs_bitstream.rs"]
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
    let buflen = buf.len();

    let mut fff_cnt = 0; //连续的0xfff同步位 计数,12bit 计数
    let mut word_cnt = 0; //12bit word 计数
    let mut word_cnt2 = 0; //前一个sync字的 word 计数位置
    let mut bit_cnt = 0; //12bit中 当前bit位置0-11
    let mut word_12bit: u16 = 0; //12bit 目标缓存

    //println!("Hexadecimal representation:");
    println!("  说明: word=12bit, bits=0-11, len=(12bit word)间隔,");

    //GetBit::new()的第二个参数是order: false=high,bit7->bit0, true=low,bit0->bit7
    let MyBits = GetBit::new(buf, args.low_first);
    for bit in MyBits {
        // move_right: true=">>", false="<<"
        if args.move_right {
            word_12bit >>= 1;
        } else {
            word_12bit <<= 1;
        }
        if bit {
            if args.move_right {
                word_12bit |= 0x800;
            } else {
                word_12bit |= 0x1;
            }
        }
        word_12bit &= 0xfff;

        // 0x247,0x5B8,0xA47,0xDB8
        if word_12bit == 0x247 {
            println!(
                "--->Mark sync1   .at 0x{:<5X}word + {:>02}bits,len:0x{:<X}",
                word_cnt,
                bit_cnt,
                word_cnt - word_cnt2,
            );
            word_cnt2 = word_cnt;
        } else if word_12bit == 0x5B8 {
            println!(
                "--->Mark sync 2  .at 0x{:<5X}word + {:>02}bits,len:0x{:<X}",
                word_cnt,
                bit_cnt,
                word_cnt - word_cnt2,
            );
            word_cnt2 = word_cnt;
        } else if word_12bit == 0xA47 {
            println!(
                "--->Mark sync  3 .at 0x{:<5X}word + {:>02}bits,len:0x{:<X}",
                word_cnt,
                bit_cnt,
                word_cnt - word_cnt2,
            );
            word_cnt2 = word_cnt;
        } else if word_12bit == 0xDB8 {
            println!(
                "--->Mark sync   4.at 0x{:<5X}word + {:>02}bits,len:0x{:<X}",
                word_cnt,
                bit_cnt,
                word_cnt - word_cnt2,
            );
            word_cnt2 = word_cnt;
        }
        bit_cnt += 1;
        if bit_cnt >= 12 {
            bit_cnt = 0;
            word_cnt += 1;
            if word_12bit == 0xfff {
                fff_cnt += 1;
            } else {
                if fff_cnt > 64 {
                    //发现超过64个 0xfff 的值,对于12bit来说，就是连续的0b1出现了很长了。
                    println!("---> 连续出现 words 0xFFF 的个数: 0x{:<5X}.", fff_cnt);
                }
                fff_cnt = 0;
            }
        }
        if args.rawlen > 0 && word_cnt > args.rawlen {
            println!("---文件已经扫描{}(12bit words), 结束---", args.rawlen);
            break;
        } //#测试用，暂时读500k就结束
    }
    println!("  说明: word=12bit, bits=0-11, len=(12bit word)间隔,");

    //println!("{:02x}", pre_byte);
    println!("The length of data is {}, 0x{:X} (bytes).", buflen, buflen);
    println!(
        "The length of data is {}, 0x{:X} (12bit words).",
        buflen * 8 / 12,
        buflen * 8 / 12
    );
}

//  自定义一个迭代器,从buf中一个bit一个bit取,
pub struct GetBit {
    pub buf: Vec<u8>,
    pub len: usize,      //buf的总字节数, ttl bytes
    pub byte_cnt: usize, //byte 计数
    pub bit: u8,         //bit 掩码
    pub order: bool,     //low_first: false=high,bit7->bit0, true=low,bit0->bit7
}
impl GetBit {
    pub fn new(buf: Vec<u8>, order: bool) -> Self {
        let bit;
        if order {
            bit = 0x01;
        } else {
            bit = 0x80;
        }
        Self {
            len: buf.len(),
            buf: buf,
            byte_cnt: 0,
            bit,
            order: order,
        }
    }
}
impl Iterator for GetBit {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.byte_cnt > self.len - 1 {
            //文件结束
            return None;
        }
        let result;
        //取一个bit
        if self.buf[self.byte_cnt] & self.bit == 0 {
            result = false;
        } else {
            result = true;
        }
        if self.order {
            self.bit <<= 1;
        } else {
            self.bit >>= 1;
        }
        if self.bit == 0 {
            if self.order {
                self.bit = 0x01;
            } else {
                self.bit = 0x80;
            }
            self.byte_cnt += 1;
        }
        Some(result)
    }
}

fn showHelp() {
    println!("Usage: dump_raw_aligned [-h | --help] [-m] [-c 50000] -f raw.dat");
    println!("   Detail:");
    println!("      -h        简略的命令行帮助");
    println!("      --help     详细的帮助信息");
    println!("      -f /path/raw.dat    指定raw文件");
    println!("      -c 50000  扫描(12bit words)的数量, 0:扫描整个文件, 默认=50000,");
    println!("     默认情况:无\"--low --right\"参数, ");
    println!("         读取raw文件的每个8bit字节后,取bit顺序, 先取高位, bit7->bit0");
    println!("         取完bit后, 拼接12bit word时,从低位移入,移位方向left\"<<\"");
    println!("      --low     读取raw文件的每个8bit字节后,取bit顺序, 先取低位, bit0->bit7");
    println!("      --right   取完bit后, 拼接12bit word时，从高位移入,移位方向right\">>\"");
    println!("");
}
