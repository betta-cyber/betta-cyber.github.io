---
title: MySQL Quick Reference
abbrlink: abc4c21
date: 2023-08-30 09:00:00
---


### 1. MySQL 解析 **json**

``` sql
select sum(json_extract(a.sec_count, "$.Critical")) from `result` AS a
where
  task_id > 20220931000999
  and task_id < 20221231000999
  and a.sec_count != '';
```


- 使用 字段->'$.json属性'进行查询条件
- 使用json_extract函数查询，json_extract(字段,"$.json属性")
- 根据json数组查询，用JSON_CONTAINS(字段,JSON_OBJECT('json属性', "内容"))


容易出现的错误是，**如果该字段为空或者空字符串**，会出现解析不了的情况，要注意好判断条件。


### 2. MySQL varchar 转 int 类型: **CAST** 函数

在使用CAST函数转换类型时，可以转换的类型是有限制的。这个类型可以是以下值其中的一个。也就是说，UNSIGNED 可以替换成：

二进制，同带binary前缀的效果 : BINARY
字符型，可带参数 : CHAR()
日期 : DATE
时间: TIME
日期时间型 : DATETIME
浮点数 : DECIMAL
整数 : SIGNED
无符号整数 : UNSIGNED

``` sql
select max(cast(sex as UNSIGNED INTEGER)) from user;
```
