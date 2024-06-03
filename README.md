# Flight Data Decode 2   

[FOQA (Flight Operations Quality Asurance)](http://en.wikipedia.org/wiki/Flight_operations_quality_assurance)  

Flight Data Decode 2, 解析,解码,译码 原始QAR数据 raw.dat 文件。ARINC 429 573 717. FOQA, arinc429, arinc717. Airfase.  

这个项目，只是些测试程序，基于airfase 导出的 PRM 配置文件。   
尝试解码 原始文件过程中，编写的测试程序。   
**目前可以对 ARINC 717 Aligned 格式的文件，解码出部分的记录参数。**   

当前目录是 rust workspace 的根目录。包含几个子项:   
* qar_raw_dump   
  - `dump_raw_aligned` 扫描raw文件，通过 sync 同步字出现的顺序和间隔，确定是否 aligned格式。   
  - `dump_raw_bitstream` 扫描raw文件，通过 sync 同步字出现的顺序和间隔，确定是否 bitstream格式。   
* qar_decode.  解码个别参数。   
  - `qar_decode5` 解1024.PRM   
  - `qar_decode7` 解320.PRM   
  - `qar_decode8` 解320.PRM, 配置来自json文件。(2024-05)   
    - 从`prm_conf320.json`文件中读取解码配置。   
      - 对于PRM配置,可以用`read_prm717.py`生成json配置文件。   
      - 对于VEC配置,可以用另一个项目中[FlightDataDecode](https://github.com/osnosn/FlightDataDecode)`/ARINC717/VEC717_to_json.py`生成json配置文件。   
    - 解码参数后, 写入自定义格式文件.dat。可以用 `ALL_read_datafile.py` 来读取,导入pd.DataFrame中。   
    - 解码程序没写完。处理了 BNR,ISO,BCD,DIS 格式的数据。其他类型还没有处理 (默认按BNR处理)。   
* qar_decode_lua.  解码个别参数。   
  - `qar_decode6` 嵌入lua脚本测试   

data/ 目录，有示例数据。  
python3/ 目录，有几个 py3 程序。  

### 更新  
* **2024-05 最后更新**   
  - rust 程序  
  - `python3/read_prm717.py` 注释中有 **PRM 配置文件 字段的含义** (大部分)。  
  - `python3/decode8_arinc717_aligned.py` 解所有参数, 用命令行参数指定 "解码配置" 和 "原始数据文件"   
  - `qar_decode8` 解所有参数, 用命令行参数指定 "解码配置" 和 "原始数据文件"   
  - `ALL_read_datafile.py` 读取解码后的参数文件, 并导入pd.DataFrame中   

### 其他  
* 认为此项目对您有帮助，请点个星星，或留个言，或发封邮件给我，让我高兴一下.   
  If you think this project is helpful to you, click a Star, or leave a message, or send me an Email to make me happy.  


