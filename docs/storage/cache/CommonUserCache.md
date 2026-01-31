# CommonUserCache

CommonUserCache是一个实现了UserCache trait的对象。它的字段如下:

- client
  client为redis的客户端

# 设置用户基础

向redis中的键名为users的map中写入用户数据序列化成的json对象。
