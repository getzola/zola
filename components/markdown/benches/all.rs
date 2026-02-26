use ahash::AHashMap as HashMap;
use config::{Config, HighlightConfig, Highlighting};
use criterion::{Criterion, criterion_group, criterion_main};
use markdown::{MarkdownContext, render_content};
use std::hint::black_box;
use templates::ZOLA_TERA;
use utils::types::InsertAnchor;

const CONTENT: &str = r#"
# Heading 1

This is a paragraph with **bold** and *italic* text.

## Heading 2

Here's a list:

- Item 1
- Item 2
- Item 3

And a numbered list:

1. First item
2. Second item
3. Third item

> This is a blockquote
> with multiple lines

Here's a [link to Zola](https://www.getzola.org/).

### Code Examples

Some inline `code` here.

```rust
fn main() {
    let message = "Hello, world!";
    println!("{}", message);

    for i in 0..10 {
        println!("Count: {}", i);
    }
}
```

```python
def greet(name):
    """Greet someone by name."""
    return f"Hello, {name}!"

if __name__ == "__main__":
    print(greet("World"))
    numbers = [1, 2, 3, 4, 5]
    squared = [n ** 2 for n in numbers]
```

## Another Section

Some more text with a table:

| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| A        | B        | C        |
| D        | E        | F        |

The end.
"#;

fn bench_without_highlighting(c: &mut Criterion) {
    let mut tera = ZOLA_TERA.clone();
    tera.set_fallback_prefixes(vec!["__zola_builtins/".to_string()]);
    let permalinks = HashMap::new();
    let wikilinks = HashMap::new();
    let config = Config::default_for_test();

    let context = MarkdownContext {
        tera: &tera,
        config: &config,
        permalinks: &permalinks,
        wikilinks: &wikilinks,
        lang: &config.default_language,
        current_permalink: "https://www.example.com/bench/",
        current_path: "bench.md",
        insert_anchor: InsertAnchor::None,
    };

    c.bench_function("render_without_highlighting", |b| {
        b.iter(|| render_content(black_box(CONTENT), &context).unwrap())
    });
}

fn bench_with_highlighting(c: &mut Criterion) {
    let mut tera = ZOLA_TERA.clone();
    tera.set_fallback_prefixes(vec!["__zola_builtins/".to_string()]);
    let permalinks = HashMap::new();
    let wikilinks = HashMap::new();
    let mut config = Config::default_for_test();

    let mut highlighting = Highlighting {
        error_on_missing_language: false,
        style: Default::default(),
        theme: HighlightConfig::Single { theme: "github-dark".to_string() },
        extra_grammars: vec![],
        extra_themes: vec![],
        registry: Default::default(),
        data_attr_position: Default::default(),
    };
    highlighting.init(std::path::Path::new(".")).unwrap();
    config.markdown.highlighting = Some(highlighting);

    let context = MarkdownContext {
        tera: &tera,
        config: &config,
        permalinks: &permalinks,
        wikilinks: &wikilinks,
        lang: &config.default_language,
        current_permalink: "https://www.example.com/bench/",
        current_path: "bench.md",
        insert_anchor: InsertAnchor::None,
    };

    c.bench_function("render_with_highlighting", |b| {
        b.iter(|| render_content(black_box(CONTENT), &context).unwrap())
    });
}

criterion_group!(benches, bench_without_highlighting, bench_with_highlighting);
criterion_main!(benches);
