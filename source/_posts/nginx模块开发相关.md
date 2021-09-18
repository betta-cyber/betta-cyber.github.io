---
title: Nginx 模块开发相关
date: 2021-09-16 10:00:00
---


## 问题由来

第一次碰上 Nginx 模块相关问题的时候，我还是挺难受的。之前一般只写一些基于 Nginx 上面的 Lua 代码开发。没有涉及到相关的 Nginx C层面的比较深的经验，但是问题已经抛出来了，就不得不解决。这段时间我在 TENCENT 做安全开发。其中负责的一个项目是邮件安全网关。简而言之就是对邮件相关的流量进行解析，并赋予相关的安全能力。

这个问题的根本来源是微软爆出的关于 exchange 邮件服务器的安全漏洞，利用已知的 Payload 可以获取到公司内部的员工信息。我们第一时间对问题进行关注，并成功提取出规则，编写 Lua 插件代码，对攻击请求进行过滤，与之同时我们发现了另外一个问题，那就是之前同事修改编译的让 Nginx 支持解析 ntlm 协议的模块存在串 session 问题。为了从根本上解决这个问题。我们有两个方案，一个是在 Lua 层面做 ntlm 协议的解析，另一个就是修复原来存在问题的代码。但是 Nginx 的代码比较复杂，Lua 层面的解密性能又不太高，还要能解决我们之前一直存在 keepalive 问题，我还是决定从 Nginx开始下手。

这其中其实现有的 Nginx 当中其实已经支持了 ntlm 协议，具体可以见[官方文档](http://nginx.org/en/docs/http/ngx_http_upstream_module.html#ntlm)，不过不幸的是，该模块是闭源的。只有使用 Nginx plus 版才享有此功能。

然后还有一个基于 Lua 的[实现](https://github.com/gosp/lua-resty-ntlm)，我没有仔细看他的代码。也看到了我原来同事使用的[实现原理](http://www.hackdig.com/06/hack-376193.htm)。最后我的同事给我看了一个github上面的开源仓库[nginx-ntlm-module](https://github.com/gabihodoroaga/nginx-ntlm-module)。让我用这个试试，当时他感觉应该是这个是作者最近开源出来的。比较靠谱，让我试一下。

于是我就开始了我艰难的 Nginx 模块 debug 之路。

首先这个作者的模块自测的用例是没有什么问题的。他自己做了一个 Node 实现的后端，然后对代码进行测试，问题不存在，但是代码我一部署到线上，就产生大量的 coredump。这一下就把我给搞懵了。

## 问题分析

要解决 coredump ，那就先要生成 coredump 文件。首先要在服务器上把 ulimit 开启。可以用`ulimit -c unlimited`来临时设置一下coredump时的文件大小，设置为 unlimited 。还需要设定 coredump 文件的保存位置与格式。`sysctl -w kernel.core_pattern=/var/crash/core.%u.%e.%p` 可以用 sysctl 命令设置一下。到时候发生 coredump 的时候就会在该目录生成 coredump 文件。 %u，%e，%p 这种都具有特殊的含义，可以查一下文档。

```
Program terminated with signal 11, Segmentation fault.
#0  0x0000000000545f28 in ngx_http_upstream_ntlm_close_handler (ev=<optimized out>)
    at /root/openresty-1.19.3.2/../nginx-ntlm-module/ngx_http_upstream_ntlm_module.c:435
435            ngx_queue_insert_head(&conf->free, &item->queue);
```
可以看到 Nginx 发生了 signal 11, Segmentation fault。

gdb 加载一下 coredump 文件。使用 `bt` 查看调用栈
```
#0  0x00000000004700fa in ngx_http_upstream_handler (ev=0x7fd98103dab0)
    at src/http/ngx_http_upstream.c:1276
#1  0x0000000000442630 in ngx_event_process_posted (cycle=cycle@entry=0x1413ff0, 
    posted=0x7d5dc0 <ngx_posted_events>) at src/event/ngx_event_posted.c:35
#2  0x00000000004421ce in ngx_process_events_and_timers (cycle=cycle@entry=0x1413ff0)
    at src/event/ngx_event.c:274
#3  0x0000000000449142 in ngx_worker_process_cycle (cycle=cycle@entry=0x1413ff0, 
    data=data@entry=0x13) at src/os/unix/ngx_process_cycle.c:811
#4  0x0000000000447ad0 in ngx_spawn_process (cycle=cycle@entry=0x1413ff0, 
    proc=proc@entry=0x4490d0 <ngx_worker_process_cycle>, data=data@entry=0x13, 
    name=name@entry=0x54b0c5 "worker process", respawn=respawn@entry=-3)
    at src/os/unix/ngx_process.c:199
#5  0x0000000000449594 in ngx_start_worker_processes (cycle=cycle@entry=0x1413ff0, n=32, 
    type=type@entry=-3) at src/os/unix/ngx_process_cycle.c:387
#6  0x0000000000449e98 in ngx_master_process_cycle (cycle=cycle@entry=0x1413ff0)
    at src/os/unix/ngx_process_cycle.c:135
#7  0x0000000000422469 in main (argc=<optimized out>, argv=<optimized out>)
    at src/core/nginx.c:385
```

stackoverflow一下，看到有一个相关问题。

```
Three basic rules:
1. Set pointer to NULL after free
2. Check for NULL before freeing.
3. Initialise pointer to NULL in the start.
Combination of these three works quite well.
```

分析一下，大概是引用了空指针问题，为了查看更多的调试信息。把 OpenResty 中的 Nginx 进行 debug 模式编译。

```
./configure --prefix=/home/work/openresty_debug --with-debug --with-http_stub_status_module  --with-http_v2_module --with-stream  --with-stream_realip_module  --with-pcre-jit --with-http_realip_module --add-module=../nginx-ntlm-module
gmake && gmake install
```

可以看到在日志中出现了很多的调试信息了。这里对日志进行排查。发现日志太多，而且复现条件不稳定。不太确定是哪一个触发了 coredump ，而且 coredump 文件每次触发 Segmentation fault 的位置都不太一样。让我感觉有点绝望。

这里还没有仔细琢磨 Nginx 模块的调用流程，就凭感觉再猜测调试，导致浪费了很多时间。

尝试使用 Valgrind memcheck 工具运行你的 Nginx 应用，检查是否有内存问题。在此运行模式下，建议使用下面的命令构造 OpenResty:
```
./configure --with-debug --with-no-pool-patch \
           --with-luajit-xcflags='-DLUAJIT_USE_SYSMALLOC -DLUAJIT_USE_VALGRIND'
```
然后在 valgrind 运行模式下，在 nginx.conf 中作如下配置：
```
worker_processes  1;
daemon off;
master_process off;
```

```
valgrind --tool=memcheck /home/work/openresty_debug/nginx/sbin/nginx -p /home/work/bin/exchange_debug/ -c conf/nginx.conf
```

这种情况启动 Nginx ，再次查看 Nginx ，就舒服多了。然后在模块代码中加入一些调试日志。

Nginx 的调试日志简单分为以下三种：
- ngx_log_error(level, log, err, fmt, ...)
- ngx_log_debug(level, log, err, fmt, ...)
- ngx_log_debugX(level, log, err, fmt, ...)

第1个参数level可选宏如下：
```
#define NGX_LOG_STDERR            0
#define NGX_LOG_EMERG             1
#define NGX_LOG_ALERT             2
#define NGX_LOG_CRIT              3
#define NGX_LOG_ERR               4
#define NGX_LOG_WARN              5
#define NGX_LOG_NOTICE            6
#define NGX_LOG_INFO              7
#define NGX_LOG_DEBUG             8
```
打印等级通过 nginx.conf 中的 error_log 指令可以配置，其中 WARN 以上级别打印会直接打印到 stderr 上，对于某些临时调试场景有意义。

第2个参数 log，用于部分回调功能。

常见获取方式包括
```
[ngx_conf_t].log
[ngx_http_request_t].connection.log
[ngx_connection_t].log
```

第3个参数 err，用于记录 errno ，非系统调用错误，一般使用0即可。

后续参数为可变长的字符串参数，其中针对 ngx 常用的几个类型有特殊定义

|标识符|对应类型|
| ----------- | ----------- |
|%O|off_t|
|%T	|time_t|
|%z	|ssize_t|
|%i	|ngx_int_t|
|%p	|void *|
|%V	|ngx_str_t *|
|%s	|u_char * (null-terminated)|
|%*s	|size_t + u_char *|


后面我还发现，在 valgrind 生成的 vgcore 文件当中，用 gdb 可以直接 print 相关的变量。之前用的少，还不知道，简直想哭。print display 都可以使用。这也极大的帮助我调试。

调试的一切都准备好了，就可以正式开始调试了。nginx-ntlm-module 是基于 keepalive 的源代码开发的。我仔细分析了 keepalive 和 ntlm-module 的源代码，发现在代码中确实存在空指针问题。

## 问题结论

```
for (q = ngx_queue_head(cache); q != ngx_queue_sentinel(cache);
         q = ngx_queue_next(q)) {
        item = ngx_queue_data(q, ngx_http_upstream_ntlm_cache_t, queue);

        if (item->client_connection == hndp->client_connection) {
            c = item->peer_connection;
            ngx_queue_remove(q);
            ngx_queue_insert_head(&hndp->conf->free, q);
            hndp->cached = 1;
            goto found;
        }
    }
```

在这段代码中，c 变量是可能存在空指针的问题的。导致 coredump。

另外一点，在 ngx_http_upstream_client_conn_cleanup 方法中。有一个 `ngx_post_event(item->peer_connection->read,&ngx_posted_events);` 代码，我最后把他给注释掉了，因为据我调试发现，item->peer_connection->read 本身是一个 `ngx_event_t` 对象。但是在 post_event 之后，会尝试去获取它的 data 和 data->log 这个因为 data 本身为 void data，没有赋予到相应的值。会导致空指针。

另外在修复代码当中，因为我的一个小错误。在 close 连接的代码中：
```
ngx_queue_remove(&item->queue);
ngx_queue_insert_head(&conf->free, &item->queue);
```

我把这段代码的顺序给弄反了。导致编译后运行的代码运行一段时间之后 CPU 直接飙升到 100% 。导致后续无法处理请求，这个主要还是跟 Nginx 中的双向链表问题有关。双向链表的遍历，我在查看了 keepalive 的源代码之后，觉得是不太会存在性能问题的，之前我们线上的代码 keepalive 的双向链表设置为 50000 的大小也没出现性能瓶颈。

```
for (q = ngx_queue_head(cache); q != ngx_queue_sentinel(cache);
    q = ngx_queue_next(q)) {
    item = ngx_queue_data(q, ngx_http_upstream_ntlm_cache_t, queue);

    if (item->client_connection == hndp->client_connection) {
        c = item->peer_connection;
        ngx_queue_remove(q);
        ngx_queue_insert_head(&hndp->conf->free, q);
        hndp->cached = 1;
        goto found;
    }
}
```

后面发现，我写反了之后，导致每次请求都会遍历链表到最后一位，所以性能消耗极大。


不过很开心的是，终于解决了这个问题，还有 Nginx 模块的相关开发的一些细节没说到。以后有机会再谈谈。


