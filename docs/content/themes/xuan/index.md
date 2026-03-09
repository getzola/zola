
+++
title = "xuan"
description = "Modern, pretty, and clean theme"
template = "theme.html"
date = 2026-02-25T00:58:14+08:00

[taxonomies]
theme-tags = []

[extra]
created = 2026-02-25T00:58:14+08:00
updated = 2026-02-25T00:58:14+08:00
repository = "https://github.com/jhq223/xuan.git"
homepage = "https://github.com/jhq223/xuan"
minimum_version = "0.21.0"
license = "MIT"
demo = "https://xuan-theme.jhq223.workers.dev"

[extra.author]
name = "jhq223"
homepage = "https://blog.guiio.com"
+++        

# Xuan

**Xuan** 是一个基于 [Duckquill](https://codeberg.org/daudix/duckquill) 二次开发的现代化 [Zola](https://www.getzola.org) 博客主题。

它保留了原版简洁、美观的设计理念，并在其基础上进行了深度定制，优化了阅读体验并集成了更强大的搜索功能。

## ✨ 主要特性与修改

相较于原版 Duckquill，Xuan 做了以下改进：

- **🎨 视觉风格**：
  - 代码高亮主题统一为 **One Dark Pro**。
  - 修改了分割线样式。
  - 移除原版所有用于定制的 "mods" 样式。
- **🧩 调整**：
  - 移除博客列表页的“按标签筛选”功能，界面更清爽。
  - 移除文章页移动端顶部的按钮列表，改为**右下角悬浮操作按钮**（返回顶部、主页等）。
  - 移除了alert和vimeo短代码。
- **🚀 功能增强**：
  - **新增归档页面**：按时间线查看所有文章。
  - **新增友情链接页面**：方便展示友链。
  - **全新搜索体验**：弃用原有搜索，改用 **[Pagefind](https://pagefind.app/)** 构建静态索引，搜索速度更快，支持中文分词，且无需前端加载大量 JS。

## 🛠️ 安装

将此主题克隆到你的 Zola 站点的 `themes` 目录下：

```bash
git clone https://github.com/jhq223/xuan.git themes/xuan
```

然后在你的 `config.toml` 中启用它：

```toml
theme = "xuan"
```

## 📄 许可证

本项目遵循 [MIT License](https://mit-license.org)。
基于 [Duckquill](https://codeberg.org/daudix/duckquill) 开发。

---
*由 [daudix](https://daudix.one) 原创设计，经过二次开发定制。*

        