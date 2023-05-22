---
title: lua-resty-mlcache 分析
abbrlink: 871119db
date: 2023-04-06 01:13:14
---

lua-resty-mlcache 用 shared dict 和 lua-resty-lrucache ，实现了多层缓存机制。

mlcache 的架构如下：

```
┌─────────────────────────────────────────────────┐
│ Nginx                                           │
│       ┌───────────┐ ┌───────────┐ ┌───────────┐ │
│       │worker     │ │worker     │ │worker     │ │
│ L1    │           │ │           │ │           │ │
│       │ Lua cache │ │ Lua cache │ │ Lua cache │ │
│       └───────────┘ └───────────┘ └───────────┘ │
│             │             │             │       │
│             ▼             ▼             ▼       │
│       ┌───────────────────────────────────────┐ │
│       │                                       │ │
│ L2    │           lua_shared_dict             │ │
│       │                                       │ │
│       └───────────────────────────────────────┘ │
│                           │ mutex               │
│                           ▼                     │
│                  ┌──────────────────┐           │
│                  │     callback     │           │
│                  └────────┬─────────┘           │
└───────────────────────────┼─────────────────────┘
                            │
  L3                        │   I/O fetch
                            ▼

                   Database, API, DNS, Disk, any I/O...
```

分为三层：

L1 层是使用 lua-resty-lrucache 的虚拟缓存。提供最快的查找。

L2 层是用的 Nginx 的 lua_shared_dict 内存共享。当 L1 层的 key miss 掉时，从这里获取。

L3 层提供一个自定义函数，有一个 Worker 执行，通过 lua-resty-lock 避免对后端的多次访问。L3 获取的值会被放到 L2 缓存当中。

用来做缓存的组件，shared dict 缓存和 lru 缓存。前者只能缓存字符串对象，缓存的数据有且只有一份，每一个 worker 都可以进行访问，所以常用于 worker 之间的数据通信。后者则可以缓存所有的 Lua 对象，但只能在单个 worker 进程内访问，有多少个 worker，就会有多少份缓存数据。

