pub struct Args {
    pub bin_name: String, //当前程序名
    pub help: bool,
    pub help2: bool,
    pub mem: bool,
    pub rawfile: String,
    pub csvfile: String,
    pub cmd: String,
}
pub fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut help = false;
    let mut help2 = false;
    let mut mem = false;
    let mut rawfile = None;
    let mut csvfile = None;
    let mut cmd = None;
    let mut parser = lexopt::Parser::from_env();
    let bin_name = parser.bin_name().unwrap_or("myapp").to_string();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('r') => {
                // -r
                rawfile = Some(parser.value()?.string()?);
            }
            Short('w') => {
                // -w
                csvfile = Some(parser.value()?.string()?);
            }
            Short('h') => {
                // -h
                help = true;
            }
            Long("help") => {
                // --help
                help2 = true;
            }
            Long("mem") => {
                // --mem
                mem = true;
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
                //println!("Usage: dump_raw_aligned [1|2|3|4|5|....] [-h | --help]");
                super::showHelp(bin_name);
                std::process::exit(0);
            }
        }
    }

    Ok(Args {
        bin_name, //当前程序名
        help,
        help2,
        mem,
        rawfile: rawfile.unwrap_or("data/raw.dat".to_string()), //缺省值为 "raw.dat"
        csvfile: csvfile.unwrap_or("data/output_data.csv".to_string()),
        cmd: cmd.unwrap_or("".to_string()), //缺省值为 ""
    })
}
