---
title: Rust 和 OpenResty 的性能问题
abbrlink: 5ab43bf2
date: 2020-03-27 15:34:00
---
关于 rust 和 openresty 的一些问题

## 问题引入

最近要入职 DY 了，然后这边准备入职之后负责一个安全网关和WAF相关的一些事情。考虑到斗鱼这边使用openresty来实现WAF功能。我这边之前又在学习RUST，所以我来做一个测试，想测试一下用RUST来做一个安全网关的工作。看看能不能又一点的效果。

之前都是在用apache自带的ab做压力测试。后面考虑到性能测试的准确性，我于是用了wrk这个工具。

```
git clone https://github.com/wg/wrk
cd wrk
make
```
编译之后在下面生成wrk的文件，可以通过一些参数测试。

```
./wrk -t 4 -c 100 -d 10s --latency http://10.1.78.178:3001
```

我这边用rust的hyper做了一个gateway的一个功能，主要是吧流量代理到0.0.0.0:8080上面，代码如下：

```
#![deny(warnings)]

use hyper::service::{make_service_fn, service_fn};
use hyper::{Client, Error, Server};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let in_addr = ([0, 0, 0, 0], 3001).into();
    let out_addr: SocketAddr = ([0, 0, 0, 0], 8080).into();

    let client_main = Client::new();

    let out_addr_clone = out_addr.clone();

    // The closure inside `make_service_fn` is run for each connection,
    // creating a 'service' to handle requests for that specific connection.
    let make_service = make_service_fn(move |_| {
        let client = client_main.clone();

        async move {
            // This is the `Service` that will handle the connection.
            // `service_fn` is a helper to convert a function that
            // returns a Response into a `Service`.
            Ok::<_, Error>(service_fn(move |mut req| {
                let uri_string = format!(
                    "http://{}{}",
                    out_addr_clone,
                    req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("")
                );
                let uri = uri_string.parse().unwrap();
                *req.uri_mut() = uri;
                client.request(req)
            }))
        }
    });

    let server = Server::bind(&in_addr).serve(make_service);

    println!("Listening on http://{}", in_addr);
    println!("Proxying on http://{}", out_addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
```

然后我用openresty的官方最新下载包，[安装编译](https://openresty.org/cn/getting-started.html)。然后编写相关的openresty的配置
```
worker_processes  1;
error_log logs/error.log;
events {
    worker_connections 1024;
}
http {
    server {
        listen 8081;
        location /hello {
            default_type text/html;
            content_by_lua_block {
                ngx.say("<p>hello, world</p>")
            }
        }

        location / {
            proxy_pass http://127.0.0.1:8080;
        }
    }
}
```
这里对流量进行了proxy_pass的代理到8080端口，然后我在8080后面用tornado启动了一个http服务。

## 测试结果

rust-hyper写的测试结果
```
./wrk -t 4 -c 100 -d 10s --latency http://10.1.78.178:3001
Running 10s test @ http://10.1.78.178:3001
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    48.49ms    3.14ms  82.11ms   87.34%
    Req/Sec   517.28     53.08   696.00     82.50%
  Latency Distribution
     50%   47.98ms
     75%   49.70ms
     90%   51.35ms
     99%   62.28ms
  20600 requests in 10.03s, 5.01MB read
Requests/sec:   2053.78
Transfer/sec:    511.44KB
```

openresty的测试结果
```
./wrk -t 4 -c 100 -d 10s --latency http://10.1.78.178:8081
Running 10s test @ http://10.1.78.178:8081
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    60.43ms    3.96ms  74.39ms   85.46%
    Req/Sec   414.72    103.06   505.00     75.00%
  Latency Distribution
     50%   59.69ms
     75%   61.90ms
     90%   65.07ms
     99%   71.96ms
  16525 requests in 10.03s, 4.38MB read
Requests/sec:   1648.06
Transfer/sec:    447.37KB
```

可以看出rust的性能还是比openresty要强的，这里还用的不是actix-web这个性能怪兽。然后在这里主要还是lua的拓展性比较强，但是我个人觉得其实rust在这块其实做的也不错。我会尝试用rust取构建一个安全网关。
