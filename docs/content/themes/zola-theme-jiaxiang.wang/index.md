
+++
title = "jiaxiang.wang"
description = "A fast and beautiful theme for creators, used by https://blog.jiaxiang.wang, porting from Halo-theme-hao."
template = "theme.html"
date = 2024-12-23T00:33:31+08:00

[taxonomies]
theme-tags = []

[extra]
created = 2024-12-23T00:33:31+08:00
updated = 2024-12-23T00:33:31+08:00
repository = "https://github.com/iWangJiaxiang/zola-theme-jiaxiang.wang"
homepage = "https://github.com/iWangJiaxiang/zola-theme-jiaxiang.wang"
minimum_version = "0.19.2"
license = "AGPL"
demo = "https://theme.jiaxiang.wang/"

[extra.author]
name = "Jiaxiang Wang"
homepage = "https://blog.jiaxiang.wang"
+++        

<div align="center">

<h1>Zola Theme for jiaxiang.wang [WIP]</h1>

<h4>为创造者而生的开源 Zola 主题</h4>

<p align="center">

主题预览](#-预览) | [快速上手](#-快速上手) | [加入讨论

[中文](./README.md) | [English](./README.en.md)

</p>
</div>

[![preview](<./content/articles/docs/01 Read Me/preview.webp>)](https://blog.jiaxiang.wang)

## ℹ️ 简介

[zola-theme-jiaxiang.wang](https://github.com/iWangJiaxiang/zola-theme-jiaxiang.wang) 是一款适用于 [Zola](https://github.com/getzola/zola) 的博客主题。

> 本主题由 [王嘉祥](https://blog.jiaxiang.wang) 移植自 [Halo](https://github.com/halo-dev/halo) 主题 [Halo-Theme-Hao](https://github.com/chengzhongxue/halo-theme-hao)，[Halo-Theme-Hao](https://github.com/chengzhongxue/halo-theme-hao) 主题移植自 [Hexo](https://hexo.io/zh-cn/index.html) 社区中 [张洪 Heo](https://blog.zhheo.com/) 对 [Hexo-Theme-Butterfly](https://github.com/chengzhongxue/halo-theme-hao) 主题的魔改与重设计版本。

## 🔥 预览

> 如果你的站点也使用了本主题，欢迎通过 PR 的形式在 readme.md 和 /static/data/friends.json 中添加站点信息

|  站点名称  |          站点地址           |
|:------:|:-----------------------|
| 王嘉祥 | https://blog.jiaxiang.wang |

### 🔌 功能

> 由于精力有限。目前只移植了主要功能，更多功能持续更新。

- [x] 基本功能
  - [x] 评论
    - [x] Twikoo
    - [ ] Artalk
    - [ ] Waline
  - [x] 搜索（Algolia）
  - [x] Markdown
  - [x] Katex
  - [x] Prism 代码高亮
  - [x] 随机访问文章
  - [x] 右键菜单
  - [x] Pjax
  - [ ] i18n国际化
  - [ ] Post GPT
- [ ] 特殊页面
  - [x] 个人装备
  - [x] 友链
  - [ ] 赞赏
    - [ ] 爱发电
  - [x] 订阅
  - [x] 关于我
  - [x] 音乐
  - [ ] 留言板
- [x] 日常运营
  - [x] 每日推荐
  - [x] 文章推荐
- [x] 文章
  - [x] 图片压缩
  - [x] 公众号链接
  - [x] 动态目录
  - [x] 分类/标签
  - [x] 访问热度（基于Twikoo）
  - [x] 相关文章
- [x] 合规
  - [x] ICP备案
  - [x] 公安备案

## 📝 快速上手

1. 参考[官方说明](https://www.getzola.org/documentation/getting-started/installation/)安装 Zola 命令行工具
1. 将本仓库克隆到本地

    ```bash
    git clone --depth=1 https://github.com/iWangJiaxiang/zola-theme-jiaxiang.wang.git
    ```

1. 进入本地仓库

    ```bash
    cd zola-theme-jiaxiang.wang
    ```

1. 运行预览命令，然后在浏览器打开提示的预览地址

    ```bash
    zola serve
    ```

    此时您应该成功访问到博客网站啦

1. 参考官方资料进一步探索并定制您的博客
   - [zola 命令说明](https://www.getzola.org/documentation/getting-started/cli-usage/)
   - [理解项目结构](https://www.getzola.org/documentation/getting-started/directory-structure/)
   - [自定义](https://www.getzola.org/documentation/getting-started/configuration/)

## 💬 讨论

如果你对主题有什么建议或者意见，欢迎提 PR & Issue。

## 🔐 许可

[Zola-Theme-Jiaxiang](https://github.com/iWangJiaxiang/zola-theme-jiaxiang.wang) 使用 [AGPL](./LICENSE) 协议开源，请遵守开源协议。


        