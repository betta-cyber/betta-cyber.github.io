---
title: Rust 交叉编译及 CI/CD 相关
abbrlink: d2381dbd
date: 2020-02-04 01:13:14
---

## 问题由来

由于涉及到 Rust 交叉编译的问题，本人不太想折腾。主要是在不想在 Windows 上安装 Rust 相关。手头现在有一台 MacOS 设备和一台 Linux 服务器，想交叉编译到 Win 上，于是做了一点尝试。MinGW 应该是比较靠谱的方案。本人在 Mac 上通过
```
brew install FiloSottile/musl-cross/musl-cross
brew install mingw-w64
```
进行安装，但是网速慢，编译慢，等待时间较长，于是在安装的过程中开始探寻其他途径。CI/CD 是一个方案。Docker 看起来也不错。在一番尝试之后，看起来 GitHub action 对于 Rust Windows 的文档不是很全。于是选择 Travis 进行测试。Docker 鉴于国内网络环境相关除非万不得已不然不太想尝试。

## travis 相关

```
os: windows
language: rust
rust:
- stable
script:
- cargo build --release --verbose
- ls target/release
deploy:
  api_key:
    secure: xxxx
  file_glob: true
  file:
    - "target/release/gs.exe"
    - "README.md"
  skip_cleanup: true
  provider: releases
  on:
    tags: true
```

上面是一份配置，简单说明一下，os 表示操作系统，然后是 rust 和版本。再就是一些脚本，deploy 会进行部署，这里 provider 选择 releases，然后这里需要一个 token。

首先通过 Ruby 的 gem 安装 travis

`gem install travis`

然后需要登录

`travis login --pro`

然后需要加密

`travis encrypt SOMEVAR="secretvalue"`生成出来的加密串放到`.travis.yml` 中，就可以完成认证。

## GitHub actions

```
name: Continuous Integration
on:
  push:
    branches:
      - master
  pull_request:
    paths:
      - '**.rs'
      - 'Cargo.toml'
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v1
    - name: Build
      run: |
        sudo apt-get update
        sudo apt-get -y install libasound2-dev libdbus-1-dev dbus
        cargo test --verbose
        cargo build --verbose
```

这个例子就是一个 action 的 ci 例子。

## Rust Windows-GNU 与 MinGW

2 月 5 号更新，鉴于 travis 生成的 exe 文件在 win10 上面不能成功运行，且 trace 看不出什么究竟，故而选择在 macOS 上面交叉编译，macOS 上面由于 rust 官方 toolchains 的 lib 中 crt2.o 较老，会出现一些 `undefined reference to '__imp___acrt_iob_func'` 类似的报错，需要手动进行替换。
在 `~/.rustup/toolchains/stable-x86_64-apple-darwin/lib/rustlib/x86_64-pc-windows-gnu/lib` 目录下，也就是你的 rust 版本的 lib 下，进行备份，然后将通过 brew 安装的 mingw-w64 下的 lib crt2 进行替换。

```
mv crt2.o crt2.o.bak
cp /System/Volumes/Data/usr/local/Cellar/mingw-w64/7.0.0/toolchain-x86_64/x86_64-w64-mingw32/lib/crt2.o ./
```

替换之后可以进行正常编译，通过指定 target`cargo build --release --target x86_64-pc-windows-gnu` 将编译出相应的 exe，但是对于 webview 的 example 例子，编译之后的文件还是无法在 win10 上面运行。最基本的 rust helloworld 程序却是可以的，这说明其实我 mac 上面交叉编译其实这条路是走通了，但是对于 rust 的 webview 这个库其实还是有点问题。但是查了相关 issuse，并没有发现这样的问题，于是决定在 win10 上面进行问题排查。

于是只好选择在 windows 上面安装 rust，通过 rustup 安装，设置相关中国源，安装成功后，由于 Rust 使用的链接器是系统提供的，而且 Rust 的标准库也对 libc 有依赖。在 Linux/Mac 等环境下，Rust 会使用 gcc 执行链接。在 Windows 下，系统没有原生自带链接器。主流的免费 C/C++ 工具链主要有五套，分别是 Visual C++、Clang、Mingw-w64 GCC、MSYS2 GCC、CYGWIN GCC。

通过 `rustup target list` 可以看到 Rust 在 Windows 下的编译支持主要是两种，分别对应列表里的 Visual C++ 和 Mingw-w64 GCC 两种，分别称作 msvc 和 gnu 的 target，同时有对应的 toolchain。msvc 的情况在这里不细说了。下面主要说下 gnu。

本人在安装默认选择 `x86_64-pc-windows-gnu`，rustup 会自行安装 rust-mingw，但是 rust 捆绑的 mingw 运行时版本要旧于现在很多 MinGW 运行时版本。导致我的项目 clone 下来，无法直接编译成功。一直提示不支持 64 位操作系统。现在想来应该是 Babun 自带的 gcc 是 32 位的导致我一直编译不通过。一开始我还没发现这个问题，索性一不做二不休，直接上 MinGW-w64，MinGW-w64 也有很多种。在 windows 上运行的版本，提供的有几家，比较官方的是 mingw-w64-builds (MinGW-builds)，另外还有 msys2 (MSYS2 homepage)、win-builds (http://win-builds.org) 等等。我这里选择的是官方的途径安装。不建议在线安装，通过下载离线版方式，并写入环境变量。

然后就是进行替换。
- 移除 rust-mingw
- 告诉 rust 使用自己安装的 mingw。
- 替换 crt2.o 和 dllcrt2.o。


1. 执行 `rustup component remove rust-mingw`
2. 添加配置到 cargo 的 config 当中
```
[target.x86_64-pc-windows-gnu]
linker = "C:\\mingw64\\bin\\gcc.exe"
ar = "C:\\mingw64\\bin\\ar.exe"
```
3. 和 macOS 上面替换是一样的原理。

现在就可以编译成功了。

ps. Windows 上还是自己安装一个 MinGW，配置好然后应用到 cargo 上，省事。
