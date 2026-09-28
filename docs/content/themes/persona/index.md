
+++
title = "Persona"
description = "A portfolio, a resume, a blog, and a Zola theme."
template = "theme.html"
date = 2026-09-28T02:13:30Z

[taxonomies]
theme-tags = ['portfolio', 'resume', 'blog']

[extra]
created = 2026-09-28T02:13:30Z
updated = 2026-09-28T02:13:30Z
repository = "https://github.com/hanson-hschang/Persona-Zola-Theme.git"
homepage = "https://github.com/hanson-hschang/Persona-Zola-Theme"
minimum_version = "0.23.0"
license = "MIT"
demo = "https://hanson-hschang.github.io/Persona-Zola-Theme"

[extra.author]
name = "Chang, Heng-Sheng"
homepage = "https://hanson-hschang.github.io/"
+++        

<div align="center">

# `Persona` [Zola](https://www.getzola.org/) Theme

**a modern, responsive and lightweight theme for resume, portfolio, and blog**

[![pagespeed report](pagespeed-report.svg)](https://pagespeed.web.dev/)

<img alt="code-size" src="https://img.shields.io/github/languages/code-size/hanson-hschang/Persona-Zola-Theme">
<img alt="repo-size" src="https://img.shields.io/github/repo-size/hanson-hschang/Persona-Zola-Theme">
<img alt="GitHub Issues" src="https://img.shields.io/github/issues/hanson-hschang/Persona-Zola-Theme">
<img alt="GitHub Created At" src="https://img.shields.io/github/created-at/hanson-hschang/Persona-Zola-Theme">
<img alt="activity" src="https://img.shields.io/github/last-commit/hanson-hschang/Persona-Zola-Theme">
<img alt="Website" src="https://img.shields.io/website?url=https%3A%2F%2Fhanson-hschang.github.io%2FPersona-Zola-Theme">
<img alt="GitHub Release" src="https://img.shields.io/github/v/release/hanson-hschang/Persona-Zola-Theme">

Demo](https://hanson-hschang.github.io/Persona-Zola-Theme/) • [Features](#-features) • [Showcase](#-showcase) • [Installation](#-installation) • [Configuration](#%EF%B8%8F-configuration) • [Troubleshooting](#-troubleshooting) • [Credits

</div>

## ✨ Features

- 🎨 **Modern Design**: Clean, professional, and minimalist aesthetic
- 📱 **Fully Responsive**: Optimized for desktop, tablet, and mobile devices
- ⚡️ **Fast Performance**: Lightweight and optimized for speed
- 📋 **Resume/CV**: Build-in structure designed for customization
- 🎭 **Portfolio with Posts**: Showcase your work with previews and posts
- 📧 **Contact Forms**: Integrated contact form email support
- 🔍 **Search Ready**: Built-in search index generation *(--upcoming feature--)*


## 🌟 Showcase

Using `Persona` for your site? 
We'd love to see it! 
Submit a Pull Request to add your site to our showcase.

- Show Your Site - Add your site with us!
- [Hanson.HSChang](https://hanson-hschang.github.io/) - Personal website of Heng-Sheng Chang

## 🚀 Installation

This is a theme built with [Zola 0.23 or newer](https://www.getzola.org/documentation/getting-started/installation/).
Please proceed to install the theme after [initializing your Zola site](https://www.getzola.org/documentation/getting-started/overview/#initialize-site).

The easiest way to install the theme is to add it as a git submodule to your `themes` directory:

```bash
# Navigate to your Zola site directory
cd your-zola-site

# Add the theme as a submodule
git submodule add https://github.com/hanson-hschang/Persona-Zola-Theme.git themes/persona
```

To update the theme:

```bash
# Update the submodule
git submodule update --init --recursive
```

## ⚙️ Configuration

<div align="center">

Basic Setup](#basic-setup) • [Build & Serve

</div>



### Basic Setup

1. **Copy the configuration and the landing page content** from the theme to your website root to get started quickly:

    ```bash
    # Copy the configuration
    cp themes/persona/config.toml config.toml

    # Copy the landing page content
    cp themes/persona/content/_index.md content/_index.md
    ```

2. **Enable the theme** by adding the following line at the beginning of your `config.toml` file:

    ```toml
    theme = "persona"
    ```

### Build & Serve

After completing the setup, build and serve your site with Zola:

| |terminal command| note|
|---|---|---|
|Build the site|```zola build```|site build under `public/`|
|Serve the site|```zola serve```|locally with live reload|

> [!TIP]
> For a complete walkthrough of configuration and customization, see the [Begin with Persona](https://hanson-hschang.github.io/Persona-Zola-Theme/maps/private-soul/begin-with-persona/) post.

## 🆘 Troubleshooting

### Common Issues

- **Theme not loading:**
  - Ensure `theme = "persona"` is set at the first line of `config.toml`
  - Check that the theme is in the correct directory: `themes/persona/`

- **Contact form not working:**
  - Verify `web3form_public_key` is set in configuration
  - Check [Web3Forms documentation](https://docs.web3forms.com/) for setup

### Getting Help
- Read the [Zola documentation](https://www.getzola.org/documentation/)
- Check [ARCHITECTURE.md](ARCHITECTURE.md) for detailed theme codebase structure 
- Report issues or request features by [contributing](CONTRIBUTING.md) on GitHub


## 🙏 Credits

`Persona` is built with and inspired by:

| Resource | Description |
| --- | --- |
| [**Zola**](https://www.getzola.org/) | A fast static site generator |
| [**Bootstrap**](https://getbootstrap.com/) | CSS framework for responsive design |
| [**Web3Forms**](https://web3forms.com/) | Contact form service |
| [**KaTeX**](https://katex.org/) | Fast math typesetting library |
| [**Pandoc**](https://pandoc.org/) | Universal document converter for citation processing |
| [**Bootstrap Icons**](https://icons.getbootstrap.com/) | Icon library |
| [**Academicons**](https://jpswalsh.github.io/academicons/) | Academic icons |
| [**AOS (Animate On Scroll)**](https://michalsnik.github.io/aos/) | Animation library |
| [**Google Fonts**](https://fonts.google.com/) | Font library |
| [**Dynamic Badges**](https://shields.io/) | Customizable badges |
| [**PageSpeed Insights**](https://github.com/lowlighter/metrics/tree/master/source/plugins/pagespeed) | Automated performance reporting |
| [**GitHub**](https://github.com/features) | [Actions (CI/CD)](https://github.com/features/actions) • [Pages (Hosting Website)](https://docs.github.com/en/pages/getting-started-with-github-pages) • [Copilot (AI Agent)](https://docs.github.com/en/copilot/get-started/what-is-github-copilot) |
| [**Anthropic**](https://www.anthropic.com/) | [Claude](https://claude.ai/) • [Claude Code](https://code.claude.com/docs/en/overview) |
| [**Google**](https://cloud.google.com/ai/gemini) | [Gemini](https://gemini.google.com/) • [Gemini CLI](https://geminicli.com/) |
| [**OpenAI**](https://openai.com/) | [ChatGPT](https://chat.openai.com/) • [Codex](https://openai.com/codex/) |
| [**BootstrapMade Templates**](https://bootstrapmade.com/) | [Active](https://bootstrapmade.com/demo/Active) • [MyResume](https://bootstrapmade.com/demo/MyResume) • [UpConstruction](https://bootstrapmade.com/demo/UpConstruction) • [Style](https://bootstrapmade.com/demo/Style/) |
| [**Zola Themes**](https://www.getzola.org/themes/) | [Mabuya](https://mabuya.vercel.app/) • [Vonge](https://pascal-berrang.de/vonge-zola-theme/) • [Zluinav](https://harrymkt.github.io/zluinav/) |
| [**Academic Project Page Template**](https://github.com/eliahuhorwitz/Academic-project-page-template) | Post layout inspiration |


---

<div align="center">

**[⭐ Star this repository](https://github.com/hanson-hschang/Persona-Zola-Theme) if you find it helpful!**

</div>

        