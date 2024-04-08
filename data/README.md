# Flight Data Decode 2   

Flight Data Decode 2, 解析,解码,译码 原始QAR数据 raw.dat 文件。ARINC 429 573 717. FOQA, arinc429, arinc717. Airfase.  

当前目录是 数据目录。  
* `1024.PRM`,`320.PRM` 截取的配置文件。从 Airfase中导出的。  
* `output*`  qar_decode 的输出。  
* `raw.dat`  截取的原始数据，aligned格式。对应的解码配置是1024.PRM。   
* `raw320.dat`  截取的原始数据，aligned格式。对应的解码配置是320.PRM。   
* `bitstream_1214.DLU`  截取的原始数据，bitstream格式。没能找到对应的解码配置。   

从airfase中，可以导出4种文件。FAP, Frame, PRM, FRED.   
* FAP, Frame 文件是加密的，无法解开。   
* PRM 是个文本文件。内容为参数解码配置。   
* FRED 没见过, 不知道内容。   

如图:   
<img src="https://github.com/osnosn/FlightDataDecode2/raw/main/data/airfase-PRM-header.png" width="300" />   
   ----  图1  ----   
<img src="https://github.com/osnosn/FlightDataDecode2/raw/main/data/airfase-regular.png" width="500" />   
   ----  图2  ----   
<img src="https://github.com/osnosn/FlightDataDecode2/raw/main/data/airfase-superframe.png" width="500" />   
   ----  图3  ----   

如果从 AGS 导出解码配置，
* 参考另一个项目 【[osnosn/FlightDataDecode/](https://github.com/osnosn/FlightDataDecode/)】   


### 其他  
* 认为此项目对您有帮助，请点个星星，或留个言，或发封邮件给我，让我高兴一下.   
  If you think this project is helpful to you, click a Star, or leave a message, or send me an Email to make me happy.    


