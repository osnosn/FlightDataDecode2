pub struct Args {
    pub help: bool,
    pub help2: bool,
    pub rawfile: String,
    pub cmd: String,
}
pub fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut help = false;
    let mut help2 = false;
    let mut rawfile = None;
    let mut cmd = None;
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('f') => {
                // -f
                rawfile = Some(parser.value()?.string()?);
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
                println!("Usage: dump_raw_aligned [1|2|3|4|5|....]");
                std::process::exit(0);
            }
        }
    }

    Ok(Args {
        help,
        help2,
        rawfile: rawfile.unwrap_or("data/raw.dat".to_string()), //缺省值为 "raw.dat"
        cmd: cmd.unwrap_or("".to_string()),                     //缺省值为 ""
    })
}
