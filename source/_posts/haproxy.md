---
title: HAProxy 问题解决
abbrlink: 5a1d2c0e
date: 2022-03-22 09:00:00
---

HAProxy 网络错误：无法绑定套接字

执行日志如下

```
[root@hk-mailng-8-162 ~]# systemctl status haproxy -l
● haproxy.service - HAProxy Load Balancer
   Loaded: loaded (/usr/lib/systemd/system/haproxy.service; disabled; vendor preset: disabled)
   Active: failed (Result: exit-code) since Tue 2022-03-22 10:38:05 CST; 11s ago
  Process: 14335 ExecStart=/usr/sbin/haproxy-systemd-wrapper -f /etc/haproxy/haproxy.cfg -p /run/haproxy.pid $OPTIONS (code=exited, status=1/FAILURE)
 Main PID: 14335 (code=exited, status=1/FAILURE)

Mar 22 10:38:05 hk-mailng-8-162 systemd[1]: Started HAProxy Load Balancer.
Mar 22 10:38:05 hk-mailng-8-162 systemd[1]: Starting HAProxy Load Balancer...
Mar 22 10:38:05 hk-mailng-8-162 haproxy-systemd-wrapper[14335]: haproxy-systemd-wrapper: executing /usr/sbin/haproxy -f /etc/haproxy/haproxy.cfg -p /run/haproxy.pid -Ds
Mar 22 10:38:05 hk-mailng-8-162 haproxy-systemd-wrapper[14335]: [ALERT] 080/103805 (14336) : Starting frontend exchange_smtp: cannot bind socket [0.0.0.0:25]
Mar 22 10:38:05 hk-mailng-8-162 haproxy-systemd-wrapper[14335]: haproxy-systemd-wrapper: exit, haproxy RC=1
Mar 22 10:38:05 hk-mailng-8-162 systemd[1]: haproxy.service: main process exited, code=exited, status=1/FAILURE
Mar 22 10:38:05 hk-mailng-8-162 systemd[1]: Unit haproxy.service entered failed state.
Mar 22 10:38:05 hk-mailng-8-162 systemd[1]: haproxy.service failed.

```

发现Starting frontend exchange_smtp: cannot bind socket [0.0.0.0:25]

要解决cannot bind socket错误，您需要确定哪些其他进程正在侦听 HAProxy 尝试使用的 IP 地址和端口，或者该 IP 地址是否可用于 HAProxy。

以下命令将确定已绑定到 port 上 IPv4 接口的进程的名称80。80如果错误消息中的端口与以下命令中的不同，请确保将其替换为端口：

```
sudo ss -4 -tlnp | grep 80
```

该ss命令的标志以下列方式更改其默认输出：

- -4 限制ss为仅显示与 IPv4 相关的套接字信息。
- -t 仅将输出限制为tcp套接字。
- -l 显示考虑了-4和限制的所有侦听套接字。-t
- -n 确保显示端口号，而不是像“http orhttps”这样的协议名称。这很重要，因为 HAProxy 可能会尝试绑定到非标准端口，并且与实际端口号相反，服务名称可能会令人困惑。
- -p 输出有关绑定到端口的进程的信息。
- | grep 80将输出限制为包含字符80的行，因此您必须检查的行更少


```
LISTEN   0         511                 0.0.0.0:80               0.0.0.0:*        users:(("nginx",pid=40,fd=6))
```

也可以使用`netstat`来查找

```
netstat -ntlp | grep 25

tcp        0      0 127.0.0.1:25                0.0.0.0:*                   LISTEN      15903/master
```

#### 查找进程对应的服务

可以用`locate`或者`ll /etc/init.d/postfix`

```
/usr/libexec/postfix/master
```

#### 停用服务

停用postfix服务

```
[root@usm ~]# /etc/init.d/postfix stop
或者
[root@usm ~]# service postfix stop
```

最后写的 HAProxy 配置

```
frontend  exchange_smtp
    mode tcp
    bind :25
    default_backend            exchange_25

backend exchange_25
    mode tcp
    balance     source
    server  mail1 10.56.8.121:25
    server  mail2 10.56.8.122:25
    server  mail3 10.56.8.123:25
```

重启之后，25端口不被占据，进行转发。

最后用 telnet 进行端口测试。
