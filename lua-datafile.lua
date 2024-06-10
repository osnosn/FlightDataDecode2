--[[  --块注释
io.write('lua:') --不换行输出
print() --带换行的输出
]]

function GG()
   print("---G----")
   for k,v in pairs(_G) do --打印全局table
      print(string.format("%s=%s, ",k,v)) --换行输出
   end
   print("---G----")
end

function getpm(pm)
   PM=qar:get_param(pm)
   if PM ~= nil then
      print(PM.rate,PM.type,PM.info)
      if type(PM.val) == 'table' then
         for k,v in pairs(PM.val) do print(string.format("%s=%s,\t%s ",k,v[1],v[2])) end
      else
         print(PM.val)
      end
   else
      print(pm,"NotFound")
   end
end

print("VERSION=",_VERSION)

print("---qar----")
print(qar.meta)
meta=json.decode(qar.meta)
meta.MetaData.FileName="123456789/12345"
--print(json.encode(meta,true))
meta_new=json.encode(meta)
print(meta_new)
--qar.meta=meta_new
--qar:set_meta(meta_new)

--[[
LIST=qar:get_param_list()
print(LIST)
for k,v in pairs(LIST) do print(string.format("%s=%s,\t%s ",k,v[1],v[2])) end
]]

--os.execute("sleep 10")

--[[
isok,result=pcall(json.decode,'sadf["test"]')
print('try:',isok,result,'aa')
]]

--getpm("VRTG")

--[[
getpm("FLINUM1234")
getpm("FLINUM5678")
getpm("FLINUM_1")
getpm("FLINUM_2")
getpm("FLINUM_3")
getpm("FLINUM_4")
getpm("FLINUM_5")
getpm("FLINUM_6")
getpm("FLINUM_7")
getpm("FLINUM_8")
]]
getpm("FLINUM_1")

print("==modify param==")
print('param_num:',qar:get_param_num())
data={
   info='{"test":123}',
   val={
      {0, 557},
      {0.2, 123.0},
      {0.4, 123.02},
      {0.3, 99.004},
   }
}
print(data.val[1][1],data.val[4][2])
print('set_param:',qar:set_param("abcTEST",data))
print('param_num:',qar:get_param_num())

--[[
print("==delete param==")
print('param_num:',qar:get_param_num())
print('remove',qar:del_param("abcTEST",data))
print('param_num:',qar:get_param_num())
]]

print("---end----")
