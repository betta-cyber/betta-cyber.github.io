---
title: 桌面客户端及 Webview 相关实践
date: 2020-02-14 22:08:14
---

## 问题由来

最近在探索GUI开发的新方式，也不能算是新方式吧，只能说是一种新的尝试，去解决一些痛点。传统GUI开发比较成熟：

- 传统的QT开发可以用C++，python等语言来写，性能也还不错，但是学习QT的样式来做样式的美化需要大量时间。

- .net的那一套也还不错，就是用的人比较少，而且也不能跨平台。

- 一些新项目用electron类似的技术用html+css+js这种web的形式来构建桌面应用。但是electron这种项目背后承载着整个v8引擎。封装好之后的应用体积较大，性能也很一般。

这次我要实践的是我在学习rust过程当中发现的一个名叫[web-view](https://github.com/Boscop/web-view)的库。当时是准备用来学习wasm和[yew](https://github.com/yewstack/yew)的。

后面发现在linux/macOS上面打包生成出来的二进制文件体积很小，且拥有非常强劲的性能。固一开始尝试配合yew去写一些高性能体积小的客户端。比如斗鱼直播间的客户端应用，但是在实现解码flv文件的时候因为一些事情暂时搁置了。但是在公司项目当中我也发现了这样的一些痛点。所以重新又开始做这方面工作。

## 实践部分

考虑到在新项目使用环境当中主要是windows系统。所以需要在windows下面进行编译及测试。当然了由于个人还是比较熟悉linux下开发。故开发工作还是放到linux下进行。

web-view是基于[webview](https://github.com/zserge/webview)这个体积小跨平台的GUI库的跨平台实现。他也有python、java、Haskell、C#等实现，但我并没有去测试它们。

web-view在macOS上面采用Cocoa/WebKit，在linux上面采用gtk-webkit2，在windows上面采用MSHTML (IE10/11) 作为其底层的渲染技术。


## 代码部分

### 前端部分

首先其实你可以把它当中一个web来做，但是是有限制的web，你最好考虑好你的一个应用的长宽和比例，然后一个样式的风格统一。这边我选择了一下。如果采用jquery+bootstrap这种老一套的方式来开发也不是不行，但是已经过时了，而且bootstrap的样式确实不太适合作为桌面应用的UI，这边我后面多方面考虑选择了饿了吗的[element](https://element.eleme.cn/#/zh-CN)作为桌面UI部分的组件。两个原因：
- 一个是风格比较统一，也是个桌面应用
- 二是基于vue.js编写，适合新人学习

### 后端部分

后端部分其实随便你用什么。他仅仅只是一个server，你可以用go，用python，用java，用rust等等等等都可以，只要你最后能把它打成一个服务。我本来是想用rust实现的，但是考虑到以后在组内做一个推广需要一个简单易学的方式，所以这次选择了python。

用python实现一个server是再基础不过的事情了，我这里随便选择了一个tornado。然后编写相关代码与前端调试成功。开始打包。

python的打包方案其实不多，这里面就用我比较熟悉用的最多的`pyinstaller`，pip安装之后，直接执行`pyinstaller
 -F app.py`即可开始打包，注意！app.py是你的入口文件，也就是你要把服务启动起来的文件。他会自动去寻找相关依赖，然后进行打包，当然打包的过程中也可以做很多优化，这里暂时不讨论了。当然打包需要在windows上面进行，这点有点不是很舒服，之后再研究一下。看能不能直接打包出exe文件

### web-view部分

web-view部分其实比较简单，说白了就是和electron一样的一个浏览器壳，但是又不是浏览器壳。webview有 C/C++/Golang的实现。通过引入相关头文件对页面进行渲染。而且也有一些坑。

主体代码其实并不能，就是通过启一个线程去运行python实现的后端程序

```rust
thread::spawn(|| {
    Command::new("cmd")
        .args(&["/C", "cd gcheck && app.exe"])
        .output()
        .expect("failed to execute process");
});
```

然后通过这边webview去请求该页面去进行交互生成桌面应用
```rust
run(
    "",
    Content::Url(format!("http://127.0.0.1:8000")),
    Some(size),
    resizable,
    debug,
    titlebar_transparent,
    move |mut webview| {
        // webview.set_background_color(0.11, 0.12, 0.13, 1.0);
    },
    frontend_cb,
    userdata
);
```

在windows下面有一些坑，下面主要列一下：

1. ie浏览器的兼容性问题，由于win下面使用的MSHTML作为其底层技术，那么浏览器的一个兼容问题是必须要考虑的。我之前就是因为一些js的写法导致出现了`script 1003` `script 1002`这样的一些脚本错误。归根结底的一个问题就是ie对ES6语法的支持不行，需要采用ES5的写法。最好的解决方案就是在vue项目当中`npm install --save-dev babel-polyfill` 然后在项目中`import 'babel-polyfill'`引入。

2. windows下面会出现dpi模糊的问题，这个问题已经出现了一段时间了，并且在github上面有很多讨论。也有相关朋友在提交一些pr 如[https://github.com/Boscop/web-view/pull/117](https://github.com/Boscop/web-view/pull/117)和[https://github.com/Boscop/web-view/pull/134](https://github.com/Boscop/web-view/pull/134)。但是问题始终还没有解决。同样webview项目也拉了一个分支webview-x来解决这样一些windows上面的问题。需要我后续继续跟进。

3. windows下面还可以使用edge在作为其底层的渲染技术。但是需要安装edge相关开发套件，这个我还没折腾完。我主要想看的是我用edge的features去编译好的exe能否在没有edge的设备上运行。这是一个兼容性的问题。

4. 编译问题，我用官方的rust的web-view库在windows上去`cargo run`没什么问题，但是`cargo build --release`出来的exe就有问题，运行不了，我在github上面也提了issue，目前还没有人回我。目前的解决方案是使用 `web-view = { git = "https://github.com/huytd/web-view" }` 作为web-view依赖。

5. 待补充

### 结论部分

说实话，这套方案其实还不错，但是也就仅仅说是还不错的范畴，如果我可以把开发路线固定好，打包流程自动化，开发者只需要去写相关的web部分然后打包发布。其实还不错，比较体积很小了，性能兼容性也还不错。

后续准备尝试一下[tauri](https://github.com/tauri-apps/tauri)，这是一套干我前面所说的这一套活的一个工具，这套工具能跑通了，那后面的开发维护就不需要那么大的精力了。

还需要解决的问题，打包出来的文件，一个更加极限的体积优化问题。设计到rust的编译和python程序的体积优化，两个方面，再一个就是加密问题，需要对程序做一定的加密，设计到一个加密方案的选择问题。
