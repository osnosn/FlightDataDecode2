pub struct Args {
    pub bin_name: String, //当前程序名
    pub help: bool,
    pub help2: bool,
    pub mem: bool,
    pub infile: String,
    pub outfile: String,
    pub luafile: String,
    pub cmd: String,
}
pub fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;
    use std::path::Path;

    let mut help = false;
    let mut help2 = false;
    let mut mem = false;
    let mut infile = None;
    let mut outfile = None;
    let mut luafile = None;
    let mut cmd = None;
    let mut parser = lexopt::Parser::from_env();
    //let bin_name = parser.bin_name().unwrap_or("myapp").to_string();
    let bin_name = Path::new(parser.bin_name().unwrap_or("myApp"))
        .file_name()
        .ok_or_else(|| "Error获取basename")?
        .to_string_lossy()
        .to_string();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('f') => {
                // -r
                infile = Some(parser.value()?.string()?);
            }
            Short('w') => {
                // -w
                outfile = Some(parser.value()?.string()?);
            }
            Short('l') | Long("lua") => {
                // -l --lua
                luafile = Some(parser.value()?.string()?);
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
        infile: infile.unwrap_or("".to_string()), //缺省值为 ""
        outfile: outfile.unwrap_or("".to_string()),
        luafile: luafile.unwrap_or("".to_string()),
        cmd: cmd.unwrap_or("".to_string()), //缺省值为 ""
    })
}
