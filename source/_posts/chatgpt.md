---
title: 从 ChatGPT 说起
abbrlink: f30d1c94
date: 2023-02-11 09:00:00
---

ChatGPT 自 2022 年末大火一波之后，最近又突然爆火，第一次是互联网人之间的大火，对于公众来说，消息还比较闭塞，可能是因为墙的问题，可能是因为行业的问题。总之，这一次爆火让全面大众也认识到了 ChatGPT 的魅力。

为了让我们的小伙伴们也使用上 ChatGPT，当然主要的问题是国内 IP 被限制访问了。以及注册账号需要国外手机号。导致普通用户想体验一下 ChatGPT 困难重重。

本人决定用微信接入一个 ChatGPT 的账号，然后把接入的账号，作为机器人拉入到群里面供大家玩耍。后面选择了 Github 上面的一个开源项目 [wechat-chatgpt](https://github.com/fuergaosi233/wechat-chatgpt) 进行接入。这个项目用 Nodejs 编写的，而且有比较详细的文档。文档里面提到用 railway 可以自行部署。

于是测试了一下 railway 平台，简单来说就是一个 Docker 的部署平台，把 Dockerfile 写好，关联一个 Github 就可以之间触发部署了。

但是在这个项目中，有很多的坑，有对中国的地区 IP 的限制问题，有微信的接入问题，有 ChatGPT 自身的拥堵问题。后面都踩了一遍，就简单记录一下。

1. Wechaty 的问题。

```
const bot =  WechatyBuilder.build({
  name: "wechat-assistant", // generate xxxx.memory-card.json and save login data for the next login
  puppetOptions: {
    uos: true, // 开启uos协议
  },
  puppet: "wechaty-puppet-wechat",
});
```
这段代码在 railway 上面部署的时候，出现 timeout
```
🤖️ Start GPT Bot Success, ready to handle message!
00:37:17 WARN WechatyPuppetMixin start() starting puppet ... timeout
00:37:17 WARN WechatyPuppetMixin start() puppet info: Puppet(wechat-assistant)
00:38:02 ERR PuppetWeChatBridge start() exception: TimeoutError: Navigation timeout of 60000 ms exceeded
00:38:02 ERR PuppetWeChat initBridge() exception: Navigation timeout of 60000 ms exceeded
00:38:02 WARN PuppetWeChatBridge stop() page.close() exception: Error: Protocol error: Connection closed. Most likely the page has been closed.
```
后面我把 uos 协议给关了，本地可以，但是 railway 上还是不行。改成了下面的代码：
```
const bot =  WechatyBuilder.singleton();
```
这个模式可以出现微信登录二维码，但是这个用的是 web 版本的协议。我在使用了大半天之后，导致一个微信小号被封，虽然可以解封，但是还是很麻烦。后面我研究了一下微信的机器人教程，机器人场景的使用最好使用 ipad 协议。
```
const puppet = new PuppetPadlocal({
    token: "puppet_padlocal_xxxxx"
})

const bot =  WechatyBuilder.build({
  name: "wechat-assistant",
  puppet
});
```
这样就可以比较完美的解决微信方面的问题。唯独就是需要付费购买 token。

[https://github.com/wechaty/puppet-padlocal](https://github.com/wechaty/puppet-padlocal)

PadLocal 最大的特点是：
- 账号状态的托管方式
- 与 WServer 的通信方式


其他 puppet 设计思路大多是这样：由 puppet server 进行管理和维持托管账号的状态。所有的请求都是通过 puppet -> puppet server -> WServer 这样一条链路完成。消息推送部分，puppet 和 puppet server 之间建立长连接，同时 puppet server 和 WServer 也建立对应的长连接。当有新消息推送的时候，是通过 WServer -> puppet server ->  puppet 这样的链路到达 puppet 端。这样的设计中 puppet server 就充当了一种有状态的代理角色，所有流量都是由服务器完成转发。在我们看来这样的设计可能有几个潜在的劣势：

因为最终和 WServer 通信的都是 puppet server。如果一个 puppet server 上托管了多个账号，且没有对各个账号配置对应的代理策略，那么这些账号将共享 puppet server 的 IP。从风控角度来看，容易产生风险。而且一旦其中某些账号风险等级比较高，容易对同一个 IP 池的其他账号造成污染，伤及无辜。
所有流量都是通过 puppet server 转发，对其带宽产生了不小压力，特别是当托管账号中产生了大量图片、视频等多媒体资源时。
由于 puppet server 维护了托管账号状态，所以 puppet server 是有状态的。从系统架构角度来看，有状态的服务器在系统稳定性、可用性、容量规划等方面都存在不小挑战。如果集群中某些服务器宕机，而备机切换机制设计不够完善的话，容易出现部分账号处于不可用的状态。

为了保证 puppet 有更好的可用性和体验，通常 puppet server 会缓存（不一定永久保存）某些数据（比如聊天数据）。也就是说，服务端无可避免地需要触碰托管账号的业务数据。这就需要 puppet 的提供者保持极高的行业自律，而且通过充分的机制保证客户数据的安全性。

基于对以上这些问题的思考，我们将所有流量转发工作都放在了 puppet 来做，这就是 PadLocal 中 Local 的来源。我们利用了 GRPC 的双向通信机制，让 puppet 成为代理，将所有流量通过 puppet 转发给 WServer。同时由 puppet 来维持和 WServer 之间的长连接。这样的好处显而易见：

托管账号和 WServer 通信所使用的 IP 都是 puppet 端的 IP，不同账号天然就不存在共享 IP 的风险。
下载图片、视频等多媒体资源的流量不需要经过 PadLocal server。而且不经过服务器，效率也更高。
账号状态维护在 puppet 端完成，于是 PadLocal server 就可以设计为 stateless 的了，应对扩容等问题天然就会简单很多，simple is beautiful。
PadLocal server 不会保存任何业务数据，没有数据安全方面风险。

![padlocal](/article_photo/padlocal.png)


2. Chrome 问题

有时候会存在 CloudFlare 人机验证, 如果出现了CloudFlare的人机验证, 则可能导致 Headless 浏览器无法成功模拟登录。

该版本使用 Puppeteer 来尽可能的实现全自动化, 包括验证码 Cloudflare CAPTCHAs是默认处理的, 但如果你想自动处理 邮箱+密码 的Recaptchas, 则需要使用付费的打码平台, 如 nopecha 或者 2captcha, 需要设定一些环境变量。

后面考虑到这个项目是实时拉取网页版的答案，不太稳定，至少是对于某些没有稳定梯子的人来说，所以，我后面参考 [这个PR](https://github.com/fuergaosi233/wechat-chatgpt/pull/574)，将代码改成了 API 的形式调用，部署在白嫖的 railway 上面。api-key的方式是接入的OpenAI的API模型，而网页版的还没有公开 API，只能通过网页登陆。OpenAI 已经公开的 API 模型最新的是 GPT-3，对话能力不如网页版的。只能说凑合着先用一下。


3. 账号问题

- 科学上网
- 外国邮箱？
- 接码平台 https://sms-activate.org/

4. 使用问题

[ChatGPT 中文调教指南](https://github.com/PlexPt/awesome-chatgpt-prompts-zh)

