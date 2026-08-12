# Giallo

A Rust syntax highlighting library using TextMate grammars and themes, producing the same output as VSCode.

This uses the curated grammars and themes from <https://github.com/shikijs/textmate-grammars-themes> for an optional built-in
starting kit and testing, but you can start from an empty canvas if you want.

For documentation on how TextMate grammars work, you can refer to <https://github.com/RedCMD/TmLanguage-Syntax-Highlighter/tree/main/documentation>.
## Installation

```toml
[dependencies]
giallo = { version = "0.2.0", features = ["dump"] }
```

The `dump` feature is required to use `Registry::builtin()` or create/load your own dump. The dump is not tracked
in git since it might change frequently, and is generated in the CI release script.

The dump is currently 1.14 MiB compressed bitcode file.

Giallo currently uses a fork of [rust-onig](https://github.com/rust-onig/rust-onig). Once <https://github.com/rust-onig/rust-onig/pull/210>
or something similar is released on crates.io, I will switch back to the rust-onig crate.

## Usage

```rust
use giallo::{HighlightOptions, HtmlRenderer, RenderOptions, Registry, ThemeVariant};

// Load the pre-built registry
let mut registry = Registry::builtin()?;
registry.link_grammars();

let code = "let x = 42;";
let options = HighlightOptions::new("javascript", ThemeVariant::Single("catppuccin-frappe"));
// For light/dark support, you can specify 2 themes
// let options = HighlightOptions::new("javascript", ThemeVariant::Dual {
//     light: "catppuccin-latte",
//     dark: "catppuccin-mocha",
// });
let highlighted = registry.highlight(code, options)?;

// Render to HTML
let html = HtmlRenderer::default().render(&highlighted, &RenderOptions::default());
println!("{html}");
```

See the `examples` directory for more examples.

## Renderers

Highlighting some code is done the same way regardless of where/how you're planning to display the output.
Giallo will give you back everything you need to implement your own renderer but also provides some (well one currently)
renderers.

### HTML renderer

This renderer outputs the text wrapped in a `<pre><code>...</code></pre>` with all the colours and attributes set correctly
as well as escaping the HTML content.

If you want to use line numbers, giallo will set some classes on `<span>` that you will need to target via CSS to have
something looking good. The minimal CSS is exported as `GIALLO_CSS` by the crate.

This renderer also supports light/dark mode automatically if you highlight the text using 2 themes by using the [light-dark](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/color_value/light-dark)
function in the `style` attribute.

You can also have Giallo generates the CSS file for any theme and let the HTML renderer use the classes from it. This
is useful for example if you want a light/dark theme switch where the above inline approach would not work.

## Built in

If you use the `dump` feature, giallo provides the following 220+ grammars and ~60 themes.
You can use [Shiki playground](https://textmate-grammars-themes.netlify.app/) to see the various themes and languages in action or open VSCode.

### Grammars

The list below is in the form: `{lang name} -> aliases`.

<!-- GRAMMARS_START -->
- abap
- actionscript-3 -> actionscript, as3
- ada
- ahk -> ahk1
- ahk2
- angular-html
- angular-ts
- apache
- apex
- apl
- applescript
- ara
- asciidoc -> adoc
- asm
- astro
- awk
- ballerina
- bat -> batch, cmd
- beancount
- berry -> be
- bibtex
- bicep
- bird2 -> bird
- blade
- bsl -> 1c
- c
- c3
- cadence -> cdc
- cairo
- chapel -> chpl
- clarity
- clojure -> clj
- cmake
- cobol
- codeowners
- codeql -> ql
- coffee -> coffeescript
- common-lisp -> lisp
- coq
- cpp -> c++
- crystal
- csharp -> c#, cs
- css
- csv
- cue
- cypher -> cql
- d
- dart
- dax
- desktop
- diff
- docker -> dockerfile
- dotenv
- dream-maker
- edge
- elixir
- elm
- emacs-lisp -> elisp
- erb
- erlang -> erl
- fennel
- fish
- fluent -> ftl
- fortran-fixed-form -> f, for, f77
- fortran-free-form -> f90, f95, f03, f08, f18
- fsharp -> f#, fs
- gdresource -> tscn, tres
- gdscript -> gd
- gdshader
- genie
- gherkin
- git-commit
- git-rebase
- gleam
- glimmer-js -> gjs
- glimmer-ts -> gts
- glsl
- gn
- gnuplot
- go
- graphql -> gql
- groovy
- hack
- haml
- handlebars -> hbs
- haskell -> hs
- haxe
- hcl
- hjson
- hlsl
- html
- html-derivative
- http
- hurl
- hxml
- hy
- imba
- ini -> properties
- java
- javascript -> js, cjs, mjs
- jinja
- jison
- json
- json5
- jsonc
- jsonl
- jsonnet
- jssm -> fsl
- jsx
- julia -> jl
- just -> justfile
- kdl
- kotlin -> kt, kts
- kusto -> kql
- latex
- lean -> lean4
- less
- liquid
- llvm
- log
- logo
- lua
- luau
- make -> makefile
- markdown -> md
- marko
- matlab
- mdc
- mdx
- mermaid -> mmd
- mipsasm -> mips
- mojo
- moonbit -> mbt, mbti
- move
- narrat -> nar
- nextflow -> nf
- nextflow-groovy
- nginx
- nim
- nix
- nsis
- nushell -> nu
- objective-c -> objc
- objective-cpp
- ocaml
- odin
- openscad -> scad
- org
- pascal
- perl
- php
- pkl
- plain -> txt, text
- plsql
- po -> pot, potx
- polar
- postcss
- powerquery
- powershell -> ps, ps1, pwsh
- prisma
- prolog
- proto -> protobuf
- pug -> jade
- puppet
- purescript
- python -> py
- qml
- qmldir
- qss
- r
- racket
- raku -> perl6
- razor
- rbs -> ruby-signature
- reg
- regexp -> regex
- rel
- riscv
- ron
- rosmsg
- rst
- ruby -> rb
- rust -> rs
- sas
- sass
- scala
- scheme
- scss
- sdbl -> 1c-query
- shaderlab -> shader
- shellscript -> bash, sh, shell, zsh
- shellsession -> console
- smalltalk
- smithy
- solidity
- soy -> closure-templates
- sparql
- splunk -> spl
- sql
- ssh-config
- stata
- stylus -> styl
- surrealql -> surql
- svelte
- swift
- system-verilog
- systemd
- talonscript -> talon
- tasl
- tcl
- templ
- terraform -> tf, tfvars
- tex
- toml
- ts-tags -> lit
- tsv
- tsx
- turtle
- twig
- typescript -> ts, cts, mts
- typespec -> tsp
- typst -> typ
- v
- vala
- vb
- verilog
- vhdl
- viml -> vim, vimscript
- vue
- vue-html
- vue-vine
- vyper -> vy
- wasm
- wenyan -> 文言
- wgsl
- wikitext -> mediawiki, wiki
- wit
- wolfram -> wl
- xml
- xsl
- yaml -> yml
- zenscript
- zig
<!-- GRAMMARS_END -->

### Themes

<!-- THEMES_START -->
- andromeeda
- aurora-x
- ayu-dark
- ayu-light
- ayu-mirage
- catppuccin-frappe
- catppuccin-latte
- catppuccin-macchiato
- catppuccin-mocha
- dark-plus
- dracula
- dracula-soft
- everforest-dark
- everforest-light
- github-dark
- github-dark-default
- github-dark-dimmed
- github-dark-high-contrast
- github-light
- github-light-default
- github-light-high-contrast
- gruvbox-dark-hard
- gruvbox-dark-medium
- gruvbox-dark-soft
- gruvbox-light-hard
- gruvbox-light-medium
- gruvbox-light-soft
- horizon
- horizon-bright
- houston
- kanagawa-dragon
- kanagawa-lotus
- kanagawa-wave
- laserwave
- light-plus
- material-theme
- material-theme-darker
- material-theme-lighter
- material-theme-ocean
- material-theme-palenight
- min-dark
- min-light
- monokai
- night-owl
- night-owl-light
- nord
- one-dark-pro
- one-light
- plastic
- poimandres
- red
- rose-pine
- rose-pine-dawn
- rose-pine-moon
- slack-dark
- slack-ochin
- snazzy-light
- solarized-dark
- solarized-light
- synthwave-84
- tokyo-night
- vesper
- vitesse-black
- vitesse-dark
- vitesse-light
<!-- THEMES_END -->


## Why not...

### syntect

syntect is using _old_ Sublime Text syntaxes, it doesn't support features that recent syntaxes use (see https://github.com/trishume/syntect/issues/271).
Projects like [bat](https://github.com/sharkdp/bat) keep their own curated set of grammars, sometimes applying patches to fix things.
The Rust syntax for example is about 6 years old and does not know about async/await.
VSCode is also a LOT more popular than Sublime these days.

Giallo has been developed to replace syntect usage in Zola.

### tree-sitter

This repository initially started as a tree-sitter highlighter but the grammars were at the time very big (eg easily
adding 100MB+ to a binary for ~50 languages, compared to ~1MiB for 4x more languages with Giallo) and queries were slow to load (see https://github.com/getzola/zola/issues/1787#issuecomment-1458569776)
Both are kind of dealbreakers for something meant to be added to Zola.
