---
title: Frida hook 应用商店 Sign
abbrlink: eca7d6b2
date: 2023-10-14 01:13:14
---



直接用 frida 进行 hook 抓包，bp 中发现原始包中包含 sign。对 sign 生成流程进行分析。
![](/article_photo/oppo1.png)

![](/article_photo/oppo2.png)

利用 frida 遍历类和方法找到想要 hook 的函数：
```python
jscode = """
Java.perform(function(){
    var classz = Java.enumerateLoadedClassesSync();
    for(var i=0;i<classz.length;i++){
        if (classz[i].indexOf("com.heytap.cdo.client.OcsTool") != -1){
            console.log(classz[i])

            var MainActivity = Java.use(classz[i]);
            var methods = MainActivity.class.getDeclaredMethods();

            for(var j=0;j<methods.length;j++){
                console.log(methods[j].getName())
            }
        }
    }
})
"""
```

对 addSign 方法进行 hook：

```python
jscode = """
Java.perform(function(){

    var temp = Java.use('com.heytap.cdo.client.domain.network.interceptor.HeaderInitInterceptor');

    temp.addSign.implementation = function(arg1, arg2, arg3, arg4) {
        send("Hook start....");
        var result = this.addSign(arg1, arg2, arg3, arg4);
        send("***arg1***:" + arg1);
        send("***arg2***:" + arg2);
        send("***arg3***:" + arg3);
        send("***arg4***:" + arg4);
        send("***result***" + result);
        return result;
    };
});
"""
```

hook到了参数和结果：
```bash
[*] Hook start....
[*] ***arg1***:Redmi%2FM2007J3SC%2F30%2F11%2FUNKNOWN%2F2%2FRKQ1.200826.002+test-keys%2F830021696941420183////tracker/v1/infopkg=&start=0&type=0
[*] ***arg2***:128
[*] ***result***7f0ad3db84508c082a9e732a4d1ee520
```
发现 addSign 中内部调用了一个 so 的 native 方法：

![](/article_photo/oppo3.png)

public static native String c(String p0,int p1);

于是想着先对 so 进行 hook：

```python
# jscode = """

# function hook_native() {
    # var addr = Module.getExportByName("libocstool.so", "Java_com_heytap_cdo_client_OcsTool_c")
    # Interceptor.attach(addr, {
        # onEnter: function (args) {
            # console.log("args 1 ", args[0])
            # console.log("str 1 ", Memory.readCString(args[0]))

            # console.log("args 2 ", args[1])
            # console.log("str 2 ", Java.vm.getEnv().getStringUtfChars(args[1], null).readCString())
            # console.log("str 3 ", Number(args[2]))
        # }, onLeave: function (retval) {
            # console.log("retval is ",  retval)
        # }
    # })
# }

# function main() {
    # hook_native()
# }

# setImmediate(main)
# """
```

对 native 的 hook 不是很顺利。入参字符串读不出来。所以直接转到 IDA 里面对 so 进行分析：
在 IDA 里面找到 Java_com_heytap_cdo_client_OcsTool_c ，直接 F5 。

![](/article_photo/oppo4.png)
![](/article_photo/oppo5.png)

看到是最后对字符串进行了一个 md5 操作。而且这个 md 还是用 java 调的。


```javascript
    var MD5Encrypt = Java.use("java.security.MessageDigest");

    MD5Encrypt.update.overload('[B').implementation = function (args1) {
        //console.log("MD5Encrypt args1:",args1);
        send(Uint8ArrayToString(args1));
        var result = this.update(args1);
        send("update", result);
        send("MD5Encrypt.encode result==:",result);
        return result;
    };

     MD5Encrypt.digest.overload().implementation = function (args1, args2, args3, args4, args5, args6) {
        var args = this.digest();
        console.log("fan_hui",byteToHexString(args));
        return args
    }
```


Hook 一波。发现了 加密方法的 入参。由三部分组成：

第一部分 `cdb09c43063ea6bb08f4fe8a43775179bdc58acb383220be`

第二部分

```bash
Redmi%2FM2007J3SC%2F30%2F11%2FUNKNOWN%2F2%2FRKQ1.200826.002+test-keys%2F830021696941454952////search/v1/searchtabId=&size=10&searchType=3&start=0&keyword=健康156
```

第三部分字符串：

```bash
STORENEWMIICeAIBADANBgkqhkiG9w0BAQEFAASCAmIwggJeAgEAAoGBANYFY/UJGSzhIhpx6YM5KJ9yRHc7YeURxzb9tDvJvMfENHlnP3DtVkOIjERbpsSd76fjtZnMWY60TpGLGyrNkvuV40L15JQhHAo9yURpPQoI0eg3SLFmTEI/MUiPRCwfwYf2deqKKlsmMSysYYHX9JiGzQuWiYZaawxprSuiqDGvAgMBAAECgYEAtQ0QV00gGABISljNMy5aeDBBTSBWG2OjxJhxLRbndZM81OsMFysgC7dq+bUS6ke1YrDWgsoFhRxxTtx/2gDYciGp/c/h0Td5pGw7T9W6zo2xWI5oh1WyTnn0Xj17O9CmOk4fFDpJ6bapL+fyDy7gkEUChJ9+p66WSAlsfUhJ2TECQQD5sFWMGE2IiEuz4fIPaDrNSTHeFQQr/ZpZ7VzB2tcG7GyZRx5YORbZmX1jR7l3H4F98MgqCGs88w6FKnCpxDK3AkEA225CphAcfyiH0ShlZxEXBgIYt3V8nQuc/g2KJtiV6eeFkxmOMHbVTPGkARvt5VoPYEjwPTg43oqTDJVtlWagyQJBAOvEeJLno9aHNExvznyD4/pR4hec6qqLNgMyIYMfHCl6d3UodVvC1HO1/nMPl+4GvuRnxuoBtxj/PTe7AlUbYPMCQQDOkf4sVv58tqslO+I6JNyHy3F5RCELtuMUR6rG5x46FLqqwGQbO8ORq+m5IZHTV/Uhr4h6GXNwDQRh1EpVW0gBAkAp/v3tPI1riz6UuG0I6uf5er26yl5evPyPrjrD299L4Qy/1EIunayC7JYcSGlR01+EDYYgwUkec+QgrRC/NstV
```

标红部分为固定值，黄色为请求 Header 中 ocs 字段，为设备信息不同设备不一样，蓝色部分为 Header 中 t 字段，固定部分。绿色部分为Header中id字段，设备IMEI号码。紫色部分为查询的 url, keyword为搜索字。之后棕色部分为前面字段的length()值。最后的红色部分也为固定字段，可以在 so 里面找到。

![](/article_photo/oppo6.png)

md5 之后 转成 16 进制，tohex

Header 中的 oak 字段是 so 生成的。
Java_com_heytap_cdo_client_OcsTool_b 方法。
![](/article_photo/oppo7.png)
固定入参 103。.

![](/article_photo/oppo8.png)
每次返回固定值。获取到即可。

![](/article_photo/oppo9.png)
init_keys 方法。

![](/article_photo/oppo10.png)
固定字段，加密前最后一部分。

![](/article_photo/oppo11.png)

编写代码：

搜索接口：

```bash
GET /search/v1/search?tabId=&size=10&searchType=10&start=0&inputWord=%E5%81%A5%E5%BA%B7&keyword=%E5%81%A5%E5%BA%B7 HTTP/1.1
rt: 0
pr: 1
romver: -1
cpu-arch: arm64-v8a
User-Agent: Redmi%2FM2007J3SC%2F30%2F11%2FUNKNOWN%2F2%2F2101%2F83002%2F8.3.2
sign: 722c57523c27be3124baa0859e73f10d
nw: 1
pid: 001
enter-id: 1
locale: zh-CN;CN
iad: 3557812%2C30813121%2C39680%2C3417593%2C3368287%2C3056825
ouid-limit-status: 1
sg: 5582a7e2abcf5689c2ca78a083528651
oak: cdb09c43063ea6bb
id: ///
ocp: 8767
ocs: Redmi%2FM2007J3SC%2F30%2F11%2FUNKNOWN%2F2%2FRKQ1.200826.002+test-keys%2F83002
traceId: 32QpNqnL-1696933128182
Accept: application/x2-protostuff; charset=UTF-8
ch: 2101
appversion: 8.3.2
token: -1
pkg-ver: 0
component: 83002/2
t: 1696933128180
appid: Redmi#001#CN
ext-info: normal
Host: api-cn.store.heytapmobi.com
Connection: close
```
其中application/x2-protostuff 协议返回一种二进制文件。直接把 Accept 改成 json 或者直接去掉就行。免得还要分析新的 protostuff 文件。

下载接口：
```bash
GET /download/v1/23646102?type=1&ref=200&mtag=e5f378cc3e146776b84f96d439a2f027 HTTP/1.1
pr: 1
romver: -1
cpu-arch: arm64-v8a
User-Agent: Redmi%2FM2007J3SC%2F30%2F11%2FUNKNOWN%2F2%2F2101%2F83002%2F8.3.2
sign: fe7baf48faf0ea9247f4de0aaff2e566
nw: 1
pid: 001
enter-id: 1
locale: zh-CN;CN
iad: 3557812%2C30813121%2C39680%2C3368287%2C3056825
sid: 7d4a99534f15cee1
ouid-limit-status: 1
sg: 8daa735e2420acb9c0c629878a89e8f1
oak: cdb09c43063ea6bb
id: ///
ocp: 8767
ocs: Redmi%2FM2007J3SC%2F30%2F11%2FUNKNOWN%2F2%2FRKQ1.200826.002+test-keys%2F83002
traceId: MdBz6w6Y-1696930967290
ch: 2101
appversion: 8.3.2
token: -1
pkg-ver: 0
component: 83002/2
t: 1696930967288
appid: Redmi#001#CN
ext-info: normal
RANGE: bytes=14208667-28417333
Host: api-cn.store.heytapmobi.com
Connection: close
```

返回结果：
```bash
HTTP/1.1 302 Found
Server: nginx
Date: Tue, 10 Oct 2023 09:42:47 GMT
Content-Length: 0
Connection: close
location: https://storedl3.heytapdownload.com/apk/202307/26/7bce7d789a29ed1dc72dcbc558fda3c3.apk?sign=52120d03935223f3b3c3b17b66b221c8&t=65251C97&m=e5f378cc3e146776b84f96d439a2f027&hm=0e16ce45b14b6f6e821b71e38039f776
x-ocip: 220.249.100.111
ocd: 1
ogv: 20230908105131
X-Backend-Host: 0409:18817
X-Gateway-Host: 23c5a88d9fa46ae19f3c7bd4cb4a8a9e3f49ef0cb03627023ff6a5beac0b0590d043eff25553b3efe9af951213bdd3d5
```
可以看到 nginx 返回 302，并进行了一个跳转，跳转的就是真实可用的下载链接。
拿到这个 url 就可以直接下载了。



参考文档：
https://blog.csdn.net/weixin_42011443/article/details/106814333
https://curlconverter.com/
https://blog.palug.cn/3026.html


代码：
```python
# -*- coding: utf-8 -*-

import hashlib
import requests
import urllib.parse


def sign(ocs, t, id, uri):
    # start_token = "23a8ba872e43065370f68c62df3ba8a45f1c1a57c91df63e"
    start_token = "cdb09c43063ea6bb08f4fe8a43775179bdc58acb383220be"
    end_token = "STORENEWMIICeAIBADANBgkqhkiG9w0BAQEFAASCAmIwggJeAgEAAoGBANYFY/UJGSzhIhpx6YM5KJ9yRHc7YeURxzb9tDvJvMfENHlnP3DtVkOIjERbpsSd76fjtZnMWY60TpGLGyrNkvuV40L15JQhHAo9yURpPQoI0eg3SLFmTEI/MUiPRCwfwYf2deqKKlsmMSysYYHX9JiGzQuWiYZaawxprSuiqDGvAgMBAAECgYEAtQ0QV00gGABISljNMy5aeDBBTSBWG2OjxJhxLRbndZM81OsMFysgC7dq+bUS6ke1YrDWgsoFhRxxTtx/2gDYciGp/c/h0Td5pGw7T9W6zo2xWI5oh1WyTnn0Xj17O9CmOk4fFDpJ6bapL+fyDy7gkEUChJ9+p66WSAlsfUhJ2TECQQD5sFWMGE2IiEuz4fIPaDrNSTHeFQQr/ZpZ7VzB2tcG7GyZRx5YORbZmX1jR7l3H4F98MgqCGs88w6FKnCpxDK3AkEA225CphAcfyiH0ShlZxEXBgIYt3V8nQuc/g2KJtiV6eeFkxmOMHbVTPGkARvt5VoPYEjwPTg43oqTDJVtlWagyQJBAOvEeJLno9aHNExvznyD4/pR4hec6qqLNgMyIYMfHCl6d3UodVvC1HO1/nMPl+4GvuRnxuoBtxj/PTe7AlUbYPMCQQDOkf4sVv58tqslO+I6JNyHy3F5RCELtuMUR6rG5x46FLqqwGQbO8ORq+m5IZHTV/Uhr4h6GXNwDQRh1EpVW0gBAkAp/v3tPI1riz6UuG0I6uf5er26yl5evPyPrjrD299L4Qy/1EIunayC7JYcSGlR01+EDYYgwUkec+QgrRC/NstV"

    data = ocs + t + id + uri
    print(data)
    print(len(data))

    # data = start_token + ocs + t + id + uri
    data = start_token + data
    print(data)

    length = len(data)
    data = data + str(length) + end_token

    print("----before-----")
    print(data)

    hl = hashlib.md5()
    hl.update(data.encode(encoding='utf-8'))
    sign = hl.hexdigest()

    print("----after-----")
    print(sign)
    return sign


headers = {
    'id': '///',
    'oak': 'cdb09c43063ea6bb',
    'ocs': 'Redmi%2FM2007J3SC%2F30%2F11%2FUNKNOWN%2F2%2FRKQ1.200826.002+test-keys%2F83002',
    't': '1697161301085',
}

url = 'https://api-cn.store.heytapmobi.com'
uri = '/search/v1/search'

params = {
    'tabId': '',
    'size': '10',
    'searchType': '3',
    'start': '0',
    'keyword': '酷狗',
}

url_str = uri
for k, v in params.items():
    url_str += k + '=' + v + '&'

url_str = url_str[:-1]
print(url_str)

# uri = urllib.parse.quote(uri)
url = url + uri
print(url)
# uri = '/download/v1/23646102?type=1&ref=200&mtag=e5f378cc3e146776b84f96d439a2f027'


# 1697102213193////card/store/v4/search/homesize=10&start=0

headers['sign'] = sign(headers['ocs'], headers['t'], headers['id'], url_str)

# response = requests.get(url, params=params, headers=headers)
# # response = requests.get(url, headers=headers)

# print(response.status_code)
# print(response.text)


url = "https://api-cn.store.heytapmobi.com"
uri = "/download/v1/23644742"

params = {
    'type': '1',
    'ref': '200',
}
url_str = uri
for k, v in params.items():
    url_str += k + '=' + v + '&'

url_str = url_str[:-1]
print(url_str)

# uri = urllib.parse.quote(uri)
url = url + uri
print(url)

headers['sign'] = sign(headers['ocs'], headers['t'], headers['id'], url_str)
response = requests.get(url, params=params, headers=headers, allow_redirects=False)
# response = requests.get(url, headers=headers)

print(response.status_code)
print(response.headers['Location'])
# with open("ddd.apk", "wb") as f:
    # f.write(response.content)
```
