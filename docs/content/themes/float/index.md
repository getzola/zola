
+++
title = "Float"
description = "An elegant blog theme"
template = "theme.html"
date = 2021-02-18T22:27:50+01:00

[extra]
created = 2021-02-18T22:27:50+01:00
updated = 2021-02-18T22:27:50+01:00
repository = "https://gitlab.com/float-theme/float.git"
homepage = "https://float-theme.netlify.app/"
minimum_version = "0.11.0"
license = "MIT"
demo = ""

[extra.author]
name = "Leon"
homepage = "https://exp2.uniuni.space/"
+++        

![Float](content/blog/2020/2020-06-14-Float theme for Zola/Float.png)

**[English](README.en.md)**

Float 是一款為 [Zola](https://www.getzola.org/) 設計的佈景主題。

[[_TOC_]]

## 特色

- 依據不同的螢幕尺寸提供最佳化版面，從小尺寸到大尺寸都可獲得優秀的閱讀體驗。
- 文章卡片提供兩種卡片尺寸，重點文章可採用更醒目的寬版卡片。
- 文章卡片配圖可自行指定，未指定者使用 [Unsplash Source](https://source.unsplash.com/) 的隨機圖片。
- 使用 Zola 的 `resize_image()` 自動產生適用於 DPR 1.0 ~ 3.0 的圖片，卡片配圖會由瀏覽器依據設備之 DPR 自動選用最佳尺寸的圖片。
- 圖片啟用延遲載入，縮短頁面載入時間。
- 預設埋入 HTML SEO 標籤、[Open Graph](https://ogp.me/) 與 [Twitter Cards](https://developer.twitter.com/en/docs/tweets/optimize-with-cards/overview/abouts-cards) 標籤。
- 整合 [Google Analytics](https://analytics.google.com/)。
- 整合 [Google AdSense](https://adsense.google.com/)。
- 版面為 [AdSense 自動廣告](https://support.google.com/adsense/answer/9261306)最佳化，不會因自動廣告的寬度不一而破版。
- 整合 [LikeCoin](https://like.co/)。
- 整合 [utterances](https://utteranc.es/)，利用 [GitHub](https://github.com/) issue 作為留言系統。

## 安裝與啟用

在您的 Zola 專案資料夾內：

把 Float 以 Git 子模組的方式加入專案內：
```shell
git submodule add https://gitlab.com/float-theme/float.git theme/float
```

編輯您的 config.toml，指定 Float 作為佈景主題：

```TOML
theme = "float"
```

編輯您的 config.toml，加入 tags 作為分類系統：

```TOML
taxonomies = [
    {name = "tags", paginate_by = 10},
]
```

複製 float/static/ 的所有子資料夾與檔案到您的 static/：

```shell
cp -r themes/float/static/* static/
```

複製 float/content/ 的所有子資料夾與檔案到您的 content/：

```shell
cp -r themes/float/content/* content/
```

## 使用 Float

### 文章與配圖

文章皆以資料夾的方式存在，如下例：

```
content/
└── blog/
    └── 2020/
        └── 2020-06-21-Float theme for Zola/
            ├── index.md
            ├── pic1.png
            ├── pic2.png
            └── qa_report.pdf
```

文章為 index.md，文內的配圖或其它檔案也是放在文章資料夾內。

### Front-matter

Front-matter 請參照下列註解說明：

```TOML
title = "Float theme for Zola"
description = "Float features and usage guide"
draft = false
[taxonomies]
tags = ["Float", "Zola"]
[extra]
feature_image = "pic1.png" # 卡片圖片。
feature = true # 是否為重點文章，重點文章會以寬版卡片顯示。
link = "" # 指定卡片連結，若有指定則卡片不會連結到文章頁。
```

## 客製化

可客製化設定大多可以在 config.toml 的 `[extra]` 區段做設定：

```TOML
[extra]
main_section = "blog"

copyright = ""

web_fonts = "<link rel='stylesheet' href='https://fonts.googleapis.com/css2?family=Noto+Serif+TC:wght@500;700&display=swap'>"

google_analytics = false
# google_analytics_id = "UA-XXXXXX-X"

google_adsense = false
# google_adsense_id = "ca-pub-XXXXXXXXXXXXXXXX"

twitter_account = "@xxx"

likecoin = false
# likecoin_name = "xxx"

utterances = false
# utterances_repo = "xxx/xxx"
```

### 字體

字體的 CSS 位於 float/sass/font.scss，欲更換字體，把 float/sass/font.scss 複製到自己的 sass/font.scss，並修改之。

## 已知問題

- 分頁設定皆須設為 10 篇分頁。因為 Zola 的 `get_section()` 無法取得該 section 的分頁設定。

        