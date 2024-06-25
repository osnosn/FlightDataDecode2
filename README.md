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
  - `qar_decode5` 解1024.PRM. (源码保留,不编译)   
  - `qar_decode7` 解320.PRM. (源码保留,不编译)   
  - `qar_decode8` 解320.PRM, **配置来自json文件。解码所有参数.**(2024-05)   
    - 从`prm_conf320.json`文件中读取解码配置。   
      - 对于PRM配置,可以用`read_prm717.py`生成json配置文件。   
      - 对于VEC配置,可以用另一个项目中[FlightDataDecode](https://github.com/osnosn/FlightDataDecode)`/ARINC717/VEC717_to_json.py`生成json配置文件。   
    - 解码参数后, 写入自定义格式文件.dat。可以用 `ALL_read_datafile.py` 来读取,导入pandas.DataFrame中。   
    - 解码程序没写完。处理了 BNR,ISO,BCD,DIS 格式的数据。其他类型还没有处理 (默认按BNR处理)。   
    - 这个程序的解码逻辑,写的不好。应该要重写。(2024-06)   
  - `qar_decode9` 按一个Frame为单位(包含4个subFrame),进行解码。在decode8基础上重写。(2024-06)   
    - 使用json格式的解码配置。   
    - 解码参数后, 写入自定义格式文件.dat。可以用 `ALL_read_datafile.py` 来读取,导入pandas.DataFrame中。   
    - 解码程序。处理了 BNR,ISO,BCD,DIS,UTC,CHAR 格式的数据。其他类型 (默认按BNR处理)。   
* qar_decode_lua. 支持嵌入lua脚本.    
  - `qar_decode6` 嵌入lua脚本测试, 解码个别参数. (源码保留,不编译)   
  - `qar_datafile2` **读取自定义格式文件,通过嵌入lua脚本,修改自定义格式文件.**   
    - 执行lua脚本, 可 调取,修改,删除 参数的值.   
    - lua中支持的内嵌函数, 请看`qar_datafile2 --luahelp`   
      或者看`qar_datafile2`源码, 在`qar_decode_lua/src/bin/qar_datafile2.rs`   

data/ 目录，有示例数据。  
python3/ 目录，有几个 py3 程序。其中:   
* `ALL_read_datafile.py` 用于**读取**, 存放于自定义格式文件中的,解码后的参数, **并导入pandas.DataFrame中**。(2024-05)   

## 更新  
* **2024-06 最后更新**   
  - rust 程序  
  - `python3/read_prm717.py` 注释中有 **PRM 配置文件 字段的含义** (大部分)。  
  - `python3/decode8_arinc717_aligned.py` 解所有参数, 用命令行参数指定 "解码配置" 和 "原始数据文件"   
    这个py程序旧了。对`qar_decode8`的后续修改, 没有同步修改这个py程序。   
  - `qar_decode9` 解所有参数, 用命令行参数指定 "解码配置" 和 "原始数据文件".   
  - `qar_datafile2` 读取自定义格式文件,通过嵌入lua脚本,修改自定义格式文件.   
  - `ALL_read_datafile.py` 读取解码后的自定义格式文件中的参数, 并导入pandas.DataFrame中   
  - `bitstream2aligned.py`, 把bitstream格式转换为aligned格式, 并把数据帧对齐。(补帧未实现)   

## 数据处理的流程   
本项目, 没打算做成一个产品, 只是一个指引。   
顺便, 我自己也要用一下。   
所以, 本项目是可以用的。大部分的参数,解码都是正确的。   
希望, 让有兴趣的公司或个人, 有信心自己做解码。因为解码并不是那么的难。   

### ARINC717   
1. 获取未解码的原始数据。   
2. 判断格式，Bitstream OR Aligned.    
   用`dump_raw_bitstream`,`dump_raw_aligned`分别扫描原始数据。   
   或`dump_rawdat_bitstream.py`,`dump_rawdat_aligned.py`功能一样。    
   如果是bitstream则下一步，如果是aligned则跳过下一步。   
3. 用`bitstream2aligned.py`, 把bitstream格式转换为aligned格式, 并把数据帧对齐。(补帧未实现)   
   如果发现有帧损坏, 则用空白数据补齐这个损坏的帧。如果有缺帧, 则补空白帧。   
4. 用`read_prm717.py`把PRM配置,改写为json配置文件。   
   或用`VEC717_to_json.py`把VEC配置, 改写为json配置文件。   
   为下一步做准备。   
5. 用`qar_decode9`依据上一步的json配置, 解码所有参数, 写入全参文件.   
   或用`decode8_arinc717_aligned.py`, 功能一样,也是解码所有参数, 写入全参文件.   
6. 用`qar_datafile2` 读取全参文件,通过嵌入lua脚本的执行,修改全参文件.   
   比如: 修改Meta信息; 做飞行阶段的划分; 增加简单的计算参数; 判断简单的超限,生成超限事件; ...    
   以新增参数的方式, 加入到全参文件中。   
   这一步, 还需要对部分跳变的,异常的参数值做修正处理。   
7. 用`ALL_read_datafile.py`读取全参文件, 做复杂的分析处理.   
   这一步, 用python3, 是因为这个语言比较有优势。   


## 其他  
* 认为此项目对您有帮助，请点个星星，或留个言，或发封邮件给我，让我高兴一下.   
  If you think this project is helpful to you, click a Star, or leave a message, or send me an Email to make me happy.  


