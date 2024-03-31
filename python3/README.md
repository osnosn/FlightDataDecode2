# Flight Data Decode 2   

Flight Data Decode 2, 解析,解码,译码 原始QAR数据 raw.dat 文件。ARINC 429 573 717. FOQA, arinc429, arinc717. Airfase.  

这个项目，只是些测试程序，基于airfase 导出的 PRM 配置文件。   
尝试解码 原始文件过程中，编写的测试程序。   
**用于 ARINC 717 Aligned 格式的文件。**   

当前目录是 python3 程序目录。  
* `read_prm.py`   
  - 注释中有 **PRM 配置文件 字段的含义** (大部分)。  
  - 此程序没写完。  
* `dump_rawdat_aligned.py` 用于扫描raw文件，通过 sync同步字出现的顺序和间隔，确定是否 aligned格式。  
* `dump_rawdat_bitstream.py` 用于扫描raw文件，通过 sync同步字出现的顺序和间隔，确定是否 bitstream格式。  

### 其他  
* 认为此项目对您有帮助，请点个星星，或留个言，或发封邮件给我，让我高兴一下.   
  If you think this project is helpful to you, click a Star, or leave a message, or send me an Email to make me happy.  


