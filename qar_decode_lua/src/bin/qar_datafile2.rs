#![allow(non_snake_case)]
//! 读取 自定义格式的数据文件 读入内存
//! 执行 luajit/lua 脚本的执行。  
//! 写入新的 自定义格式的数据文件
//!    author: osnosn@126.com   

use mlua::prelude::*;
use serde::Serialize;
//use mlua::Lua;
use mlua::Table;
use mlua::Value;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
//#[cfg(target_os = "linux")] //lua用到了
use std::process;

#[path = "../CmdLineArgs2.rs"]
mod CmdLineArgs;

#[derive(Serialize, Debug)]
pub struct PrmDict<T> {
    t: f32,
    v: T,
}
#[derive(Serialize, Debug)]
pub struct PrmValue {
    val: PrmType,
    info: String,
}
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum PrmType {
    Float(Vec<PrmDict<f32>>),
    Int(Vec<PrmDict<i32>>),
    Str(Vec<PrmDict<String>>),
}

fn main() {
    let args;
    match CmdLineArgs::parse_args() {
        Ok(tmp) => args = tmp,
        Err(err) => {
            eprintln!("Command line parse ERR.\r\n{err}");
            return ();
        }
    }
    if args.help || args.help2 {
        showHelp(args.bin_name);
        return ();
    }

    // 读取的文件名
    let filename_read;
    if args.infile.len() < 2 {
        showHelp(args.bin_name);
        eprintln!("Error, Missing \"-f\".\r\n");
        return ();
    } else {
        filename_read = args.infile.as_str();
    }

    let (PrmHeader, PrmTable, PrmData);
    if let Some(vv) = read_datafile(filename_read) {
        (PrmHeader, PrmTable, PrmData) = vv;
        println!();
        println!("{}", PrmHeader.meta);
        println!();
        println!("{:?}", PrmTable[0]);
        println!("{:?}", PrmTable[PrmTable.len() - 1]);
        println!();

        // 读取的lua文件名
        let filename_lua = args.luafile.clone();
        let mut lua_string = String::new();
        if filename_lua.len() > 1 {
            if let Ok(vv) = OpenOptions::new()
                .read(true)
                .write(false)
                .create(false)
                .open(filename_lua.clone())
            {
                let mut luafile: File = vv;
                println!("lua程序文件打开成功: {}", filename_lua);
                let _ = luafile.read_to_string(&mut lua_string);
            } else {
                eprintln!("无法打开lua程序文件:\"{}\"", filename_lua);
            }
        }

        let lua = Lua::new();
        //为lua创建一个函数, 获取WordPerSec值。
        let qar_NUM = PrmTable.len();
        let qar_prm_number = lua
            .create_function(move |_, ()| Ok(qar_NUM))
            .expect("lua create_function 'qar_WordPerSec' Fail.");
        lua.globals()
            .set("qar_Prm_Number", qar_prm_number)
            .expect("lua set function 'qar_Prm_Number' Fail.");
        /*
        嵌入执行lua脚本，目前仅测试，为了生成"非记录参数"，或者生成"超限事件"。
           见 main()前面的注释。
        */
        //创建一个 table, 传入当前解码值
        let lua_map_table = lua.create_table().expect("lua create map_table Fail.");
        lua_map_table
            .set("time", "01:02:03".to_string())
            .expect("lua set table 'time' Fail.");
        lua_map_table
            .set("frame", 999)
            .expect("lua set table 'frame' Fail.");
        lua_map_table
            .set("value", 888)
            .expect("lua set table 'value' Fail.");
        lua.globals()
            .set("map_table", lua_map_table)
            .expect("lua set globals table 'map_table' Fail.");
        //创建一个空 table,用于接受由lua新创建的值
        let lua_qar_table = lua.create_table().expect("lua create qar_table Fail.");
        lua.globals()
            .set("qar_table", lua_qar_table)
            .expect("lua set globals table 'qar_table' Fail.");
        //执行lua脚本
        /*
        if let Err(err) = lua
            .load(
                r#"
                         local word_per_sec = qar_Prm_Number(123) --调用rust提供的函数
                         --[[  --块注释
                         io.write('lua:') --不换行输出
                         io.write(' ',word_per_sec,' ')
                         for k,v in pairs(map_table) do
                           io.write(string.format("%s=%s, ",k,v)) --不换行输出
                         end
                         print() --带换行的输出
                         ]]
                         qar_table["wordPerSec"]=word_per_sec  --创建一个新值
                         qar_table["qar"]="test2"              --创建一个新值
                         qar_table["qar2"]=map_table.value +1  --创建一个新值
                         -- qar_table["qar3"]=map_table["value"]+1.0  --同map_table.value
                         "#,
            )
            .exec()
        */
        if let Err(err) = lua.load(lua_string).exec() {
            println!("lua load script & exec Fail.\r\n{err}");
            process::exit(1); //lua脚本执行出错。非正常退出.
        }
        //读取qar_table
        let qar_table: Table = lua
            .globals()
            .get("qar_table")
            .expect("lua get globals table 'qar_table' Fail.");
        //raw_len()/len() 无法获取table的长度。
        //println!("qar_table.len()={}", qar_table.raw_len());

        //把qar_table中的值，拼接为csv格式，准备输出到csv文件。
        let mut qar_csv_out = String::new();
        for pair in qar_table.pairs::<Value, Value>() {
            let (key, val) = pair.unwrap();
            //println!("{:?}:{:?}", key, val);
            //println!("{}:{}", key.to_string().unwrap(), val.to_string().unwrap());
            let mut prm_name = String::new();
            match key.as_str() {
                //取新建的参数名称
                Some(tmp) => prm_name.push_str(format!("{}", tmp).as_str()),
                _ => continue,
            }
            match val {
                //取新建的参数值
                Value::Number(vv) => qar_csv_out.push_str(format!("{:?}", vv as f32).as_str()),
                Value::String(vv) => {
                    qar_csv_out.push_str(format!("{}", vv.to_string_lossy()).as_str())
                }
                Value::Integer(vv) => qar_csv_out.push_str(format!("{}", vv).as_str()),
                _ => continue, //其他类型的值,忽略
                               //_ => unreachable!(), //是 panic!() 的简写
            }
            println!("{}: {}", prm_name, qar_csv_out);
        }
        //qar_csv_out.pop(); //去掉最后一个逗号

        // 写入的文件名
        if args.outfile.len() < 2 {
            eprintln!("Using \"-w out.dat\" to write into file.\r\n");
            return ();
        } else {
            let filename_write = args.outfile.as_str();
            write_datafile(filename_write, PrmHeader, PrmTable, PrmData);
        }
    }

    show_mem(&args);
}
//对于Linux系统，显示内存占用情况
fn show_mem(args: &CmdLineArgs::Args) {
    #[cfg(target_os = "linux")]
    if args.mem {
        // --begin--查看内存占用(linux)
        use std::io::{BufRead, BufReader};
        //println!(" PID is {}.", process::id());
        let status_file = File::open(format!("/proc/{}/status", process::id()))
            .expect("读取'/proc/?/status'失败");
        let vm_keys = ["Name", "VmPeak", "VmSize", "VmHWM", "VmRSS", "VmData"];
        for line in BufReader::new(status_file).lines().flatten() {
            for vm_key in vm_keys {
                if line.contains(vm_key) {
                    println!(" {}.", line);
                    break;
                }
            }
        }
        // --end--查看内存占用(linux)
    }
    #[cfg(target_os = "windows")]
    if args.mem {
        // --begin--查看内存占用(windows)
        eprintln!(" Windows 不支持 --mem 参数。");
        eprintln!("   因为 Windows 不支持 /proc/pid/status 文件的查看。");
    }
}
#[derive(Serialize, Debug)]
pub struct OneParamTable {
    selfsize: u16,
    data_point: u64,
    data_size: u32,
    val_size: u16,
    rate: i16,
    start_frameid: u32,
    name: String,
    compress: String,
    data_type: String,
    info: String,
}
use std::io::Error;
impl OneParamTable {
    //自定义struct的序列化.LittleEndian.
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&self.selfsize.to_le_bytes())?;
        writer.write_all(&self.data_point.to_le_bytes())?;
        writer.write_all(&self.data_size.to_le_bytes())?;
        writer.write_all(&self.val_size.to_le_bytes())?;
        writer.write_all(&self.rate.to_le_bytes())?;
        writer.write_all(&self.start_frameid.to_le_bytes())?;
        writer.write_all(&self.name.as_bytes())?;
        writer.write_all(&[0])?; //写入一个 0 作为字符串结束.
        writer.write_all(&self.compress.as_bytes())?;
        writer.write_all(&[0])?; //写入一个 0 作为字符串结束.
        writer.write_all(&self.data_type.as_bytes())?;
        writer.write_all(&[0])?; //写入一个 0 作为字符串结束.
        writer.write_all(&self.info.as_bytes())?;
        writer.write_all(&[0])?; //写入一个 0 作为字符串结束.
        Ok(())
    }
    fn length(&self) -> usize {
        let mut size = 2 + 8 + 4 + 2 + 2 + 4;
        size += &self.name.len() + 1;
        size += &self.compress.len() + 1;
        size += &self.data_type.len() + 1;
        size += &self.info.len() + 1;
        size
    }
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut u16_bytes = [0; 2];
        reader.read_exact(&mut u16_bytes)?;
        let selfsize = u16::from_le_bytes(u16_bytes);
        let mut u64_bytes = [0; 8];
        reader.read_exact(&mut u64_bytes)?;
        let data_point = u64::from_le_bytes(u64_bytes);
        let mut u32_bytes = [0; 4];
        reader.read_exact(&mut u32_bytes)?;
        let data_size = u32::from_le_bytes(u32_bytes);
        reader.read_exact(&mut u16_bytes)?;
        let val_size = u16::from_le_bytes(u16_bytes);
        reader.read_exact(&mut u16_bytes)?;
        let rate = i16::from_le_bytes(u16_bytes);
        reader.read_exact(&mut u32_bytes)?;
        let start_frameid = u32::from_le_bytes(u32_bytes);
        let str_len = (selfsize - (2 + 8 + 4 + 2 + 2 + 4)) as usize;
        let mut str_bytes = vec![0; str_len];
        reader.read_exact(&mut str_bytes)?;
        //let str_tmp=String::from_utf8_lossy(str_bytes);
        let mut str_iter = str_bytes.splitn(8, |ch| *ch == 0);
        let mut name = String::new();
        let mut compress = String::new();
        let mut data_type = String::new();
        let mut info = String::new();
        if let Some(vv) = str_iter.next() {
            name = String::from_utf8_lossy(vv).to_string();
        }
        if let Some(vv) = str_iter.next() {
            compress = String::from_utf8_lossy(vv).to_string();
        }
        if let Some(vv) = str_iter.next() {
            data_type = String::from_utf8_lossy(vv).to_string();
        }
        if let Some(vv) = str_iter.next() {
            info = String::from_utf8_lossy(vv).to_string();
        }

        Ok(OneParamTable {
            selfsize,
            data_point,
            data_size,
            val_size,
            rate,
            start_frameid,
            name,
            compress,
            data_type,
            info,
        })
    }
}
#[derive(Debug)]
pub struct PrmHeader {
    //tag: "QAR_Decoded_DATA_V1.0\0", //22bytes
    header_size: u32,
    meta: String,
    description: String,
}
impl PrmHeader {
    //自定义struct的序列化.LittleEndian.
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&self.header_size.to_le_bytes())?;
        writer.write_all(&self.meta.as_bytes())?;
        writer.write_all(&[0])?; //写入一个 0 作为字符串结束.
        writer.write_all(&self.description.as_bytes())?;
        writer.write_all(&[0])?; //写入一个 0 作为字符串结束.
        Ok(())
    }
    fn length(&self) -> usize {
        let mut size = 22 + 4; //包含了tag的 22bytes.
        size += &self.meta.len() + 1;
        size += &self.description.len() + 1;
        size
    }
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut u32_bytes = [0; 4];
        reader.read_exact(&mut u32_bytes)?;
        let header_size = u32::from_le_bytes(u32_bytes);
        let str_len = (header_size - (22 + 4)) as usize;
        let mut str_bytes = vec![0; str_len];
        reader.read_exact(&mut str_bytes)?;
        //let str_tmp=String::from_utf8_lossy(str_bytes);
        let mut str_iter = str_bytes.splitn(4, |ch| *ch == 0);
        let mut meta = String::new();
        let mut description = String::new();
        if let Some(vv) = str_iter.next() {
            meta = String::from_utf8_lossy(vv).to_string();
        }
        if let Some(vv) = str_iter.next() {
            description = String::from_utf8_lossy(vv).to_string();
        }

        Ok(PrmHeader {
            header_size,
            meta,
            description,
        })
    }
}
fn read_datafile(filename_read: &str) -> Option<(PrmHeader, Vec<OneParamTable>, Vec<u8>)> {
    let mut PrmTable: Vec<OneParamTable> = vec![];

    // 打开数据文件
    let mut rfile: File = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(filename_read)
        .expect(format!("无法打开DAT数据文件:\"{}\"", filename_read).as_str());
    println!("DAT数据文件打开成功: {}", filename_read);

    let mut header_tag = [0; 22];
    rfile.read_exact(&mut header_tag).unwrap();
    if header_tag != &b"QAR_Decoded_DATA_V1.0\0"[..] {
        eprintln!("DataFile Format Error: {}", filename_read);
        return None;
    }
    let PrmHeader = PrmHeader::deserialize(&mut rfile).expect("PrmHeader 反序列化失败.");

    let mut u32_bytes = [0; 4];
    rfile.read_exact(&mut u32_bytes).unwrap();
    let parameter_ttl_size = u32::from_le_bytes(u32_bytes) - 4; //减去ttl_size本身占用的4bytes
    let mut read_len: u32 = 0;
    let mut point: u64 = 0;
    let mut table_ii = 0;
    //从ttl_size中减去OneParamTable前面固定长度的22bytes,做判断
    while read_len < parameter_ttl_size - 22 {
        table_ii += 1;
        let mut one_parameter_table = OneParamTable::deserialize(&mut rfile)
            .expect(format!("index {} OneParamTable 反序列化失败.", table_ii).as_str());
        one_parameter_table.data_point = point; //重置data指针
        point += one_parameter_table.data_size as u64;
        read_len += one_parameter_table.selfsize as u32;
        PrmTable.push(one_parameter_table);
        /*
        if table_ii < 2 || parameter_ttl_size - read_len < 160 {
            println!("read:{}, ttl:{}", read_len, parameter_ttl_size);
        }
        */
    }
    let mut PrmData: Vec<u8> = vec![];
    let data_size = rfile.read_to_end(&mut PrmData).unwrap();
    println!(
        "Parameter count: {}, DATA total size: {}",
        PrmTable.len(),
        data_size
    );
    return Some((PrmHeader, PrmTable, PrmData));
}
fn write_datafile(
    filename_write: &str,
    mut PrmHeader: PrmHeader,
    mut PrmTable: Vec<OneParamTable>,
    PrmData: Vec<u8>,
) {
    // 创建一个文件，用于写入自定义格式的数据
    let mut wfile = File::create(filename_write)
        .expect(format!("创建文件失败:\"{}\"", filename_write).as_str());
    //------ 写 Header ----------
    let header_tag = b"QAR_Decoded_DATA_V1.0\0"; //自定义数据文件的tag,22bytes
    wfile.write_all(header_tag).expect("写入失败");

    PrmHeader.header_size = PrmHeader.length() as u32;
    let mut buf: Vec<u8> = vec![];
    PrmHeader
        .serialize(&mut buf)
        .expect("PrmHeader serialize失败");
    wfile.write_all(&buf).expect("写入失败");

    //------ 写 Parameter_Table ----------
    let mut param_table_total_size: u32 = 0;
    for ii in 0..PrmTable.len() {
        param_table_total_size += PrmTable[ii].selfsize as u32;
    }
    param_table_total_size += 4; //加上total_size本身的4bytes,
    wfile
        .write_all(&param_table_total_size.to_le_bytes())
        .expect("写入失败");
    let mut data_point = (PrmHeader.header_size + param_table_total_size) as u64;
    //   --- 写,单个参数的table ---
    for ii in 0..PrmTable.len() {
        PrmTable[ii].data_point = data_point; //修改,指向DATA的指针
        data_point += PrmTable[ii].data_size as u64; //加上data_size,指向下一个DATA
        buf = vec![];
        PrmTable[ii]
            .serialize(&mut buf)
            .expect("PrmTable serialize失败");
        wfile.write_all(&buf).expect("写入失败");
    }
    //------ 写 DATA ---------
    wfile.write_all(&PrmData).expect("写入失败");
}
fn showHelp(bin_name: String) {
    println!("Usage: {bin_name} [-f datafile.dat] [-w outfile.dat] [-h | --help]");
    println!("   Detail:");
    println!("      -h        简略的命令行帮助");
    println!("      -l, --lua /path/file.lua   需要执行的lua脚本");
    println!("      -f /path/datafle.dat       指定 读取的文件");
    println!("      -w /path/outfile.dat       指定 写入的文件");
    println!(
        "           自定义格式的datafile.dat, outfile.dat文件,可以用ALL_read_datafile.py读取,并导入pd.DataFrame()"
    );
    println!("      --mem        打印内存占用情况");
    println!(" 说明: ");
    println!("   读取 自定义格式的数据文件。");
    println!("   执行lua脚本。");
    println!("   写入新的 自定义格式的数据文件。");
    println!("      author: osnosn@126.com");
}
