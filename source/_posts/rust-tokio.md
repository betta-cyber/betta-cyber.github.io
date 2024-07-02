---
title: Rust Tokio
abbrlink: 5a6dd95c
date: 2023-09-14 01:13:14
---


## tokio::select!

tokio::select! 宏允许在多个异步计算中等待，并在单个计算完成后返回。

注意，select 是在单线程中执行

目前来说，select! 最多可以支持 64 个分支，每个分支形式如下：

<模式> = <async 表达式> => <结果处理>,


``` rust
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();

    tokio::spawn(async {
        let _ = tx1.send("one");
    });

    tokio::spawn(async {
        let _ = tx2.send("two");
    });

    tokio::select! {
        val = rx1 => {
            println!("rx1 completed first with {:?}", val);
        }
        val = rx2 => {
            println!("rx2 completed first with {:?}", val);
        }
    }
}
```



## tokio::spawn

创建一个异步任务,不仅可以单独的走异步任务，异步任务还可以返回结果，使用 await 接收


``` rust
use tokio;

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        println("OX")
    });

    let handle = tokio::spawn(async {
        10086
    });
    let out = handle.await.unwrap();
    println!("GOT {}", out);

    async fn more_async_work() -> &'static str {
        tokio::spawn(async {
            "two"
        }).await.unwrap_or("error")
    }

    async fn do_stuff_async() -> &'static str {
        tokio::spawn(async {
            "two"
        }).await.unwrap_or("error")
    }

    println("{}", do_stuff_async().await)
}
```

## tokio::join!

使用 tokio::join! 宏同时执行多个异步任务

``` rust
async fn do_stuff_async() -> &'static str {
    let s = tokio::spawn(async {
        "one"
    }).await.unwrap();
    s
}

async fn more_async_work() -> &'static str {
    tokio::spawn(async {
        "two"
    }).await.unwrap_or("error")
}

#[tokio::main]
async fn main() {
    let (first, second) = tokio::join!(
        do_stuff_async(),
        more_async_work()
    );

    println!("{}, {}, {}", first, second, more_async_work().await)
    // one, two, two

}
```


tokio提供了两种工作模式的runtime：

1.单一线程的runtime(single thread runtime，也称为current thread runtime)
2.多线程(线程池)的runtime(multi thread runtime)

创建单一线程的runtime

``` rust
#![allow(unused)]
fn main() {
    // 创建单一线程的runtime
    let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
}
```

创建多线程的runtime，查看其线程数:
``` rust
use tokio;

fn main(){
    let rt = tokio::runtime::Runtime::new().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(10));
}
```

[tokio::main]创建的是多线程runtime，还有以下几种方式创建多线程runtime：
[tokio::main(flavor = “multi_thread”] // 等价于#[tokio::main]
[tokio::main(flavor = “multi_thread”, worker_threads = 10))]
[tokio::main(worker_threads = 10))]


https://liangyuetian.cn/posts/61015457/
https://rust-book.junmajinlong.com/ch100/01_understand_tokio_runtime.html
https://skyao.io/learning-tokio/docs/tutorial/select.html#tokioselect
