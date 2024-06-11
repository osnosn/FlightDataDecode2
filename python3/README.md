# Flight Data Decode 2   

Flight Data Decode 2, 解析,解码,译码 原始QAR数据 raw.dat 文件。ARINC 429 573 717. FOQA, arinc429, arinc717. Airfase.  

这个项目，只是些测试程序，基于airfase 导出的 PRM 配置文件。   
尝试解码 原始文件过程中，编写的测试程序。   
**用于 ARINC 717 Aligned 格式的文件。**   

当前目录是 python3 程序目录。  
* `read_prm717.py`   
  - 注释中有 **PRM 配置文件 字段的含义** (大部分)。  
  - 此程序读取PRM文件，显示指定参数的配置内容。   
  - 读取PRM文件，导出json格式的自定义解码配置文件。(2024-05)   
* `ALL_read_datafile.py` 用于读取, 存放于自定义格式文件中的,解码后的参数, 并导入pandas.DataFrame中。(2024-05)   
* `dump_rawdat_aligned.py` 用于扫描raw文件，通过 sync同步字出现的顺序和间隔，确定是否 aligned格式。   
* `dump_rawdat_bitstream.py` 用于扫描raw文件，通过 sync同步字出现的顺序和间隔，确定是否 bitstream格式。  
* `decode5_arinc717_aligned.py` 解320.PRM, raw320.dat   
  - 用 `param_prm5.py` 解码配置320.PRM   
* `decode6_arinc717_aligned.py` 解单个参数
  - 用命令行参数指定 "解码配置" 和 "原始数据文件"   
* `decode8_arinc717_aligned.py` 解单个参数, 或所有参数
  - 用命令行参数指定 "解码配置" 和 "原始数据文件"   
  - 解所有参数, 写入自定格式的单文件   

## 其他  
* 认为此项目对您有帮助，请点个星星，或留个言，或发封邮件给我，让我高兴一下.   
  If you think this project is helpful to you, click a Star, or leave a message, or send me an Email to make me happy.  


