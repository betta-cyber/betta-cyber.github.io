---
title: Blog devlog
abbrlink: c2305f46
date: 2023-01-18 09:00:00
---

# 简单记录一下写博客时碰到的一些问题

## 1. Dark mode

现在的站点基本上都适配了暗黑模式，我这个小博客也不能凑合着适配一下吧。使用了最简单的 CSS 代码，代码内容如下：

``` css
@media (prefers-color-scheme: dark) {
    html {
        filter: invert(90%) hue-rotate(180deg);
        background: aliceblue;
    }

    img, video, svg, iframe {
        filter: invert(110%) hue-rotate(180deg);
        opacity: .8;
    }
}
```

代码就是简单的通过 media 进行判断，然后 invert 一下，我自己对背景色做了个处理。

需要调试暗黑模式的话，控制台下 `command + shift + p` 输入 dark 或者 light 就可以相关的模拟选项。

`prefers-color-scheme: dark`
`prefers-color-scheme: light`

参考:
https://www.izhaoo.com/2020/11/12/adapt-to-dark-mode/
https://www.zhangxinxu.com/wordpress/2020/11/css-mix-blend-mode-filter-dark-theme/
https://blog.csdn.net/hiumanChung/article/details/120788659

## 2. Font

本站点采用的中文字体为[霞鹜文楷](https://github.com/lxgw/LxgwWenKai)，但是个人觉得在加载的时候字体有点大，对字体加载方面也不是很了解，又不想妥协采用其他的方案。就写了一点小程序，对原字体进行处理。

``` yml
name: github pages

on:
  push:
    branches:
    - dev

jobs:
  build-deploy:
    runs-on: ubuntu-18.04
    steps:
    - uses: actions/checkout@v1

    - name: Setup Node
      uses: actions/setup-node@v1
      with:
        node-version: '12.x'

    - name: Cache dependencies
      uses: actions/cache@v1
      with:
        path: ~/.npm
        key: ${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}
        restore-keys: |
          ${{ runner.os }}-node-

    - run: npm ci
    - run: npm install
    - run: npm install font-spider -g
    - run: npm run build
    - run: sed -i 's/\/css\/style.css/..\/css\/style.css/g' `grep ".html" -rl public`
    - run: font-spider "public/!(page)**/*.html" --debug
    - run: sed -i 's/..\/css\/style.css/\/css\/style.css/g' `grep ".html" -rl public`

    - name: Deploy
      uses: peaceiris/actions-gh-pages@v2
      env:
        ACTIONS_DEPLOY_KEY: ${{ secrets.ACTIONS_DEPLOY_KEY }}
        PUBLISH_BRANCH: master
        PUBLISH_DIR: ./public
```

这段是我的 Github Action 文件，我在这里使用了 [font-spider](https://github.com/aui/font-spider) 做一个字体处理，把我 html 文件中需要用到的文字全部提取出来，然后用 font-spider 进行一个全盘的抽取替换，最后再把之前的内容还原回去。这样字体库中就之后拥有我博客中需要的字了，多余用不到的字就被去掉了，体积大大缩减。本来 15M 的字体现在不到 1M ，满足了我的需求。

参考:
https://juejin.cn/post/6844904019060588551

## 3. Deploy

其实博客的部署很简单，代码也是参考上面的 Action 文件。平时写作用的是 dev 分支，部署的时候，通过 build 出相关的静态代码。放在 Github Pages 上。官方文档写得已经很清楚了。

参考：
https://hexo.io/zh-cn/docs/github-pages.html


## 4. Music share

可以看到我的博客有一项专栏，就是音乐，里面收藏了很多我喜欢的音乐专辑。

因为网易云的各种问题，在此就没有用网易云进行分享。而是使用 Spotify ，也方便了我在工作中随时通过这个点击到相关的专辑页面进行收听。

在 Spotify 中，打开任意一个歌单或者专辑，点击歌单内的「…」，在展开的菜单，选择「分享 >> 嵌入播放清单」。

![example](/article_photo/spotify-example.jpeg)

在弹出的面板，勾选右下角的「显示代码」，下方就会显示歌单的前端代码，点击「复制」，将代码粘贴到相关 Markdown 文件，就能在页面中看到了，整个操作过程就是这么简单。

参考：
https://penghh.fun/2022/09/10/2022-9-10-notion_music/


## 5. git action 调试

github action是git-ops中不可缺少的环节，使用在线环境只能查看logs，对于实际解决问题，起不到调试作用。这里请出`act`。它可以帮助我们在本地调试github action。

Install
``` bash
brew install act
```

Usage
``` bash
## 列出workflow
act -l
## act dryrun模式，列出执行顺序，但是不真正执行
act -n
```

``` bash
## mac m1上执行需要加额外参数指定架构
## 直接通过-s 来指定GITHUB_TOKEN
act --container-architecture linux/amd64 -s GITHUB_TOKEN=xxxx
```
就可以愉快的进行本地调试了，而不污染 git 仓库。
