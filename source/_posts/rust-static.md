---
title: Rust 静态编译可执行文件
abbrlink: e986bf90
date: 2022-06-30 09:00:00
---

## Rust默认运行时环境

Linux 下 Rust 默认使用 gcc 作为链接器，编译后的文件在运行时需要 glibc 运行库和其他的一些库。

这就导致在某个 Linux 版本下编译的执行文件，无法在另一个 Linux 版本上顺利运行。而且，如果你的程序还使用了 OpenSSL 动态库，那这样的问题会更加突出。

我用 Cargo 创建一个可执行项目。这个项目是一个 grpc 项目，编译这个项目，用 ldd 命令查看编译出来的执行文件依赖了哪些动态链接库：

``` bash
➜  dagent git:(master) ✗ ldd target/release/dagent
        linux-vdso.so.1 (0x00007ffdbfbf6000)
        libssl.so.1.1 => /usr/lib/x86_64-linux-gnu/libssl.so.1.1 (0x00007fbac355f000)
        libcrypto.so.1.1 => /usr/lib/x86_64-linux-gnu/libcrypto.so.1.1 (0x00007fbac3094000)
        libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1 (0x00007fbac2e7c000)
        librt.so.1 => /lib/x86_64-linux-gnu/librt.so.1 (0x00007fbac2c74000)
        libpthread.so.0 => /lib/x86_64-linux-gnu/libpthread.so.0 (0x00007fbac2a55000)
        libm.so.6 => /lib/x86_64-linux-gnu/libm.so.6 (0x00007fbac26b7000)
        libdl.so.2 => /lib/x86_64-linux-gnu/libdl.so.2 (0x00007fbac24b3000)
        libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007fbac20c2000)
        /lib64/ld-linux-x86-64.so.2 (0x00007fbac3f59000)
```

可以看到这个执行文件依赖了多个 .so 的动态链接库。而且这种依赖是基于绝对路径的。一旦运行时环境下没有这些动态库文件，那程序执行的结果就只有一个：报错！

那么，Rust 能否像 Golang 那样编译成独立的静态可执行文件呢？答案是可以的，这需要使用 MUSL 静态库。

## 使用 MUSL 进行静态编译

使用 MUSL 编译，首先需要安装 musl 环境。命令如下：

``` bash
$ rustup target add x86_64-unknown-linux-musl
$ rustup target add x86_64-unknown-linux-musl --toolchain=nightly
```

然后，我们编译前面创建的 hello 工程。

``` bash
$ cd hello
$ cargo build --release --target=x86_64-unknown-linux-musl
$ ldd target/x86_64-unknown-linux-musl/release/hello
        not a dynamic executable
```

可以看到新的可执行文件已经不再依赖任何动态链接库。我们可以将这个文件放到任何一个 Linux 操作系统里运行了。

当然，这只是一个最简单的例子。实际工作中，我们可能会遇到更复杂的场景。如：依赖了 OpenSSL 库。在这样的场景里，我们还需要做很多配置才能获得我们想要的静态编译文件。这里就不再详细介绍了。

如何避免各种繁复的配置，尽可能快捷的进行MUSL编译？

## 使用预置好的 Docker 容器进行 MUSL 编译

为解决使用 MUSL 编译配置繁琐的问题，国外的开发者贡献了一个预置好的容器。用这个容器来进行 MUSL 编译会非常方便快捷。

项目地址是：https://gitlab.com/rust_musl_docker/image

我们直接用这个容器来编译我们前面创建的 grpc 工程。然后依然用 ldd 来查看编译好的可执行文件。

``` bash
$ cd dagent
$ docker run -it --rm \
-v $PWD:/workdir \
-v ~/.cargo/git:/root/.cargo/git \
-v ~/.cargo/registry:/root/.cargo/registry \
registry.gitlab.com/rust_musl_docker/image:stable-latest \
cargo build --release -vv --target=x86_64-unknown-linux-musl

$ ldd target/x86_64-unknown-linux-musl/release/dagent
        statically linked
```

这个容器镜像里已经配置了对 OpenSSL 库的静态编译。亲测可用。更详细的内容，可以去看项目里的注释，已经非常详尽了。


## PS.

https://github.com/sfackler/rust-openssl/issues/766
