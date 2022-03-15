---
title: eBPF
abbrlink: 7ba63872
date: 2022-03-15 09:00:00
---

# 简介
eBPF是一项革命性的技术，可以在Linux内核中运行沙盒程序，而无需更改内核源代码或加载内核模块。通过使Linux内核可编程，基础架构软件可以利用现有的层，从而使它们更加智能和功能丰富，而无需继续为系统增加额外的复杂性层。


# eBPF内核技术

- TCP网络数据捕获
- UDP网络数据捕获
- uprobe方式的DNS信息捕获
- 进程数据捕获
- uprobe方式实现JAVA的RASP命令执行场景事件捕获
- eBPF的go框架实现，针对kprobe\uprobe挂载方式，多类型event进行抽象实现。
- 开发者只需要实现内核态C文件，用户态go文件，用户态event消息结构体三个文件即可，框架会自动加载执行。
- 使用者可以按照logger的interface自行实现数据的上报处理，比如上报到ES\kafka等日志中心。

# 相关产品

cilium、datadog、tracee、falco、kubeArmor

[Cilium eBPF实现机制源码分析](https://www.cnxct.com/how-does-cilium-use-ebpf-with-go-and-c/?f=g_ehids)

[datadog的eBPF安全检测机制分析](https://www.cnxct.com/how-does-datadog-use-ebpf-in-runtime-security/?f=g_ehids)

[内核态eBPF程序实现容器逃逸与隐藏账号rootkit](https://mp.weixin.qq.com/s?__biz=MzUyMDM0OTY5NA==&mid=2247483773&idx=1&sn=d9a6233f2ec94b63304209246b1b6a3b&chksm=f9eaf3ecce9d7afa8c539e47ddd0250874859bc4e81e6206a0d1b3fdaffd712bf81389ced579&token=1909106120&lang=zh_CN#rd)

# 相关文章

[Linux中基于eBPF的恶意利用与检测机制](https://mp.weixin.qq.com/s/-1GiCncNTqtfO_grQT7cGw)
