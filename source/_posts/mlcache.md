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


## 缓存有两个原则

一是越靠近用户的请求越好，比如能用本地缓存的就不要发送HTTP请求，能用CDN缓存的就不要打到web服务器，能用nginx缓存的就不要用数据库的缓存。

二是尽量使用本进程和本机的缓存解决，因为跨了进程和机器甚至机房，缓存的网络开销就会非常大，在高并发的时候会非常明显。

我们直接贴一下官方的示例

``` nginx
# nginx.conf

http {
    lua_package_path "/path/to/lua-resty-mlcache/lib/?.lua;;";
    lua_shared_dict cache_dict 1m;

    init_by_lua_block {
        local mlcache = require "resty.mlcache"

        local cache, err = mlcache.new("my_cache", "cache_dict", {
            lru_size = 500,    -- size of the L1 (Lua VM) cache
            ttl      = 3600,   -- 1h ttl for hits
            neg_ttl  = 30,     -- 30s ttl for misses
        })
        if err then

        end

        _G.cache = cache
    }

    server {
        listen 8080;
        location / {
            content_by_lua_block {
                local function callback(username)
                    return db:get_user(username) -- { name = "John Doe", email = "john@example.com" }
                end

                local user, err = cache:get("my_key", nil, callback, "John Doe")
                ngx.say(user.username) -- "John Doe"
            }
        }
    }
}
```

以上的示例很好了描述了整个程序运行的逻辑，在init阶段初始化缓存，然后用_G变量赋予全局变量，在使用阶段cache:get获取指定Key的缓存，缓存未命中就会调用L3，也就是callback方法。

## 缓存风暴问题
但这个示例中缺失了lua-resty-lock这个组件的调用，为了防止在L3阶段发现缓存风暴，所以把锁非常有必要。

将局部配置修改如下：

``` c
lua_shared_dict cache_dict 1m;
lua_shared_dict cache_lock 1m;

init_by_lua_block {
    local mlcache = require "resty.mlcache"

    local cache, err = mlcache.new("my_cache", "cache_dict", {
        lru_size = 500,    -- size of the L1 (Lua VM) cache
        ttl      = 3600,   -- 1h ttl for hits
        neg_ttl  = 30,     -- 30s ttl for misses
        shm_locks = "cache_lock",
        resty_lock_opts = {
            exptime = 10,
            timeout = 5
        }
    })
    if err then

    end
}
```

## 进程之间通讯问题

这个问题我们使用lua-resty-worker-events模块解决。

此模块提供了一种向Nginx服务器中的其他工作进程发送事件的方法。通信是通过一个共享的存储区进行的，事件数据将存储在该存储区中。

结合我们之前的缓存使用场景，在一个Worker中的缓存更新之后，要通知其他Worker也同步更新，它就发挥作用了。

我们看以下官方提供的示例

``` c
lua_shared_dict process_events 1m;

init_worker_by_lua_block {
    local ev = require "resty.worker.events"

    local handler = function(data, event, source, pid)
        print("received event; source=",source,
                ", event=",event,
                ", data=", tostring(data),
                ", from process ",pid)
    end

    ev.register(handler)

    local ok, err = ev.configure {
        shm = "process_events", -- defined by "lua_shared_dict"
        timeout = 2,            -- life time of unique event data in shm
        interval = 1,           -- poll interval (seconds)

        wait_interval = 0.010,  -- wait before retry fetching event data
        wait_max = 0.5,         -- max wait time before discarding event
        shm_retries = 999,      -- retries for shm fragmentation (no memory)
    }
    if not ok then
        ngx.log(ngx.ERR, "failed to start event system: ", err)
        return
    end
}
```

在init_worker_by_lua_block阶段初始化，是因为它需要在每个Worker中都运行，便于同步到其他进程，其他的就是一些配置参数问题。

下面我们把它结合上面的缓存模块一起使用。

lua-resty-mlcache提供了ipc接口来支持lua-resty-worker-events模块，我们直接配置参数即可。


``` c
lua_shared_dict cache_dict    1m;
lua_shared_dict cache_lock    1m;
lua_shared_dict worker_events 1m;

init_worker_by_lua_block {
    local mlcache = require "resty.mlcache"
    local worker_events = require "resty.worker.events"

    local ok, err = worker_events.configure {
        shm = "worker_events",
        timeout = 2,
        interval = 1,

        wait_interval = 0.010,
        wait_max = 0.5,
        shm_retries = 999,
    }

    local cache, err = mlcache.new("my_cache", "cache_dict", {
        lru_size = 500,    -- size of the L1 (Lua VM) cache
        ttl      = 3600,   -- 1h ttl for hits
        neg_ttl  = 30,     -- 30s ttl for misses
        shm_locks = "cache_lock",
        resty_lock_opts = {
            exptime = 10,
            timeout = 5
        },
        ipc = {
            register_listeners = function(events)
                for _, event_t in pairs(events) do
                    worker_events.register(
                        function(data)
                            event_t.handler(data)
                        end,
                        channel_name,
                        event_t.channel
                    )
                end
            end,
            broadcast = function(channel, data)
                worker_events.post(channel_name, channel, data)
            end
        }
    })
    if err then

    end
}
```
