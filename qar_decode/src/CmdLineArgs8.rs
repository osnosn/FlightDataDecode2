pub struct Args {
    pub bin_name: String, //当前程序名
    pub help: bool,
    pub help2: bool,
    pub mem: bool,
    pub json: String,
    pub param: String,
    pub paramlist: bool,
    pub allparam: bool,
    pub rawfile: String,
    pub outfile: String,
    pub custom_detail: bool,
    pub cmd: String,
}
pub fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;
    use std::path::Path;

    let mut help = false;
    let mut help2 = false;
    let mut mem = false;
    let mut json = None;
    let mut param = None;
    let mut paramlist = false;
    let mut allparam = false;
    let mut rawfile = None;
    let mut outfile = None;
    let mut custom_detail = false;
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
            Short('r') => {
                // -r
                rawfile = Some(parser.value()?.string()?);
            }
            Short('w') => {
                // -w
                outfile = Some(parser.value()?.string()?);
            }
            Short('j') | Long("json") => {
                // -j , --json
                json = Some(parser.value()?.string()?);
            }
            Short('p') | Long("param") => {
                // -p
                param = Some(parser.value()?.string()?);
            }
            Short('l') | Long("paramlist") => {
                paramlist = true;
            }
            Short('a') | Long("all") => {
                allparam = true;
            }
            Short('h') => {
                // -h
                help = true;
            }
            Long("show") => {
                custom_detail = true;
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
        json: json.unwrap_or("prm_conf320.json".to_string()), //缺省值为 "prm_conf320.json"
        param: param.unwrap_or("".to_string()),               //缺省值为 ""
        paramlist,
        allparam,
        rawfile: rawfile.unwrap_or("".to_string()), //缺省值为 ""
        outfile: outfile.unwrap_or("data/output_data.csv".to_string()),
        custom_detail,
        cmd: cmd.unwrap_or("".to_string()), //缺省值为 ""
    })
}
