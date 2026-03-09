
+++
title = "homepage-creators"
description = "A fast and beautiful personal homepage for creators, used by https://jiaxiang.wang, a port of HeoWeb."
template = "theme.html"
date = 2026-03-01T23:52:49+08:00

[taxonomies]
theme-tags = []

[extra]
created = 2026-03-01T23:52:49+08:00
updated = 2026-03-01T23:52:49+08:00
repository = "https://github.com/iWangJiaxiang/Homepage-Creators.git"
homepage = "https://github.com/iWangJiaxiang/homepage-for-creators"
minimum_version = "0.19.2"
license = "AGPL"
demo = "https://jiaxiang.wang/"

[extra.author]
name = "Jiaxiang Wang"
homepage = "https://blog.jiaxiang.wang"
+++        

<div align="center">

<h1>Homepage Creators</h1>

<p align="center">

主题预览](#-预览) | [快速上手](#-快速上手) | [加入讨论

[中文](https://github.com/iWangJiaxiang/Homepage-Creators/blob/main/README.md) | [English](https://github.com/iWangJiaxiang/Homepage-Creators/blob/main/README.en.md)

</p>
</div>

[![preview](https://github.com/iWangJiaxiang/Homepage-Creators/raw/refs/heads/main/screenshot.png)](https://jiaxiang.wang)

## 🔥 预览

| 站点名称 | 站点地址 |
|:------:|:-----------------------|
| 王嘉祥 个人主页 | [https://www.jiaxiang.wang](https://www.jiaxiang.wang) |


## ℹ️ 简介

[Homepage Creators](https://github.com/iWangJiaxiang/Homepage-Creators) 是一款适用于 [Zola](https://github.com/getzola/zola) 的个人主页主题，风格接近 Apple，美观大气。

> 注意：本主题移植于的开源 [HeoWeb](https://github.com/zhheo/HeoWeb) 纯静态主题，感谢 [张洪 Heo](https://blog.zhheo.com/) 的无私分享。

本主题使用简单，只需修改`config.toml`文件即可动态调整内容。无需像上游仓库一样修改 HTML 内容，极大降低用户的心智负担。

### 🔌 功能

特色功能

> 已完成所有功能移植

- [x] 基本功能
  - [x] 移动端自适应
  - [x] 动画滚动
  - [x] AVIF / WebP自适应
  - [x] 动态更新底部年份
  - [x] 访问量统计（Umami 或自定义）
  - [x] 多语言支持（i18n）
  - [x] 浏览器语言检测提示
- [x] 内容板块
  - [x] 导航菜单
  - [x] 首屏板块
  - [x] 作者板块
  - [x] 事件板块
  - [x] 产品板块（独立）
  - [x] 产品板块（清单）
- [x] 日常运营
  - [x] 置顶通知
- [x] 合规
  - [x] ICP备案
  - [x] 公安备案

## 📝 快速上手

本章节助你快速运行自己的主页网站，如果打算正式使用本主题，请参阅正式使用章节，能获得更好的主题版本更新体验

### 免费静态页面托管服务

#### GitHub Pages

1. [Fork](https://github.com/iWangJiaxiang/Homepage-Creators/fork) 本仓库。
1. 确保仓库已经包含 `.github/workflows/deploy.yml` 文件，无需额外配置。
1. 在仓库的 **Actions** 页面启用 `Build GH Pages` 工作流，然后手动触发构建。
1. 提交更改后，GitHub Actions 将自动构建并部署到 `gh-pages` 分支，等待完成。
1. 在您的 GitHub 仓库中，进入 **Settings** -> **Pages**，在 **Source** 下拉菜单中选择 `gh-pages` 分支并保存。
1. 部署完成后，您可以通过 `https://<your-username>.github.io/<repository-name>` 访问您的站点。
1. 参考定制主页章节，定制个人主页

#### CloudFlare Pages

1. [Folk](https://github.com/iWangJiaxiang/Homepage-Creators/fork) 本仓库
1. 登录 [Cloudflare](https://dash.cloudflare.com/) 并进入 **Pages** 页面。
1. 点击 **Create a project** 按钮。
1. 选择 **Connect to Git**，然后授权 Cloudflare 访问您的 GitHub 仓库。
1. 在仓库列表中选择您的 `Homepage-Creators` 仓库。
1. 配置构建设置：
  - **Framework preset**: 选择 `None`。
  - **Build command 构建命令**: 输入 `zola build`。
  - **Build output directory 构建输出**: 输入 `public`。
  - 添加环境变量`UNSTABLE_PRE_BUILD`，填写`asdf plugin add zola https://github.com/salasrod/asdf-zola && asdf install zola $ZOLA_VERSION && asdf global zola $ZOLA_VERSION`
  - 添加环境变量`ZOLA_VERSION`，填写`0.20.0`
  - 如果这里遇到问题，请参考[官方文档](https://www.getzola.org/documentation/deployment/cloudflare-pages/)
1. 点击 **Save and Deploy** 按钮，Cloudflare Pages 将开始构建和部署您的站点。
1. 部署完成后，您可以通过 Cloudflare 提供的域名访问您的站点，或者绑定自定义域名。
1. 参考定制主页章节，定制个人主页

### 本地部署

1. 参考[官方说明](https://www.getzola.org/documentation/getting-started/installation/)安装 Zola 命令行工具
1. 将本仓库克隆到本地

    ```bash
    git clone --depth=1 https://github.com/iWangJiaxiang/Homepage-Creators
    ```

1. 进入本地仓库

    ```bash
    cd Homepage-Creators
    ```

1. 运行预览命令，然后在浏览器打开提示的预览地址

    ```bash
    zola serve
    ```

    此时您应该成功访问到博客网站啦

1. 参考官方资料进一步并定制您的主页
   - [zola 命令说明](https://www.getzola.org/documentation/getting-started/cli-usage/)
   - [理解项目结构](https://www.getzola.org/documentation/getting-started/directory-structure/)
   - [自定义](https://www.getzola.org/documentation/getting-started/configuration/)

1. 参考定制主页章节，根据需要修改`config.toml`配置文件内容，您需要对 TOML 格式有基本的了解。

1. 根据需要将自己的图片素材放到`static/img`文件夹

## 正式使用

本章节提供的方案和直接修改本仓库代码的最大区别就是内容隔离。基于 Zola 博客框架的机制，将本仓库的代码安装为主题，这样主题更新和用户自己的修改将完全隔离，能够长期使用，避免产生技术债务。

正式使用时，假定你已经对 [Zola](https://github.com/getzola/zola) 框架和 Git Submodule 具备基础的了解，具体操作步骤如下：

1. 本地安装 Zola 命令行工具，参考[官方说明](https://www.getzola.org/documentation/getting-started/installation/)
1. 使用 `zola` 命令行本地初始化一个新的网站，也就是你的个人主页，并初始化为 Git 仓库。命令格式为`zola init <site name>`
1. 将本主题仓库作为 Git Submodule 安装在你的网站
   ```bash
   git submodule add -b main https://github.com/iWangJiaxiang/Homepage-Creators themes
   ```
   此时，你的仓库应该自动创建了`themes/Homepage-Creators`文件夹
1. 下载本仓库内容
   ```bash
   git submodule update --init
   ```
   此时，`themes/Homepage-Creators`文件夹应该具有内容了
2. 配置新网站使用本主题，修改`config.toml`文件的属性`theme = "Homepage-Creators"`
3. 参考定制主页章节，根据需要修改`config.toml`配置文件内容，您需要对 TOML 格式有基本的了解。

之后，你的个人主页网站就可以作为一个单独的 Git 仓库维护管理了。

如果需要更新主题，只需要更新 Git Submodule 的分支/标签/代码即可。

## 定制主页

定制过程十分简单！无需改一行代码，因为我已经对板块进行模块化抽取，你只需要：

1. 将使用到的图片素材并存放在`static/img`文件夹（最费时且最难的工作其实是图片制作……）
2. 修改 `config.toml` 文件，配置板块、文字内容和引用的图片
3. 运行`zola serve`命令，本地预览主页，支持实时刷新

要进行主页定制，你需要对 Zola 框架具有基本的了解，例如[理解项目结构](https://www.getzola.org/documentation/getting-started/directory-structure/)和[配置文件](https://www.getzola.org/documentation/getting-started/configuration/)，这些内容十分简单，只需要通读一遍即可。

> **V2 新版说明**：内容配置现在支持两种方式：
> 1. **`config.toml`**（传统方式）：将所有板块写在 `config.toml` 中，**仍然完全兼容**
> 2. **`content/_index.md`**（推荐方式）：将板块和导航配置写在 `content/_index.md` 的 front-matter `[extra]` 中，支持多语言

### 基础配置

在 `config.toml` 文件中可以对网站信息进行设置，说明如下

```toml
[extra.site]
# 建站年份，用于底部版权内容生成
start_year = 2024
# 网站 Logo
logo = "/img/logo.webp"
# 导航栏 Logo，留空则默认为网站 Logo
nav_logo = "/img/logo.webp"
# 联系邮箱
mail = "contact@example.com"
# ICP 备案号
compliance_icp = "ICP备XXXXXXXX号"
# 公安备案号
compliance_security = "公网安备0000000000号"
# 公安备案链接
compliance_security_link = "https://www.beian.gov.cn/portal/registerSystemInfo?recordcode=0000000000"

[extra.other]
# 是否启用 AVIF 图片格式转换，大幅降低图片尺寸
avif_enable = true
```

### 导航菜单

在 `config.toml` 文件中可以对**顶部导航栏**和**通知**的内容进行设置，说明如下

```toml
[extra.nav]

# 导航栏下方弹出通知
[extra.nav.message]
enable = true
# support inline html
text = "🎉 访问作者博客"
url = "https://blog.jiaxiang.wang"

# 中间导航栏配置
[extra.nav.center]
menus = [
    # 设置 internal = true，url 为文字的表示内部导航，页面滑动到特定板块的区域
    { name = "主页", url = "主页", internal = true},
    { name = "主题", url = "主题", internal = true},
    { name = "博客", url = "博客", internal = true},
    { name = "媒体", url = "媒体", internal = true},
    # 设置 internal = true，url 为链接的表示外部跳转
    { name = "项目", url = "https://blog.jiaxiang.wang/tags/project/", internal = false},
]

# 右侧导航栏配置
[extra.nav.right]
menus = [
    { name = "作者博客", url = "https://blog.jiaxiang.wang", internal = false},
    { name = "作者 Github", url = "https://github.com/iWangJiaxiang", internal = false},
]
```

### 内容板块

你可以根据需求对板块进行灵活定制，除了顶部导航栏之外，页面的任何板块都是**模块化组件**，可以**无限自定义顺序、数量**，随心所需，做你所想！

板块的使用十分简单，将**板块的配置代码粘贴在 `config.toml` 的最后部分**，也就是 `[extra.other]` Section 的后面即可。配置代码的第一行统一为 `[[extra.index.widgets]]`，`[[ ]]`在 Toml 语法中表示数组的意思

**板块的顺序和添加的顺序一致**，也就是说，你可以在 `config.toml` 文件中通过复制粘贴调整代码顺序以达到控制页面展示顺序的目的。

如果有疑惑的地方，最快的方法是参考本项目的 [config.toml 文件](https://github.com/iWangJiaxiang/Homepage-Creators/blob/main/config.toml)，相信你会有所收获。

目前支持的模块化组件如下：

#### 模块化组件板块：首屏内容

配置代码

```toml
[[extra.index.widgets]]
# 重要，不要修改
type = "header"
[extra.index.widgets.value]
title_1 = "大标题1"
title_2 = "大标题2"
bio_1 = "这行描述有<span class=\"inline-word\">高亮文字</span>"
bio_2 = "另一行描述"
# “了解更多”按钮的链接
about_url = "https://blog.jiaxiang.wang/about/"
# 右侧的图片
cover = "/img/logo.svg"
# 下面是“了解更多”按钮旁边的小按钮，可根据需求增删，通常放社交媒体链接
[[extra.index.widgets.value.links]]
class_icon = " icon-github-line"
url = "https://github.com/iWangJiaxiang"
[[extra.index.widgets.value.links]]
class_icon = " icon-github-line"
url = "https://github.com/iWangJiaxiang"
```

截图（仅展示比较美观的效果，可能和配置代码内容无关）

![首屏内容截图](./docs/header.png)

#### 模块化组件板块：作者介绍

配置代码

```toml
[[extra.index.widgets]]
# 重要，不要修改
type = "author"
[extra.index.widgets.value]
# 姓名
name = "站长"
# 头像，图片放在 /static/img 文件夹，此处格式开头为：/img/
avatar = "/img/logo01.webp"
title = "Team leader, architect,"
# 个人介绍或者个人的想法
bio = "站长的介绍有点短哦~"
```

截图（仅展示比较美观的效果，可能和配置代码内容无关）

![作者介绍截图](./docs/author.png)

#### 模块化组件板块：产品单页

可用来展示个人项目、作品、成果等

配置代码

```toml
[[extra.index.widgets]]
# 重要，不要修改
type = "product-single"
[extra.index.widgets.value]
# 文字内容根据实际需要修改
tip = "主页"
title = "个人主页<br>正式开源"
bio_1 = "像本页面一样，呈现美轮美奂<span class=\"inline-word\">效果</span>"
bio_2 = "轻松配置，快速构建你的<span class=\"inline-word\">个人主页</span>"
# 产品大图，图片放在 /static/img 文件夹，此处格式开头为：/img/
img = "/img/homepage-single.avif"
# 产品按钮列表
[[extra.index.widgets.value.links]]
# 样式，支持 primary-button 和 second-link 两种
class = "primary-button"
# 链接
url = "https://github.com/iWangJiaxiang/Homepage-Creators"
# 显示名称
name = "立刻获取源代码"
[[extra.index.widgets.value.links]]
class = "second-link"
url = "https://github.com/iWangJiaxiang"
name = "开发者主页"
```

截图（仅展示比较美观的效果，可能和配置代码内容无关）


![产品单页截图](./docs/product-single.png)

#### 模块化组件板块：产品列表

用于展示一系列内容

配置代码

```toml
[[extra.index.widgets]]
# 重要，不要修改
type = "product-list"
[extra.index.widgets.value]
# 文字内容根据实际需要修改
title = "媒体"
bio = "为互联网共享精神添砖加瓦"
[[extra.index.widgets.value.items]]
# 产品 logo，图片放在 /static/img 文件夹，此处格式开头为：/img/
logo = "/img/internet.svg"
# 文字内容根据实际需要修改
title = "个人博客"
bio = "个人博客的介绍文字"
# 按钮配置
url = "https://blog.jiaxiang.wang/"
button = "访问"
# 显示热门标签
hot = true
[[extra.index.widgets.value.items]]
logo = "/img/wechat.svg"
title = "公众号"
bio = "第一时间获取动态"
url = "https://blog.jiaxiang.wang/wechat"
button = "访问"

```

截图（仅展示比较美观的效果，可能和配置代码内容无关）

![产品列表截图](./docs/product-list.png)

#### 模块化组件板块：推荐文章

支持多栏分组，用于推荐多个维度的精彩文章。文章**列表超过 3 条时还会自动向上循环滚动**。

配置代码

```toml
[[extra.index.widgets]]
# 重要，不要修改
type = "featured-posts"
[extra.index.widgets.value]
# 左侧大标题与简介
title = "推荐文章"
bio = "精选好文，值得一读"
# 背景样式设置，可为 CSS 背景属性（如渐变色）
style = "background: linear-gradient(180deg, #f5f7fa 0%, #c3cfe2 100%);"
# 每组文章占用单独一栏（桌面端最多一行3栏）
[[extra.index.widgets.value.columns]]
title = "🔧 科技"
[[extra.index.widgets.value.columns.items]]
title = "Pangolin：基于零信任理念的反向代理"
url = "https://blog.jiaxiang.wang/articles/pangolin-a-reverse-proxy-for-zero-trust-network/"
[[extra.index.widgets.value.columns.items]]
title = "GitHub Action：让静态网站实现定时发布"
url = "https://blog.jiaxiang.wang/articles/github-action-makes-static-site-publish-on-schedule/"
# 第二栏
[[extra.index.widgets.value.columns]]
title = "🎵 音乐"
[[extra.index.widgets.value.columns.items]]
title = "前无古人唱超算，《超算为家国天下》"
url = "https://blog.jiaxiang.wang/articles/sc-song/"
```

截图（仅展示比较美观的效果，可能和配置代码内容无关）

![推荐文章截图](./docs/featured-posts.png)

#### 模块化组件板块：重要事件

展示重要活动、大事件等

配置代码

```toml
[[extra.index.widgets]]
# 重要，不要修改
type = "event"
[extra.index.widgets.value]
# 文字内容根据实际需要修改
tip = "大事件"
title = "王嘉祥博客主题<br>正式开源！"
bio = "一个为创作者而构建的主题，0成本、0运维开始你的博客生涯，与各位优秀博主一同进步！"
button = "立刻获取源代码"
note = "基于 Zola 构建"
url = "https://github.com/iWangJiaxiang/zola-theme-jiaxiang.wang"
# 背景图，图片放在 /static/img 文件夹，此处格式开头为：/img/
img = "/img/blog-event.avif"
```

截图（仅展示比较美观的效果，可能和配置代码内容无关）

![产品列表截图](./docs/event.png)

## 🌐 多语言

本主题支持多语言，默认包含中文和英文。

### 工作原理

每种语言的内容（板块、导航、UI字符串）存放在对应的 `content/_index.[lang].md` 文件的 `[extra]` front-matter 中：

```
content/
  _index.md        ← 中文（默认语言）
  _index.en.md     ← 英文
```

`config.toml` 仅保存通用配置（logo、备案号等）和语言检测提示字典。

### 添加新语言

以添加日语为例：

1. 在 `config.toml` 注册语言：
   ```toml
   [languages.ja]
   title = "ホームページ"
   ```

2. 添加语言检测提示（`config.toml`）：
   ```toml
   [extra.i18n_detect.ja]
   message = "このページは日本語でもご覧いただけます"
   action = "切替"
   url = "/ja/"
   ```

3. 创建 `content/_index.ja.md`，从 `_index.en.md` 复制结构并翻译内容

### 向前兼容

如果你仍然在 `config.toml` 中配置板块内容（旧方式），默认语言页面会优先使用 `config.toml` 中的内容，确保升级主题后无需任何改动。

## 💬 讨论

如果你对主题有什么建议或者意见，欢迎提 PR & Issue。

## 🔐 许可

[Homepage Creators](https://github.com/iWangJiaxiang/Homepage-Creators) 使用 [AGPL](./LICENSE) 协议开源，请遵守开源协议。

## 📝 致谢

该项目的CDN加速和安全防护由[腾讯EdgeOne](https://edgeone.ai/?from=github)赞助。

CDN acceleration and security protection for this project are sponsored by [Tencent EdgeOne](https://edgeone.ai/?from=github).

[![Edge One](https://edgeone.ai/media/34fe3a45-492d-4ea4-ae5d-ea1087ca7b4b.png)](https://edgeone.ai/?from=github)

        