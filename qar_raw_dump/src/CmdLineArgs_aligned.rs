pub struct Args {
    pub help: bool,
    pub help2: bool,
    pub rawfile: String, //raw文件名
    pub rawlen: usize,   //扫描的字节数
    pub cmd: String,     //未使用
}
pub fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut help = false;
    let mut help2 = false;
    let mut rawfile = None;
    let mut rawlen: usize = 50000; //默认5k
    let mut cmd = None;
    let mut parser = lexopt::Parser::from_env();
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
                super::showHelp();
                std::process::exit(0);
            }
        }
    }

    Ok(Args {
        help,
        help2,
        rawfile: rawfile.unwrap_or("data/raw.dat".to_string()), //缺省值为 "raw.dat"
        rawlen,
        cmd: cmd.unwrap_or("".to_string()), //缺省值为 ""
    })
}
