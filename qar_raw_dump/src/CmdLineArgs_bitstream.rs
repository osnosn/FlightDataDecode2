pub struct Args {
    pub bin_name: String, //当前程序名
    pub help: bool,
    pub help2: bool,
    pub rawfile: String,  //raw文件名
    pub rawlen: usize,    //扫描的字节数
    pub move_left: bool,  //拼接方向,
    pub high_first: bool, //先取低位
    pub no_align: bool,   //扫描时,是否bit对齐
    pub cmd: String,      //未使用
}
pub fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;
    use std::path::Path;

    let mut help = false;
    let mut help2 = false;
    let mut rawfile = None;
    let mut rawlen: usize = 50000; //默认5k
    let mut move_left = false;
    let mut high_first = false;
    let mut no_align = false;
    let mut cmd = None;
    let mut parser = lexopt::Parser::from_env();
    //let bin_name = parser.bin_name().unwrap_or("myApp").to_string();
    let bin_name = Path::new(parser.bin_name().unwrap_or("myApp"))
        .file_name()
        .ok_or_else(|| "Error获取basename")?
        .to_string_lossy()
        .to_string();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('f') => {
                // -f
                rawfile = Some(parser.value()?.string()?);
            }
            Short('c') => {
                // -c
                rawlen = parser.value()?.parse()?;
            }
            Long("left") => {
                // -m
                move_left = true;
            }
            Long("high") => {
                // -m
                high_first = true;
            }
            Long("noalign") => {
                // -m
                no_align = true;
            }
            Short('h') => {
                // -h
                help = true;
            }
            Long("help") => {
                // --help
                help2 = true;
            }
            Value(val) if cmd.is_none() => {
                //获取命令行第一个值
                cmd = Some(val.string()?);
            }
            Value(_val) => { //忽略命令行后续的值
            }
            _ => {
                println!("{:#?}", arg.unexpected());
                //println!("Usage: dump_raw_aligned -f raw.dat [-h | --help]");
                super::showHelp(bin_name);
                std::process::exit(0);
            }
        }
    }

    Ok(Args {
        bin_name, //当前程序名
        help,
        help2,
        rawfile: rawfile.unwrap_or("".to_string()), //缺省值为 ""
        rawlen,
        move_left,
        high_first,
        no_align,
        cmd: cmd.unwrap_or("".to_string()), //缺省值为 ""
    })
}
