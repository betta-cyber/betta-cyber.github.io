---
title: OpenResty Note
abbrlink: fcfbe98c
date: 2020-12-12 12:48:00
---

# 概念

## lua-resty-core

在核心的 lua-nginx-module 中，调用 C 函数的 API，都是使用 Lua C API 来完成的；
而在 lua-resty-core 中，则是把 lua-nginx-module 已有的部分 API，使用 FFI 的模式重新实现了一遍。

LuaJIT 只负责由自己分配的资源；而 ffi.C 是 C 库的命名空间，
所以，使用 ffi.C 分配的空间不由 LuaJIT 负责，需要你自己手动释放。`

FFI 的方式不仅代码更简洁，而且可以被 LuaJIT 优化，显然是更优的选择。其实现实也是如此，
实际上，CFunction 的实现方式已经被 OpenResty 废弃，相关的实现也从代码库中移除了。现在新的 API，都通过 FFI 的方式，在 lua-resty-core 仓库中实现。

## NYI

编译器的死穴。

LuaJIT 的运行时环境，除了一个汇编实现的 Lua 解释器外，还有一个可以直接生成机器代码的 JIT 编译器。

LuaJIT 中 JIT 编译器的实现还不完善，有一些原语它还无法编译，因为这些原语实现起来比较困难，再加上 LuaJIT 的作者目前处于半退休状态。这些原语包括常见的 pairs() 函数、unpack() 函数、基于 Lua CFunction 实现的 Lua C 模块等。这样一来，当 JIT 编译器在当前代码路径上遇到它不支持的操作时，便会退回到解释器模式。

完整的[NYI列表](http://wiki.luajit.org/NYI)

``` bash
resty -j v -e 'local t = {}
for i=1,100 do
    t[i] = i
end

for i=1, 1000 do
    for j=1,1000 do
        for k,v in pairs(t) do
            --
        end
    end
end'
```
output

``` bash
[TRACE   1 (command line -e):2 loop]
[TRACE --- (command line -e):7 -- NYI: bytecode 72 at (command line -e):8]
[TRACE --- (command line -e):7 -- NYI: bytecode 72 at (command line -e):8]
[TRACE --- (command line -e):7 -- NYI: bytecode 72 at (command line -e):8]
[TRACE --- (command line -e):7 -- NYI: bytecode 72 at (command line -e):8]
[TRACE --- (command line -e):7 -- NYI: bytecode 72 at (command line -e):8]
[TRACE --- (command line -e):7 -- NYI: bytecode 72 at (command line -e):8]
[TRACE --- (command line -e):7 -- NYI: bytecode 72 at (command line -e):8]
[TRACE --- (command line -e):7 -- NYI: bytecode 72 at (command line -e):8]
[TRACE --- (command line -e):7 -- NYI: bytecode 72 at (command line -e):8]
[TRACE --- (command line -e):7 -- NYI: bytecode 72 at (command line -e):8]
[TRACE --- (command line -e):7 -- NYI: bytecode 72 at (command line -e):8]
[TRACE --- (command line -e):6 -- NYI: bytecode 72 at (command line -e):8]
[TRACE --- (command line -e):6 -- NYI: bytecode 72 at (command line -e):8]
[TRACE --- (command line -e):6 -- NYI: bytecode 72 at (command line -e):8]
[TRACE --- (command line -e):6 -- NYI: bytecode 72 at (command line -e):8]
[TRACE --- (command line -e):6 -- NYI: bytecode 72 at (command line -e):8]
```

If could be JIT:

``` bash
resty -j v -e 'for i=1, 1000 do local newstr, n, err = ngx.re.gsub("hello, world", "([a-z])[a-z]+", "[$0,$1]", "i") end'
```

output:

``` bash
[TRACE   1 regex.lua:1081 loop]
[TRACE --- (command line -e):1 -- inner loop in root trace at regex.lua:1082]
[TRACE   2 (1/10) regex.lua:1116 -> 1]
[TRACE   3 (1/21) regex.lua:1084 -> 1]
```
every code could be traced


## Table

## string

千万不要在热代码路径上拼接字符串：

``` lua
--No
local s = ""
for i = 1, 100000 do
    s = s .. "a"
end
```


``` lua
--Yes
local t = {}
for i = 1, 100000 do
    t[i] = "a"
end
local s =  table.concat(t, "")
```

## 编码方式

为了风格的统一，require 和 ngx 也需要 local 化：

``` lua
local ngx = ngx
local require = require
```

## 常见的坑

### 下标从 0 开始还是从 1 开始

第一个坑，Lua 的下标是从 1 开始的

在 LuaJIT 中，使用 ffi.new 创建的数组，下标又是从 0 开始的:

``` lua
local buf = ffi_new("char[?]", 128)
```

所以，如果你要访问上面这段代码中 buf 这个 cdata，请记得下标从 0 开始，而不是 1。


## luacheck

[luacheck](https://github.com/mpeterv/luacheck)

install luacheck by luarocks

``` bash
luarocks install luacheck
```

### usage

``` bash
luacheck src extra_file.lua another_file.lua
```

# API

## 请求行

[完整的 NGINX 内置变量列表](http://nginx.org/en/docs/http/ngx_http_core_module.html#variables)

既然可以通过ngx.var.* 这种返回变量值的方法，来得到请求行中的数据，为什么 OpenResty 还要单独提供针对请求行的 API 呢？

比如ngx.req.get_method

这其实是很多方面因素的综合考虑结果：
- 首先是对性能的考虑。ngx.var 的效率不高，不建议反复读取；
- 也有对程序友好的考虑，ngx.var 返回的是字符串，而非 Lua 对象，遇到获取 args 这种可能返回多个值的情况，就不好处理了；
- 另外是对灵活性的考虑，绝大部分的 ngx.var 是只读的，只有很少数的变量是可写的，比如 $args 和 limit_rate，可很多时候，我们会有修改 method、URI 和 args 的需求。

## 请求头

改写和删除请求头

``` c
ngx.req.set_header("Content-Type", "text/css")
ngx.req.clear_header("Content-Type")
```

## 响应状态行

状态行中，我们主要关注的是状态码。在默认情况下，返回的 HTTP 状态码是 200，也就是 OpenResty 中内置的常量 ngx.HTTP_OK。但在代码的世界中，处理异常情况的代码总是占比最多的。如果你检测了请求报文，发现这是一个恶意的请求，那么你需要终止请求:

``` c
ngx.exit(ngx.HTTP_BAD_REQUEST)
```

不过，OpenResty 的 HTTP 状态码中，有一个特别的常量：ngx.OK。当 ngx.exit(ngx.OK) 时，请求会退出当前处理阶段，进入下一个阶段，而不是直接返回给客户端。

ngx.OK 确实不是http状态码，它是 OpenResty 中的一个常量，值是0.
ngx.exit 的官方文档正好可以解答你的问题：
```
When status >= 200 (i.e., ngx.HTTP_OK and above), it will interrupt the execution of the current request and return status code to nginx.

When status == 0 (i.e., ngx.OK), it will only quit the current phase handler (or the content handler if the content_by_lua* directive is used) and continue to run later phases (if any) for the current request.
```

不过，里面并没有提到对于ngx.exit(ngx.ERROR)和ngx.exit(ngx.DECLINED)是如何处理的，我们可以自己来做个测试：
``` nginx
location /lua {
    rewrite_by_lua "ngx.exit(ngx.ERROR)";
    echo hello;
}
```
访问这个 location，可以看到 http 响应码为空，响应体也是空。并没有引入下一个执行阶段。

[更多状态行](https://github.com/openresty/lua-nginx-module/#http-status-constants)


## 数据共享

基于 shared dict，你可以实现多个 worker 之间的缓存和通信，以及限流限速、流量统计等功能。你可以把 shared dict 当作简单的 Redis 来使用，只不过 shared dict 中的数据不能持久化，所以你存放在其中的数据，一定要考虑到丢失的情况。

[sharddict文档](https://github.com/openresty/lua-nginx-module#ngxshareddict)

## cosocket

cosocket 是 OpenResty 中的专有名词，是把协程和网络套接字的英文拼在一起形成的，即 cosocket = coroutine + socket。所以，你可以把 cosocket 翻译为“协程套接字”。

![cosocket](https://static001.geekbang.org/resource/image/80/06/80d16e11d2750d6e4127445c126c9f06.png)

从图中你可以看到，用户的 Lua 脚本每触发一个网络操作，都会有协程的 yield 以及 resume。遇到网络 I/O 时，它会交出控制权（yield），把网络事件注册到 Nginx 监听列表中，并把权限交给 Nginx；当有 Nginx 事件达到触发条件时，便唤醒对应的协程继续处理（resume）。

OpenResty 正是以此为蓝图，封装实现 connect、send、receive 等操作，形成了我们如今见到的 cosocket API。

### cosocket API 和指令简介

TCP 相关的 cosocket API 可以分为下面这几类。

- 创建对象：ngx.socket.tcp。
- 设置超时：tcpsock:settimeout 和 tcpsock:settimeouts。
- 建立连接：tcpsock:connect。
- 发送数据：tcpsock:send。
- 接受数据：tcpsock:receive、tcpsock:receiveany 和 tcpsock:receiveuntil。
- 连接池：tcpsock:setkeepalive。
- 关闭连接：tcpsock:close。

## privileged process

``` lua
init_by_lua_block {
    local process = require "ngx.process"

    local ok, err = process.enable_privileged_agent()
    if not ok then
        ngx.log(ngx.ERR, "enables privileged agent failed error:", err)
    end
}
```
特权只在 init_worker_by_lua 阶段运行一次，既然它不监听端口，也就是不能被终端请求触发，那就只有使用我们刚才介绍的 ngx.timer ，来周期性地触发了


example
``` lua
init_worker_by_lua_block {
    local process = require "ngx.process"

    local function reload(premature)
        local f, err = io.open(ngx.config.prefix() .. "/logs/nginx.pid", "r")
        if not f then
            return
        end
        local pid = f:read()
        f:close()
        os.execute("kill -HUP " .. pid)
    end

    if process.type() == "privileged agent" then
         local ok, err = ngx.timer.every(5, reload)
        if not ok then
            ngx.log(ngx.ERR, err)
        end
    end
}
```
## 非阻塞的 ngx.pipe

``` bash
os.execute("kill -HUP " .. pid)
```
会导致阻塞，这显然是不好的。

为此，lua-resty-shell 库应运而生，使用它来调用命令行就是非阻塞的：

## sleep

这些返回当前时间的 API，如果没有非阻塞网络 IO 操作来触发，便会一直返回缓存的值，而不是像我们想的那样，能够返回当前的实时时间。

``` bash
$ resty -e 'ngx.say(ngx.now())
os.execute("sleep 1")
ngx.say(ngx.now())'
```

在两次调用 ngx.now 之间，我们使用 Lua 的阻塞函数 sleep 了 1 秒钟，但从打印的结果来看，这两次返回的时间戳却是一模一样的。

如果换成是非阻塞的 sleep 函数

``` bash
$ resty -e 'ngx.say(ngx.now())
ngx.sleep(1)
ngx.say(ngx.now())'
```

显然，它就会打印出不同的时间戳了。

Nginx 是以性能优先作为设计理念的，它会把时间缓存下来。从 ngx.now 的源码中我们可以得到印证：
``` c
static int
ngx_http_lua_ngx_now(lua_State *L)
{
    ngx_time_t *tp;

    tp = ngx_timeofday();

    lua_pushnumber(L, (lua_Number) (tp->sec + tp->msec / 1000.0L));

    return 1;
}
```


是调用了 Nginx 中的 ngx_timeofday 函数获取的时间。
而这个函数其实是一个宏定义：
``#define ngx_timeofday() (ngx_time_t *) ngx_cached_time`

而 ngx_cached_time 的值只在函数 ngx_time_update 中会更新。
那问题就简化为 ngx_time_update 什么时候会被调用。如果你在 Nginx 的源码中去跟踪它的话，就会发现ngx_time_update的调用比较多，在事件循环中都有出现。

也就是说只有在调用了 ngx_timer_update 的时候，ngx.timer的值才会更新，而调用前者多是在事件循环中，而调用yield 函数通常是添加了一个事件。这样解释了需要yield操作之后，ngx.timer才会更新

这里顺带引出了 ngx.sleep ，这个非阻塞的 sleep 函数。这个函数除了可以休眠指定的时间外，还有另外一个特别的用处。

举个例子，比如你有一段正在做密集运算的代码，需要花费比较多的时间，那么在这段时间内，这段代码对应的请求就会一直占用着 worker 和 CPU 资源，导致其他请求需要排队，无法得到及时的响应。这时，我们就可以在其中穿插 ngx.sleep(0)，使这段代码让出控制权，让其他请求也可以得到处理。

## nginx.null

*只有 nil 和 false 是假值*

## CVE-2018-9230

OpenResty 中的 ngx.req.get_uri_args、ngx.req.get_post_args 和 ngx.req.get_headers接口，默认只返回前 100 个参数。如果 WAF 的开发者没有注意到这个细节，就会被参数溢出的方式攻击。攻击者可以填入 100 个无用参数，把 payload 放在第 101 个参数中，借此绕过 WAF 的检测。

最终，OpenResty 维护者选择新增一个 err 的返回值来解决这个问题。如果输入参数超过 100 个，err 的提示信息就是 truncated。这样一来，这些 API 的调用者就必须要处理错误信息，自行判断拒绝请求还是放行。

ngx.req.get_uri_args(max_args?) 其实是有参数的。默认为100个。官方建议设置为0，可以接收所有的参数。但是不建议设置为0，因为这样会造成大量的系统占用。造成DDOS攻击。

*返回函数有错误，一定要做错误处理!!!*

## 变量竞争问题

关于变量竞争的问题，其实，只要两个操作之间有 yield 操作，就可能出现竞争，而不是阻塞操作；有阻塞操作时是不会出现竞争的。换句话说，只要你不把主动权交给 Nginx 的事件循环，就不会有竞争。

# 测试

## test::nginx

“自动化测试”和“持续集成”

test::nginx 糅合了 Perl、数据驱动以及 DSL（领域小语言）。对于同一份测试案例集，通过对参数和环境变量的控制，可以实现乱序执行、多次重复、内存泄漏检测、压力测试等不同的效果。

推荐 travis 中安装，其他方式的安装总是会遇到各种各样的问题。具体参考[这里](https://time.geekbang.org/column/article/109506)

# 优化

## string

理念一：处理请求要短、平、快

- 这里提到的“短”，是指请求的生命周期要短，不要长时间占用资源而不释放；即使是长连接，也要设定一个时间或者请求次数的阈值，来定期地释放资源。
- 第二个字“平”，则是指在一个 API 中只做一件事情。要把复杂的业务逻辑拆散为多个 API，保持代码的简洁。
- 最后的“快”，是指不要阻塞主线程，不要有大量 CPU 运算。即使是不得不有这样的逻辑，也别忘了咱们上节课介绍的方法，要配合其他的服务去完成。

理念二：避免产生中间数据

## 用好 table

尽量复用，避免不必要的 table 创建。

### 预先生成数组

``` lua
local new_tab = require "table.new"
local t = new_tab(100, 0)
for i = 1, 100 do
  t[i] = i
end
```

### 自己计算 table 下标

lua-resty-redis example

``` lua
local function _gen_req(args)
    local nargs = #args


    local req = new_tab(nargs * 5 + 1, 0)
    req[1] = "*" .. nargs .. "\r\n"
    local nbits = 2


    for i = 1, nargs do
        local arg = args[i]
        req[nbits] = "$"
        req[nbits + 1] = #arg
        req[nbits + 2] = "\r\n"
        req[nbits + 3] = arg
        req[nbits + 4] = "\r\n"
        nbits = nbits + 5
    end
    return req
end
```
### 循环使用单个 table

``` lua
local local_plugins = {}

function load()
    core.table.clear(local_plugins)


    local local_conf = core.config.local_conf()
    local plugin_names = local_conf.plugins


    local processed = {}
    for _, name in ipairs(plugin_names) do
        if processed[name] == nil then
            processed[name] = true
            insert_tab(local_plugins, name)
        end
    end


    return local_plugins
```

### table 池

lua-tablepool 官方库

``` lua
local tablepool = require "tablepool"
local tablepool_fetch = tablepool.fetch
local tablepool_release = tablepool.release

local pool_name = "some_tag"
local function do_sth()
     local t = tablepool_fetch(pool_name, 10, 0)
     -- -- using t for some purposes
    tablepool_release(pool_name, t)
end
```
