+++
title = "Syntax Highlighting"
weight = 80
+++

Zola comes with built-in syntax highlighting but you first
need to enable it in the [configuration](@/documentation/getting-started/configuration.md).

Once this is done, Zola will automatically highlight all code blocks
in your content. A code block in Markdown looks like the following:

````md

```rust
let highlight = true;
```

````

You can replace the `rust` by the language you want to highlight or not put anything to get it
interpreted as plain text.

Here is a full list of the supported languages and the short names you can use:

```
- ActionScript -> ["as"]
- AppleScript -> ["applescript", "script editor"]
- ASP -> ["asa"]
- Assembly x86 (NASM) -> ["asm", "inc", "nasm"]
- Batch File -> ["bat", "cmd"]
- BibTeX -> ["bib"]
- Bourne Again Shell (bash) -> ["sh", "bash", "zsh", "fish", ".bash_aliases", ".bash_completions", ".bash_functions", ".bash_login", ".bash_logout", ".bash_profile", ".bash_variables", ".bashrc", ".profile", ".textmate_init", ".zshrc"]
- C -> ["c", "h"]
- C# -> ["cs", "csx"]
- C++ -> ["cpp", "cc", "cp", "cxx", "c++", "C", "h", "hh", "hpp", "hxx", "h++", "inl", "ipp"]
- Clojure -> ["clj"]
- CMake -> ["CMakeLists.txt", "cmake"]
- CMake C Header -> ["h.in"]
- CMake C++ Header -> ["hh.in", "hpp.in", "hxx.in", "h++.in"]
- CMakeCache -> ["CMakeCache.txt"]
- Crystal -> ["cr"]
- CSS -> ["css", "css.erb", "css.liquid"]
- D -> ["d", "di"]
- Dart -> ["dart"]
- Diff -> ["diff", "patch"]
- Elixir -> ["ex", "exs"]
- Elm -> ["elm"]
- Erlang -> ["erl", "hrl", "Emakefile", "emakefile"]
- fsharp -> ["fs"]
- Generic Config -> ["cfg", "conf", "config", "ini", "pro", "mak", "mk", "Doxyfile", "inputrc", ".inputrc", "dircolors", ".dircolors", "gitmodules", ".gitmodules", "gitignore", ".gitignore", "gitattributes", ".gitattributes"]
- Git Attributes -> ["attributes", "gitattributes", ".gitattributes"]
- Git Commit -> ["COMMIT_EDITMSG", "MERGE_MSG", "TAG_EDITMSG"]
- Git Config -> ["gitconfig", ".gitconfig", ".gitmodules"]
- Git Ignore -> ["exclude", "gitignore", ".gitignore"]
- Git Link -> [".git"]
- Git Log -> ["gitlog"]
- Git Rebase Todo -> ["git-rebase-todo"]
- Go -> ["go"]
- Graphviz (DOT) -> ["dot", "DOT", "gv"]
- Groovy -> ["groovy", "gvy", "gradle", "Jenkinsfile"]
- Handlebars -> ["handlebars", "handlebars.html", "hbr", "hbrs", "hbs", "hdbs", "hjs", "mu", "mustache", "rac", "stache", "template", "tmpl"]
- Haskell -> ["hs"]
- HTML -> ["html", "htm", "shtml", "xhtml"]
- HTML (ASP) -> ["asp"]
- HTML (Erlang) -> ["yaws"]
- HTML (Rails) -> ["rails", "rhtml", "erb", "html.erb"]
- HTML (Tcl) -> ["adp"]
- Java -> ["java", "bsh"]
- Java Properties -> ["properties"]
- Java Server Page (JSP) -> ["jsp"]
- JavaScript -> ["js", "htc"]
- JavaScript (Rails) -> ["js.erb"]
- Jinja2 -> ["j2", "jinja2"]
- JSON -> ["json", "sublime-settings", "sublime-menu", "sublime-keymap", "sublime-mousemap", "sublime-theme", "sublime-build", "sublime-project", "sublime-completions", "sublime-commands", "sublime-macro", "sublime-color-scheme"]
- Julia -> ["jl"]
- Kotlin -> ["kt", "kts"]
- LaTeX -> ["tex", "ltx"]
- Less -> ["less", "css.less"]
- Linker Script -> ["ld"]
- Lisp -> ["lisp", "cl", "clisp", "l", "mud", "el", "scm", "ss", "lsp", "fasl"]
- Literate Haskell -> ["lhs"]
- Lua -> ["lua"]
- Makefile -> ["make", "GNUmakefile", "makefile", "Makefile", "makefile.am", "Makefile.am", "makefile.in", "Makefile.in", "OCamlMakefile", "mak", "mk"]
- Markdown -> ["md", "mdown", "markdown", "markdn"]
- MATLAB -> ["matlab"]
- MiniZinc (MZN) -> ["mzn", "dzn"]
- NAnt Build File -> ["build"]
- Nim -> ["nim", "nims"]
- Nix -> ["nix"]
- Objective-C -> ["m", "h"]
- Objective-C++ -> ["mm", "M", "h"]
- OCaml -> ["ml", "mli"]
- OCamllex -> ["mll"]
- OCamlyacc -> ["mly"]
- Pascal -> ["pas", "p", "dpr"]
- Perl -> ["pl", "pm", "pod", "t", "PL"]
- PHP -> ["php", "php3", "php4", "php5", "php7", "phps", "phpt", "phtml"]
- Plain Text -> ["txt"]
- PowerShell -> ["ps1", "psm1", "psd1"]
- Python -> ["py", "py3", "pyw", "pyi", "pyx", "pyx.in", "pxd", "pxd.in", "pxi", "pxi.in", "rpy", "cpy", "SConstruct", "Sconstruct", "sconstruct", "SConscript", "gyp", "gypi", "Snakefile", "wscript"]
- R -> ["R", "r", "s", "S", "Rprofile"]
- Rd (R Documentation) -> ["rd"]
- Reason -> ["re", "rei"]
- Regular Expression -> ["re"]
- reStructuredText -> ["rst", "rest"]
- Ruby -> ["rb", "Appfile", "Appraisals", "Berksfile", "Brewfile", "capfile", "cgi", "Cheffile", "config.ru", "Deliverfile", "Fastfile", "fcgi", "Gemfile", "gemspec", "Guardfile", "irbrc", "jbuilder", "podspec", "prawn", "rabl", "rake", "Rakefile", "Rantfile", "rbx", "rjs", "ruby.rail", "Scanfile", "simplecov", "Snapfile", "thor", "Thorfile", "Vagrantfile"]
- Ruby Haml -> ["haml", "sass"]
- Ruby on Rails -> ["rxml", "builder"]
- Rust -> ["rs"]
- Scala -> ["scala", "sbt"]
- SQL -> ["sql", "ddl", "dml"]
- SQL (Rails) -> ["erbsql", "sql.erb"]
- SWI-Prolog -> ["pro"]
- Swift -> ["swift"]
- Tcl -> ["tcl"]
- TeX -> ["sty", "cls"]
- Textile -> ["textile"]
- TOML -> ["toml", "tml", "Cargo.lock", "Gopkg.lock", "Pipfile"]
- TypeScript -> ["ts"]
- TypeScriptReact -> ["tsx"]
- VimL -> ["vim"]
- XML -> ["xml", "xsd", "xslt", "tld", "dtml", "rss", "opml", "svg"]
- YAML -> ["yaml", "yml", "sublime-syntax"]
```

If you want to highlight a language not on that list, please open an issue or a pull request on the [Zola repo](https://github.com/getzola/zola).
Alternatively, the `extra_syntaxes` config option can be used to add additional syntax files.

If your site source is laid out as follows:

```
.
├── config.toml
├── content/
│   └── ...
├── static/
│   └── ...
├── syntaxes/
│   ├── Sublime-Language1/
│   │   └── lang1.sublime-syntax
│   └── lang2.sublime-syntax
└── templates/
    └── ...
```

you would set your `extra_syntaxes` to `["syntaxes", "syntaxes/Sublime-Language1"]` in order to load `lang1.sublime-syntax` and `lang2.sublime-syntax`.
