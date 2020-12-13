---
title: 火焰图分析
date: 2020-12-03 19:00:00
---
关于火焰图的一些经验

## 什么是Flame Graph?

火焰图（Flame Graph）是由 Linux 性能优化大师 Brendan Gregg 发明的，和所有其他的 profiling 方法不同的是，火焰图以一个全局的视野来看待时间分布，它从底部往顶部，列出所有可能导致性能瓶颈的调用栈。

火焰图有以下特征（这里以 on-cpu 火焰图为例）：

![job-4342741811-Lua-land-CPU-Flame-Graph.png](https://i.loli.net/2020/12/13/4l6qM9JW7ecCrHv.png)

- 每一列代表一个调用栈，每一个格子代表一个函数
- 纵轴展示了栈的深度，按照调用关系从下到上排列。最顶上格子代表采样时，正在占用 cpu 的函数(或者是其他的特征)。
- 横轴的意义是指：火焰图将采集的多个调用栈信息，通过按字母横向排序的方式将众多信息聚合在一起。需要注意的是它并不代表时间。横轴格子的宽度代表其在采样中出现频率，所以一个格子的宽度越大，说明它是瓶颈原因的可能性就越大。(故这里的先后顺序与采样的顺序时间无关，只表示合起来的采样的总和)
- 火焰图格子的颜色是随机的暖色调，方便区分各个调用信息。
- on-cpu 火焰图横轴是指 cpu 占用时间，off-cpu 火焰图横轴则代表阻塞时间。(这里的时间指代为相对时间)
- 采样可以是单线程、多线程、多进程甚至是多 host，进阶用法可以参考 [systemtap-beginners-guide](https://spacewander.gitbooks.io/systemtapbeginnersguide_zh/content/index.html)和[Blazing Performance with Flame Graphs](https://www.slideshare.net/brendangregg/blazing-performance-with-flame-graphs)

## 分类

这里按照openresty的xray产品提供的分析工具进行分类。不得不说xray是一款非常好用的产品。

还可以对生成的火焰图进行代码行级别和函数级别的区分。

以及对样本进行正向和反向的生成。下图就是上面例子的反向生成

![job-4342741811-Lua-land-CPU-Flame-Graph-reversed.png](https://i.loli.net/2020/12/13/uiZ3Kwh4qFlNRWv.png)

xray里面提供了下面几种工具。不嫌麻烦的话其实 [openresty-systemtap-toolkit](https://github.com/openresty/openresty-systemtap-toolkit) 和 [stapxx](https://github.com/openresty/stapxx) 这两个工具集其实都可以实现。

- count-on-cpu-frames (count on cpu frames)
- count-off-cpu-frames (count off cpu frames)
- lj-c-on-cpu (C-land CPU Flame Graph)
- lj-lua-on-cpu (Lua-land CPU Flame Graph)
- lj-c-off-cpu (C-land off-CPU Flame Graph)
- lj-lua-off-cpu (Lua-land off-CPU Flame Graph)
- c-on-cpu (C-land CPU Flame Graph)
- c-off-cpu (C-land off-CPU Flame Graph)
- kernel-on-cpu (kernel-land on-CPU Flame Graph)
- process-memory (Virtual Memory Usage Breakdown)
- resty-memory (Application-Level Memory Usage Breakdown)
- count-lj-newgco-frames (count LuaJIT new GC objcects frames)
- lj-lua-newgco (LuaJIT GC Object Allocation Flame Graph)
- glibc-chunks (Distributions of Memory Chunk Sizes in Glibc Allocator)
- lj-gco-stat (Statistics for LuaJIT GC Objects)
- count-lj-newthread-frames (count LuaJIT new thread frames)
- lj-lua-newthread (LuaJIT Thread Objects Allocation Flame Graph)
- lj-gco-ref (GC Object Reference Flame Graph)
- count-c-memory-allocation-frames (count C Memory Allocation)
- c-memory-allocation (C Memory Allocation Flame Graph)

具体更多的信息可以参考工具集[stapxx](https://github.com/openresty/stapxx)

## 相关安装

手动安装依赖的内核调试信息包

SystemTap需要内核信息，这样才能注入指令。此外，这些信息还能帮助SystemTap生成合适的检测代码。

这些必要的内核信息分别包括在特定内核版本所对应的-devel，-debuginfo和-debuginfo-common包中。对于“标准版”内核（指按照常规配置编译的内核），所需的-devel和-debuginfo等包命名为：

```
kernel-debuginfo
kernel-debuginfo-common
kernel-devel
```
要想确定当前系统的内核版本，敲入：
```
uname -r
```

举个例子，如果你想在i686环境下的2.6.18-53.el5内核上使用SystenTap，需要下载安装如下的RPM包：
```
kernel-debuginfo-2.6.18-53.1.13.el5.i686.rpm
kernel-debuginfo-common-2.6.18-53.1.13.el5.i686.rpm
kernel-devel-2.6.18-53.1.13.el5.i686.rpm
```
一旦手动下载了所依赖的包之后，以root权限运行下面的命令来安装它们：
```
rpm --force -ivh package_names
```

### 检查安装是否成功

```
stap -v -e 'probe vfs.read {printf("read performed\n"); exit()}'
```
如果出现相关输出：
```
Pass 1: parsed user script and 45 library script(s) in 340usr/0sys/358real ms.
Pass 2: analyzed script: 1 probe(s), 1 function(s), 0 embed(s), 0 global(s) in 290usr/260sys/568real ms.
Pass 3: translated to C into "/tmp/stapiArgLX/stap_e5886fa50499994e6a87aacdc43cd392_399.c" in 490usr/430sys/938real ms.
Pass 4: compiled C into "stap_e5886fa50499994e6a87aacdc43cd392_399.ko" in 3310usr/430sys/3714real ms.
Pass 5: starting run.
read performed
Pass 5: run completed in 10usr/40sys/73real ms.
```
就表示安装完成


## 火焰图分析技巧
- 纵轴代表调用栈的深度（栈桢数），用于表示函数间调用关系：下面的函数是上面函数的父函数。
- 横轴代表调用频次，一个格子的宽度越大，越说明其可能是瓶颈原因。
- 不同类型火焰图适合优化的场景不同，比如 on-cpu 火焰图适合分析 cpu 占用高的问题函数，off-cpu 火焰图适合解决阻塞和锁抢占问题。
- 无意义的事情：横向先后顺序是为了聚合，跟函数间依赖或调用关系无关；火焰图各种颜色是为方便区分，本身不具有特殊含义
- 多练习：进行性能优化有意识的使用火焰图的方式进行性能调优（如果时间充裕）

## 使用 perf 或者 systemtap 的方式采集数据，会对后台服务有性能影响吗？

有，但是很小，可以基本忽略不计。
