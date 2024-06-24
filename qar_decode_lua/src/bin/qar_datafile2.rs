#![allow(non_snake_case)]
//! 读取 自定义格式的数据文件 读入内存
//! 执行 luajit/lua 脚本的执行。  
//! 写入新的 自定义格式的数据文件
//!    author: osnosn@126.com   

use mlua::prelude::*;
use serde::Serialize;
//use mlua::Lua;
use mlua::Table;
//use mlua::Value;
use bzip2::bufread::BzDecoder;
use mlua::AnyUserData;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
#[cfg(not(target_os = "windows"))]
use std::io::{BufRead, BufReader};
#[cfg(target_os = "linux")]
use std::process;
//use serde::Deserialize;
//use bzip2::read::BzDecoder;
//use bzip2::write::BzDecoder;
//use core::slice::SlicePattern;

#[path = "../CmdLineArgs2.rs"]
mod CmdLineArgs;

#[derive(Serialize, Debug)]
//#[derive(Serialize, Deserialize, Debug)]
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

//========lua qar_userdata===begin====
pub struct Qar {
    PrmHeader: PrmHeader,
    PrmTable: Vec<OneParamTable>,
    PrmData: Vec<Vec<u8>>,
}
impl Qar {
    pub fn new(PrmHeader: PrmHeader, PrmTable: Vec<OneParamTable>, PrmData: Vec<Vec<u8>>) -> Self {
        Self {
            PrmHeader,
            PrmTable,
            PrmData,
        }
    }
}
//参考 https://github.com/mlua-rs/mlua/blob/master/examples/userdata.rs
impl LuaUserData for Qar {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("meta", |_lua, this| Ok(this.PrmHeader.meta.clone()));
        fields.add_field_method_set("meta", |_lua, this, val: LuaValue| {
            this.PrmHeader.meta = val.to_string()?;
            Ok(())
        });
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        //methods.add_function("get1", |_lua, ()| Ok(Some("test get1".to_string())));
        //methods.add_method("get2", |_lua, _this, ()| Ok(345));
        //methods.add_method("get3", |_lua, _this, ()| Ok(Some("test.get".to_string())));
        //methods.add_method("getmetalen", |_lua, this, ()| Ok(this.meta.len()));
        methods.add_method("get_meta", |_lua, this, ()| Ok(this.PrmHeader.meta.clone()));
        methods.add_method_mut("set_meta", |_lua, this, val: LuaValue| {
            this.PrmHeader.meta = val.to_string()?;
            Ok(())
        });
        methods.add_method("get_param", |lua, this, val: LuaValue| {
            //val:LuaString 或 val:String ,如果传入table会panic,
            //  如果 panic = "abort" 就不能显示以下错误信息
            //  报错: error converting Lua table to string (expected string or number)
            //let pm_name=val;  //val:String
            //let pm_name=val.to_string_lossy().to_string();  //val:LuaString
            let pm_name = val.to_string()?; //val: LuaValue
            let prm = lua.create_table()?;
            let mut found = None;
            for ii in 0..this.PrmTable.len() {
                if this.PrmTable[ii].name == pm_name {
                    let mut rate = this.PrmTable[ii].rate as f32;
                    if rate > 0.0 {
                        rate = 1.0 / rate;
                    } else {
                        rate *= -1.0;
                    }
                    prm.set("rate", rate)?;
                    prm.set("type", this.PrmTable[ii].data_type.clone())?;
                    prm.set("info", this.PrmTable[ii].info.clone())?;
                    found = Some(ii);
                    break;
                }
            }
            match found {
                None => return Ok(None),
                Some(vv) => {
                    //let data_idx = this.PrmTable[vv].data_point as usize;
                    //PrmTable的顺序和PrmData的顺序是一样的,所以 vv==data_idx
                    /*
                    println!(
                        "buf2: {},{}, prmtable:{}, data:{}",
                        buf2.len(),
                        this.PrmTable[vv].data_size,
                        vv,
                        data_idx
                    );
                    */
                    let buf2 = &this.PrmData[vv];
                    let mut buf: Vec<u8> = Vec::new();
                    BzDecoder::new(buf2.as_slice())
                        .read_to_end(&mut buf)
                        .expect("bzip2解压失败");

                    match this.PrmTable[vv].data_type.as_str() {
                        "float" => {
                            prm.set("val", lua.create_table()?)?;
                            let prmval: Table = prm.get("val")?;
                            //设置为array()
                            prmval.set_metatable(Some(lua.array_metatable()));
                            //float 就是 f32, 所以忽略了 val_size值,默认为4
                            //val_size=this.PrmTable[vv].val_size as usize;
                            let mut u32_bytes: [u8; 4] = [0; 4];
                            let mut rate = this.PrmTable[vv].rate as f32;
                            if rate > 0.0 {
                                rate = 1.0 / rate;
                            } else {
                                rate *= -1.0;
                            }
                            let mut frame = this.PrmTable[vv].start_frameid;
                            let mut ii = 0;
                            while ii < buf.len() {
                                u32_bytes.copy_from_slice(&buf[ii..ii + 4]);
                                let data = f32::from_le_bytes(u32_bytes);
                                //prmval.push(data)?;
                                let oneval = lua.create_table()?;
                                oneval.set_metatable(Some(lua.array_metatable()));
                                oneval.push(frame)?;
                                oneval.push(data)?;
                                prmval.push(oneval)?;
                                frame += rate;
                                ii += 4;
                            }
                        }
                        "int" => {
                            prm.set("val", lua.create_table()?)?;
                            let prmval: Table = prm.get("val")?;
                            //设置为array()
                            prmval.set_metatable(Some(lua.array_metatable()));
                            //int 就是 i32, 所以忽略了 val_size值,默认为4
                            //val_size=this.PrmTable[vv].val_size as usize;
                            let mut u32_bytes: [u8; 4] = [0; 4];
                            let mut rate = this.PrmTable[vv].rate as f32;
                            if rate > 0.0 {
                                rate = 1.0 / rate;
                            } else {
                                rate *= -1.0;
                            }
                            let mut frame = this.PrmTable[vv].start_frameid;
                            let mut ii = 0;
                            while ii < buf.len() {
                                u32_bytes.copy_from_slice(&buf[ii..ii + 4]);
                                let data = i32::from_le_bytes(u32_bytes);
                                //prmval.push(data)?;
                                let oneval = lua.create_table()?;
                                oneval.set_metatable(Some(lua.array_metatable()));
                                oneval.push(frame)?;
                                oneval.push(data)?;
                                prmval.push(oneval)?;
                                frame += rate;
                                ii += 4;
                            }
                        }
                        _ => {
                            //其他格式,为文本,比如json, 不处理
                            prm.set("val", lua.create_string(&buf)?)?;
                        }
                    }
                }
            }
            Ok(Some(prm))
        });
        methods.add_method_mut(
            "set_param",
            |_lua, this, (val, table): (LuaValue, LuaValue)| {
                //先获取table中的值
                if !table.is_table() {
                    /*
                    return Err(LuaError::RuntimeError(
                        "Parameter DATA is NOT Table".to_string(),
                    ));
                    */
                    return Ok((-2, "NotTable".to_string()));
                }
                let table_0 = table.as_table().unwrap();
                //.ok_or(LuaError::RuntimeError("as_table Fail.".to_string()))?;

                let dataval_val = table_0.get::<_, LuaValue>("val")?;
                //通过检查所有值,判断一下,值的类型
                let mut datatype = 'S'; //默认为string
                if !dataval_val.is_string() {
                    if !dataval_val.is_table() {
                        return Ok((-3, "val NotTable,String".to_string()));
                    }
                    let dataval_t = dataval_val.as_table().unwrap(); //table.val
                    for ii in 1..=dataval_t.len()? {
                        let dataval_v1 = dataval_t.get::<_, LuaValue>(ii)?; //table.val[ii]
                        if !dataval_v1.is_table() {
                            return Ok((-4, format!("val[{ii}] NotTable")));
                        }
                        let dataval_t1 = dataval_v1.as_table().unwrap(); //table.val[ii]

                        let dataval_v11 = dataval_t1.get::<_, LuaValue>(1)?; //table.val[ii][1]
                        if !dataval_v11.is_number() && !dataval_v11.is_integer() {
                            // 2.0 会被lua解析为 2
                            return Ok((-5, format!("val[{ii}][1]Not(Number,Integer)")));
                        }

                        let dataval_v12 = dataval_t1.get::<_, LuaValue>(2)?; //table.val[ii][2]
                        if dataval_v12.is_number() {
                            datatype = 'F'; //为float
                        } else if dataval_v12.is_integer() || dataval_v12.is_boolean() {
                            if datatype != 'F' {
                                //只要有一组值是 float,则全部按float处理
                                datatype = 'I'; //为int
                            }
                        } else {
                            return Ok((-6, format!("val[{ii}][1]Not(Number,Integer,Bool)")));
                        }
                    }
                }

                //获取参数名
                let pm_name = val.to_string()?;
                if pm_name.len() < 1 {
                    return Ok((-1, "EmptyName".to_string()));
                }
                //寻找参数名,是否已经存在
                let mut found = None;
                for ii in 0..this.PrmTable.len() {
                    if this.PrmTable[ii].name == pm_name {
                        found = Some(ii);
                        break;
                    }
                }
                let idx;
                match found {
                    None => {
                        //没找到, 创建一个新参数名
                        let OnePrmTable = OneParamTable {
                            selfsize: 0u16,
                            data_point: 0u64,
                            data_size: 0u32,
                            val_size: 0u16,
                            rate: 0i16,
                            start_frameid: 0.0_f32,
                            name: pm_name,
                            compress: "none".to_string(),
                            data_type: "str".to_string(),
                            info: "{}".to_string(), //初始值为{}
                        };
                        let data: Vec<u8> = vec![];
                        this.PrmTable.push(OnePrmTable);
                        this.PrmData.push(data);
                        idx = this.PrmTable.len() - 1;
                    }
                    Some(vv) => idx = vv,
                }

                {
                    //设置info的值
                    let info_v = table_0.get::<_, LuaValue>("info")?; //table.info
                    if info_v.is_string() {
                        this.PrmTable[idx].info = info_v.to_string()?;
                    }
                }
                let mut buf: Vec<u8> = vec![]; //序列化的缓存
                if datatype == 'S' {
                    //table.val 是 str
                    //这是纯文本的参数值
                    this.PrmTable[idx].data_type = "str".to_string();
                    buf = dataval_val.to_string()?.into();
                } else {
                    //重新取值,按table取
                    let dataval_val = table_0.get::<_, LuaTable>("val")?;
                    //只取了前两个frameID
                    let mut frame1: f32 = 0.0;
                    let mut frame2: f32 = 0.0;
                    //let mut frame: f32;
                    let mut data_i32: i32;
                    let mut data_f32: f32;
                    let mut ii = 0;
                    for val_tab in dataval_val.sequence_values::<LuaTable>() {
                        //sequence_values() 就是 lua的 ipairs()
                        ii += 1;
                        let val_tab2: LuaValue = val_tab.clone()?.get(2)?;
                        //frame = val_tab.clone()?.get(1)?;
                        if ii == 1 {
                            frame1 = val_tab.clone()?.get(1)?;
                        } else if ii == 2 {
                            frame2 = val_tab.clone()?.get(1)?;
                        }
                        if datatype == 'I' {
                            data_i32 = if val_tab2.is_boolean() {
                                //bool类型不能自动转换为i32或f32
                                if val_tab2.as_boolean().unwrap() {
                                    1
                                } else {
                                    0
                                }
                            } else {
                                val_tab.clone()?.get(2)?
                            };
                            //println!("int: {}, {}", frame, data_i32);
                            buf.write_all(&data_i32.to_le_bytes()).unwrap(); //仅返回 Ok(()),不会出错
                        } else if datatype == 'F' {
                            data_f32 = if val_tab2.is_boolean() {
                                //bool类型不能自动转换为i32或f32
                                if val_tab2.as_boolean().unwrap() {
                                    1.0
                                } else {
                                    0.0
                                }
                            } else {
                                val_tab.clone()?.get(2)?
                            };
                            //println!("int: {}, {}", frame, data_f32);
                            buf.write_all(&data_f32.to_le_bytes()).unwrap(); //仅返回 Ok(()),不会出错
                        }
                    }
                    //计算 frame_rate, 正值表示每秒几个记录, 负值表示几秒记录一次.
                    let mut frame_rate: f32 = 0.0;
                    if ii > 2 {
                        frame_rate = frame2 - frame1;
                    }
                    if frame_rate != 0.0 && frame_rate <= 1.0 {
                        this.PrmTable[idx].rate = (1.0 / frame_rate) as i16;
                    } else {
                        this.PrmTable[idx].rate = (frame_rate * -1.0) as i16;
                    }
                    this.PrmTable[idx].start_frameid = frame1;
                    //填入值的类型,长度
                    this.PrmTable[idx].val_size = 4;
                    if datatype == 'I' {
                        this.PrmTable[idx].data_type = "int".to_string();
                    } else if datatype == 'F' {
                        this.PrmTable[idx].data_type = "float".to_string();
                    }
                }
                //使用bzip2压缩,bz2; (需占7-9MB内存,占内存较小)
                use bzip2::read::BzEncoder;
                use bzip2::Compression;
                this.PrmTable[idx].compress = "bzip2".to_string();
                let mut buf2: Vec<u8> = Vec::new(); //压缩的缓存
                BzEncoder::new(buf.as_slice(), Compression::best())
                    .read_to_end(&mut buf2)
                    .expect("bzip2失败");

                this.PrmTable[idx].data_size = buf2.len() as u32;
                this.PrmTable[idx].selfsize = this.PrmTable[idx].length() as u16;
                this.PrmData[idx] = buf2;
                if datatype == 'I' {
                    return Ok((1, "OK".to_string()));
                } else if datatype == 'F' {
                    return Ok((2, "OK".to_string()));
                } else {
                    return Ok((3, "OK".to_string()));
                }
            },
        );
        methods.add_method_mut("del_param", |_lua, this, val: LuaValue| {
            //获取参数名
            let pm_name = val.to_string()?;
            if pm_name.len() < 1 {
                return Ok((-1, "EmptyName".to_string()));
            }
            //寻找参数名,是否已经存在
            let mut found = None;
            for ii in 0..this.PrmTable.len() {
                if this.PrmTable[ii].name == pm_name {
                    found = Some(ii);
                    break;
                }
            }
            match found {
                Some(vv) => {
                    this.PrmTable.remove(vv);
                    this.PrmData.remove(vv);
                    return Ok((1, "OK".to_string()));
                }
                None => {
                    //没找到
                    return Ok((-2, "NotFound".to_string()));
                }
            }
        });
        methods.add_method("get_param_list", |lua, this, ()| {
            let prm = lua.create_table()?;
            for ii in 0..this.PrmTable.len() {
                let oneval = lua.create_table()?;
                oneval.push(this.PrmTable[ii].name.clone())?;
                oneval.push(this.PrmTable[ii].info.clone())?;
                prm.push(oneval)?;
            }
            Ok(Some(prm))
        });
        methods.add_method("get_param_num", |_lua, this, ()| Ok(this.PrmTable.len()));
    }
}
//========lua qar_userdata===end====
//========lua json.encode, json.decode===begin====
// 参考 https://github.com/benwilber/mlua-userdata-json/blob/main/src/lib.rs
//use serde_json;
fn json_encode(value: LuaValue, pretty: Option<bool>) -> Result<Option<String>, LuaError> {
    match pretty {
        Some(true) => match serde_json::to_string_pretty(&value) {
            Ok(s) => Ok(Some(s)),
            Err(e) => Err(LuaError::SerializeError(e.to_string())),
        },
        _ => match serde_json::to_string(&value) {
            Ok(s) => Ok(Some(s)),
            Err(e) => Err(LuaError::SerializeError(e.to_string())),
        },
    }
}
pub struct Json;
impl Json {
    pub fn new() -> Self {
        Self {}
    }
}
impl Default for Json {
    fn default() -> Self {
        Self::new()
    }
}
impl LuaUserData for Json {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("null", |lua, _| Ok(lua.null()));
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("array", |lua, table: Option<LuaTable>| {
            let array = match table {
                Some(table) => table,
                None => lua.create_table()?,
            };
            array.set_metatable(Some(lua.array_metatable()));
            Ok(array)
        });
        methods.add_function(
            "encode",
            |_lua, (value, pretty): (LuaValue, Option<bool>)| json_encode(value, pretty),
        );
        methods.add_meta_method(
            LuaMetaMethod::Call,
            |_lua, _this, (value, pretty): (LuaValue, Option<bool>)| json_encode(value, pretty),
        );
        methods.add_function("decode", |lua, value: String| {
            match serde_json::from_str::<serde_json::Value>(&value) {
                Ok(value) => Ok(lua.to_value(&value)?),
                Err(e) => Err(LuaError::DeserializeError(e.to_string())),
            }
        });
    }
}
//========lua json.encode, json.decode===end====
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
    if args.luahelp {
        showLuaHelp(args.bin_name);
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
        /*
        println!();
        println!("{}", PrmHeader.meta);
        println!();
        println!("{:?}", PrmTable[0]);
        println!("{:?}", PrmTable[PrmTable.len() - 1]);
        println!();
        */

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
        let Qar_userdata = Qar::new(PrmHeader, PrmTable, PrmData);

        println!(" -----准备执行 lua 脚本-----");
        let lua = Lua::new();
        lua.globals()
            .set("qar", Qar_userdata)
            .expect("lua create_userdata 'qar' Fail.");
        lua.globals()
            .set("json", Json::new())
            .expect("lua create_userdata 'qar' Fail.");
        //执行lua前,刷新rust的输出缓冲区
        std::io::stdout().flush().unwrap();
        std::io::stderr().flush().unwrap();
        /*
        嵌入执行lua脚本，目前仅测试，为了生成"非记录参数"，或者生成"超限事件"。
           见 main()前面的注释。
        */
        //执行lua脚本,脚本样例
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
            println!(" -----ERROR, lua 脚本执行错误-----\r\n{err}");
            //lua脚本执行出错。可以不退出rust主程序。
            //process::exit(1); //也可以非正常退出.
        }
        //执行lua脚本后, 刷新lua的输出缓冲区
        lua.load("io.flush()")
            .exec()
            .expect("ERROR, LUA flush output Fail.");
        //rust中不能刷新lua的输出缓冲区
        //std::io::stdout().flush();
        //std::io::stderr().flush();
        println!(" -----lua 脚本执行结束-----");

        //获取 userdata
        // 参考 https://github.com/mlua-rs/mlua/discussions/30
        let Qar_userdata_any = lua
            .globals()
            .raw_get::<_, AnyUserData>("qar")
            .expect("get qar_userdata Fail.");
        let Qar_userdata = Qar_userdata_any
            .borrow::<Qar>()
            .expect("borrow qar_userdata Fail.");

        // 写入的文件名
        if args.outfile.len() < 2 {
            eprintln!("Using \"-w out.dat\" to write into file.\r\n");
            show_mem(&args);
            return ();
        } else {
            let filename_write = args.outfile.as_str();
            write_datafile(
                filename_write,
                Qar_userdata.PrmHeader.clone(),
                Qar_userdata.PrmTable.clone(),
                Qar_userdata.PrmData.clone(),
            );
        }
    }

    show_mem(&args);
}
//对于Linux系统，显示内存占用情况
fn show_mem(args: &CmdLineArgs::Args) {
    #[cfg(target_os = "linux")]
    if args.mem {
        // --begin--查看内存占用(linux)
        //use std::io::{BufRead, BufReader};
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
#[derive(Serialize, Debug, Clone)]
pub struct OneParamTable {
    selfsize: u16,
    data_point: u64,
    data_size: u32,
    val_size: u16,
    rate: i16,
    start_frameid: f32,
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
        let start_frameid = f32::from_le_bytes(u32_bytes);
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
#[derive(Debug, Clone)]
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
fn read_datafile(filename_read: &str) -> Option<(PrmHeader, Vec<OneParamTable>, Vec<Vec<u8>>)> {
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
    //let mut point: u64 = 0;
    let mut table_ii = 0;
    //从ttl_size中减去OneParamTable前面固定长度的22bytes,做判断
    while read_len < parameter_ttl_size - 22 {
        let mut one_parameter_table = OneParamTable::deserialize(&mut rfile)
            .expect(format!("index {} OneParamTable 反序列化失败.", table_ii).as_str());
        //PrmTable的顺序和PrmData的顺序是一样的, 重置为index,后续没有使用这个值
        one_parameter_table.data_point = table_ii; //重置data指针为数组index
                                                   //one_parameter_table.data_point = point; //重置data指针
                                                   //point += one_parameter_table.data_size as u64;
        read_len += one_parameter_table.selfsize as u32;
        PrmTable.push(one_parameter_table);
        /*
        if table_ii < 1 || parameter_ttl_size - read_len < 160 {
            println!("read:{}, ttl:{}", read_len, parameter_ttl_size);
        }
        */
        table_ii += 1;
    }
    let mut PrmData: Vec<Vec<u8>> = vec![];
    let mut data_ttl_size = 0;
    for one_prm in &PrmTable {
        data_ttl_size += one_prm.data_size;
        let mut buf_bytes = vec![0; one_prm.data_size as usize];
        rfile.read_exact(&mut buf_bytes).unwrap();
        PrmData.push(buf_bytes);
    }
    println!(
        "Header size:{}, Parameter count:{}, Parameter table size:{}, DATA count:{}, DATA total size:{}",
        PrmHeader.length(),
        PrmTable.len(),
        read_len,
        PrmData.len(),
        data_ttl_size
    );
    return Some((PrmHeader, PrmTable, PrmData));
}
fn write_datafile(
    filename_write: &str,
    mut PrmHeader: PrmHeader,
    mut PrmTable: Vec<OneParamTable>,
    PrmData: Vec<Vec<u8>>,
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
    for one_data in PrmData {
        wfile.write_all(&one_data).expect("写入失败");
    }
}
fn showHelp(bin_name: String) {
    println!(
        r##" Usage: {bin_name} [-f datafile.dat] [-w outfile.dat] [-h | --help]
    Detail:
      -h         简略的命令行帮助
      --luahelp  显示lua的使用帮助,内置的function
      -l, --lua /path/file.lua   需要执行的lua脚本
      -f /path/datafle.dat       指定 读取的文件
      -w /path/outfile.dat       指定 写入的文件
           自定义格式的datafile.dat, outfile.dat文件,可以用ALL_read_datafile.py读取,并导入pd.DataFrame()
      --mem        打印内存占用情况
  说明: 
    读取 自定义格式的数据文件。
    执行lua脚本。
    写入新的 自定义格式的数据文件。
       author: osnosn@126.com"##
    );
}
fn showLuaHelp(_bin_name: String) {
    let buf = r##"
 Luajit-5.2
   table=json.decode(str)
       把json字符串, 解码为lua的table,失败:会抛出错误
   str=json.encode(table, pretty)
       把lua的table, 转化为json字符串, pretty=true显示更漂亮
       支持 json.null值, json.array()函数.
   str=qar.meta
       获取/设置Header.meta信息 (json字符串格式), 此属性,可读,可写
   str=qar:get_meta()  OR  qar.get_meta(qar)
       获取Header.meta信息 (json字符串格式)
   str=qar:get_meta()  OR  qar.get_meta(qar)
       设置Header.meta信息 (json字符串格式)
   num=qar:get_param_num()
       获取所有参数的数量
   table=qar:get_param_list()
       获取所有的参数名, 如无参数名,返回空table []
   table=qar:get_param(name) OR  qar.get_param(qar,name)
       获取参数名name的所有值, 如果没找到参数,返回nil
   stat,txt=qar:set_param(name,data) OR  qar.set_param(qar,name,data)
       设置参数名name的值, 如果参数名存在则覆盖,不存在则创建, 成功stat>=1,失败stat<=-1, txt=失败的原因
         默认用bzip2压缩保存.
         data中的info,建议用json字符串.
       值为str类型:
          data={ info='{"msg":"test"}', val='[[0,"v12"],[0.5,"v23"],[1,"vv34"], ...]' }  --val建议用json字符串
       值为float,int类型.
          参数的rate是通过第一组值和第二组值中,第一个值的差值来计算的. 后续组中的第一个值被忽略.
          data={ info='{"msg":"test"}', val={ {0.0,10.1},{0.5,10.2},{1.0,10.3},{1.5,10.4}, ... } }  --rate通过0.0, 0.5计算
          data={ info='{"msg":"test"}', val={ {0,101},{1,102},{2,103},{3,104}, ... } }  --rate通过0, 1计算
   stat,txt=qar:del_param(name) OR  qar.del_param(qar,name)
       删除参数名name的值, 成功stat=1,失败stat<=-1, txt=失败的原因
"##;
    println!("{}", buf);
}
