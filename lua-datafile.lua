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
for k,v in pairs(_G) do --打印全局table
print(string.format("%s=%s, ",k,v)) --换行输出
end
