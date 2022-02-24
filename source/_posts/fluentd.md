---
title: Fluentd 基础使用方式
abbrlink: 916b8ba4
date: 2022-02-24 18:13:14
---

Fluentd 是一套开源数据搜集软体软件 (Data Collection Software)。通常在专案中我们会需要将各种资料传递到不同服务，如 Apache, MySQL, elasticsearch 等服务，但不同服务间的资料传递方式却各自不同，常会造成混乱。

![fluentd-before](/article_photo/fluentd-before.png)

Fluentd 提供了统一的资料中介层 (Unified Logging Layer)，可将资料由不同来源汇入后，经过 Buffer 与资料处理后再将转抛到所设定的目的地，可大幅度降低系统间资料传递的复杂度。

![fluentd-architecture](/article_photo/fluentd-architecture.png)

Fluentd 还包含以下特色

- 由 C 与 Ruby 写成。
- 资料以 Json 格式搜集与转抛。
- 支援多重 Input/Output 格式。
- 由多重 Plugin 组成，可自行加入非预设的功能。
- 透过设定档设定资料处理流程。

## td-agent

要使用 Fluentd 除了直接透过 Ruby Gem 安装外，也可安装 td-agent，由 Treasure Data 所维护的的发行版(The stable distribution of Fluentd)，因此之后的使用范例均用 td-agent。

### 安装 td-agent (Ubuntu)

```
## Ubuntu 16.04 (Xenial)
curl -L https://toolbelt.treasuredata.com/sh/install-ubuntu-xenial-td-agent3.sh | sh

## Ubuntu 18.04 (Bionic)
curl -L https://toolbelt.treasuredata.com/sh/install-ubuntu-bionic-td-agent3.sh | sh
```

### 安裝 plugin

Fluentd 可透过安装外部 plugin 来扩充功能，可到 Fluentd 官网查询可安装列表，之后使用 td-agent-gem 指令来安装支援该服务的 plugin 到 td-agent 中。

如安装 elasticsearch 的 Fluentd plugin，可执行下列指令

```
td-agent-gem install fluent-plugin-elasticsearch
```

### Fluentd Config

Fluentd 的资料接收，资料处理与资料导出的资料流处理流程都透过设定档来进行设定。而 td-agent 的设定档位于/etc/td-agent/td-agent.conf。其中包含许多资料处理区间如

```
<source>
...
</source>

<match pattern>
  <filter>
  ...
  </filter>
...
</match>
```

该区间就是在定义 Fluentd 的资料来源与处理方式。不同区间代表不同的处理类型，如

- \<source\> - 资料输入(Input)来源设定
- \<match pattern\> - 将 tag 符合 pattern 的资料输出(Output)到设定的目的地。
- \<filter\>: 资料处理与过滤方式。

还有其他如 `<parse>`，`<format>`，`<buffer>` 等处理区间。

### Example

```
<source>
  @type http
  port 9880
</source>

<match debug.**>
  @type stdout
</match>
```

上面的设定 Fluentd 会接收来自 port 9880 的输入，并将 Tag 为 debug.\* 的内容输出到标准输出。当透过指令输入


### Routing

由于 Fluentd 的资料流为 Top-down 的方式处理，也就是若之前已经使用 \<match pattern\> 撷取资料，在之后的段落是无法取得已经被撷取的资料，因此可以透过相关 plugin 对资料做 Routing 以分流处理。

### copy

`out_copy` plugin 可以复制资料流到不同的 \<match\> 区间中

```
<match park.log>
  @type copy
  <store>
    @type file
    ...
  </store>
  <store>
    @type forward
    ...
  </store>
</match>
```

### relabel

通过 `out_relable` plugin，将资料标注新 label 并在外部处理

```
<match park.log>
  @type copy
  <store>
    @type relabel
    @label OUTPUT_FILE
  </store>
  <store>
    @type relabe
    @label OUTPUT_FORWARD
  </store>
</match>

<label @OUTPUT_FILE>
  <match park.log>
    @type file
    ...
  </match>
</label>

<label @OUTPUT_FORWARD>
  <match park.log>
    @type forward
    ...
  </match>
</label>
```


### 测试 config 并重新启动服务
当修改过 td-agent.conf 后可先测试该 config 设定是否可执行，只要在资料中

```
td-agent --dry-run -c [config-file]
```


### Reference

- [Fluentd](https://www.fluentd.org/)
- [Fluentd - Doc](https://www.fluentd.org/)
- [Fluentd Bit](https://www.fluentd.org/)
