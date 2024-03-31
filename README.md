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
  - `qar_decode5`   
  - `qar_decode6` 嵌入lua脚本测试   

data/ 目录，有示例数据。  
python3/ 目录，有几个 py3 程序。  

### 更新  
* **2024-03 最后更新**   
  - rust 程序  
  - python3/read_prm.py 注释中有 **PRM 配置文件 字段的含义** (大部分)。  

### 其他  
* 认为此项目对您有帮助，请点个星星，或留个言，或发封邮件给我，让我高兴一下.   
  If you think this project is helpful to you, click a Star, or leave a message, or send me an Email to make me happy.  


