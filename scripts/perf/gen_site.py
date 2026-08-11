#!/usr/bin/env python3
"""Deterministic synthetic Zola site generator for performance benchmarking.

Every site is a pure function of (scenario, pages, seed): running the generator
twice with the same arguments produces a byte-identical tree. We deliberately do
not use Python's `random` module, whose stream is not guaranteed stable across
interpreter versions; instead we use an explicit xorshift64* PRNG.

Usage:
    ./gen_site.py --scenario mixed-realistic --pages 1000 --out /tmp/site
    ./gen_site.py --list-scenarios

Knobs (all overridable on the command line, defaults come from the scenario):
    --links-per-page        internal @/ links emitted per page
    --sections              number of leaf sections
    --depth                 section tree depth
    --taxonomies-per-page   how many taxonomy entries each page declares
    --taxonomy-count        how many taxonomies are configured
    --taxonomy-cardinality  distinct terms per taxonomy
    --markdown-paragraphs   paragraphs of body text per page
    --templating-frequency  fraction of pages containing Tera syntax in content
                            (the 0.23 replacement for shortcodes)
    --alias-frequency       fraction of pages declaring an alias
    --paginate-by           section pagination size (0 = disabled)
    --languages             number of languages (1 = monolingual)
    --search                build a search index
    --assets-frequency      fraction of pages that are colocated with an asset
    --template-complexity   simple | rich  (how much work page.html does)
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
from dataclasses import dataclass, asdict, field
from pathlib import Path

MASK64 = (1 << 64) - 1


class Rng:
    """xorshift64* — small, fast, and stable across Python versions."""

    __slots__ = ("state",)

    def __init__(self, seed: int) -> None:
        # 0 is a fixed point of xorshift; force a non-zero state.
        self.state = (seed * 2685821657736338717 + 1442695040888963407) & MASK64 or 88172645463325252

    def next_u64(self) -> int:
        x = self.state
        x ^= (x >> 12) & MASK64
        x ^= (x << 25) & MASK64
        x ^= (x >> 27) & MASK64
        self.state = x & MASK64
        return (self.state * 2685821657736338717) & MASK64

    def below(self, n: int) -> int:
        """Uniform-ish integer in [0, n). Modulo bias is irrelevant here."""
        return self.next_u64() % n if n > 0 else 0

    def chance(self, probability: float) -> bool:
        return (self.next_u64() % 10_000) < int(probability * 10_000)

    def pick(self, seq):
        return seq[self.below(len(seq))]


WORDS = [
    "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
    "iota", "kappa", "lambda", "mu", "nu", "xi", "omicron", "pi", "rho",
    "sigma", "tau", "upsilon", "phi", "chi", "psi", "omega", "vector",
    "matrix", "tensor", "graph", "index", "cache", "buffer", "stream",
    "kernel", "lattice", "cluster", "gradient", "entropy", "manifold",
]

TOPICS = [
    "architecture", "deployment", "observability", "storage", "scheduling",
    "networking", "security", "tooling", "migration", "benchmarking",
]


@dataclass
class Scenario:
    name: str
    description: str
    links_per_page: int = 2
    sections: int = 10
    depth: int = 2
    taxonomies_per_page: int = 0
    taxonomy_count: int = 0
    taxonomy_cardinality: int = 0
    markdown_paragraphs: int = 6
    templating_frequency: float = 0.0
    alias_frequency: float = 0.0
    paginate_by: int = 0
    languages: int = 1
    search: bool = False
    assets_frequency: float = 0.0
    template_complexity: str = "simple"
    minify: bool = False
    feeds: bool = False
    # Size in KB of a per-page JSON "view model" loaded via load_data() from the
    # page template. 0 disables the whole mechanism. This mirrors data-driven
    # sites that keep their payload outside markdown.
    view_model_kb: int = 0


SCENARIOS = {s.name: s for s in [
    Scenario(
        name="simple-pages",
        description="Flat-ish sections, tiny pages, no links, no taxonomies. Measures the floor cost per page.",
        links_per_page=0, sections=10, depth=1, markdown_paragraphs=3,
    ),
    Scenario(
        name="dense-internal-links",
        description="Every page links to 40 other pages via @/ links. Stresses link resolution, backlinks and anchor checking.",
        links_per_page=40, sections=20, depth=2, markdown_paragraphs=4,
    ),
    Scenario(
        name="many-taxonomies",
        description="4 taxonomies, 6 terms per page, high cardinality. Stresses taxonomy construction and term page lists.",
        links_per_page=1, sections=20, depth=2, taxonomy_count=4,
        taxonomies_per_page=6, taxonomy_cardinality=60, markdown_paragraphs=4,
    ),
    Scenario(
        name="deep-sections",
        description="Deep section tree with few pages per section (mirrors doc/dossier sites). Stresses ancestors, subsections and per-section sorting.",
        links_per_page=1, sections=400, depth=5, markdown_paragraphs=3,
    ),
    Scenario(
        name="template-heavy",
        description="Rich templates that walk section pages, ancestors and site data on every render.",
        links_per_page=1, sections=20, depth=2, markdown_paragraphs=4,
        template_complexity="rich",
    ),
    Scenario(
        name="markdown-heavy",
        description="Large markdown bodies with code blocks, tables and many headings.",
        links_per_page=2, sections=15, depth=2, markdown_paragraphs=60,
    ),
    Scenario(
        name="data-heavy",
        description="Each page loads its own JSON view model through load_data(). Mirrors data-driven sites where content is a thin shell over generated data.",
        links_per_page=2, sections=200, depth=4, markdown_paragraphs=4,
        template_complexity="rich", view_model_kb=12, minify=True,
    ),
    Scenario(
        name="mixed-realistic",
        description="Approximates the reference site: deep sections, moderate links, content templating, aliases, minify, some assets.",
        links_per_page=6, sections=300, depth=4, taxonomy_count=2,
        taxonomies_per_page=3, taxonomy_cardinality=40, markdown_paragraphs=8,
        templating_frequency=0.25, alias_frequency=0.1, assets_frequency=0.05,
        template_complexity="rich", minify=True,
    ),
]}


# --------------------------------------------------------------------------
# templates
# --------------------------------------------------------------------------

BASE_SIMPLE = """<!DOCTYPE html>
<html lang="{{ lang }}">
<head>
<meta charset="utf-8">
<title>{% block title %}{{ config.title }}{% endblock %}</title>
<link rel="canonical" href="{{ current_url }}">
</head>
<body>
<header><a href="{{ get_url(path='/') }}">{{ config.title }}</a></header>
<main>{% block content %}{% endblock %}</main>
</body>
</html>
"""

BASE_RICH = """<!DOCTYPE html>
<html lang="{{ lang }}">
<head>
<meta charset="utf-8">
<title>{% block title %}{{ config.title }}{% endblock %}</title>
<meta name="description" content="{% block description %}{{ config.description }}{% endblock %}">
<link rel="canonical" href="{{ current_url }}">
{% block extra_head %}{% endblock %}
</head>
<body>
<header>
  <a href="{{ get_url(path='/') }}">{{ config.title }}</a>
  <nav>
  {% for entry in nav_data.items %}
    <a href="{{ get_url(path=entry.path) }}">{{ entry.label }}</a>
  {% endfor %}
  </nav>
</header>
<main>{% block content %}{% endblock %}</main>
<footer>
  <p>{{ config.title }} &middot; {{ now() | date(format='%Y') }}</p>
</footer>
</body>
</html>
"""

# `nav_data` is loaded once in a macro-free way so every page pays the load_data
# lookup, mirroring real sites that call load_data from base.html.
RICH_PRELUDE = """{% set nav_data = load_data(path='data/nav.toml') %}
"""

PAGE_SIMPLE = """{% extends 'base.html' %}
{% block title %}{{ page.title }}{% endblock %}
{% block content %}
<article>
<h1>{{ page.title }}</h1>
{{ page.content | safe }}
</article>
{% endblock %}
"""

PAGE_RICH = """{% extends 'base.html' %}
{% block title %}{{ page.title }} — {{ config.title }}{% endblock %}
{% block description %}{{ page.description | default(value=config.description) }}{% endblock %}
{% block content %}
<article>
<nav class="breadcrumbs">
{% for ancestor in page.ancestors %}
  {% set crumb = get_section(path=ancestor) %}
  <a href="{{ crumb.permalink }}">{{ crumb.title }}</a>
{% endfor %}
</nav>
<h1>{{ page.title }}</h1>
<p class="meta">{{ page.reading_time }} min &middot; {{ page.word_count }} words</p>
{% if page.taxonomies %}
<ul class="taxonomies">
{% for name, terms in page.taxonomies %}
  {% for term in terms %}<li>{{ name }}: {{ term }}</li>{% endfor %}
{% endfor %}
</ul>
{% endif %}
<div class="toc">
{% for h in page.toc %}<a href="{{ h.permalink }}">{{ h.title }}</a>{% endfor %}
</div>
{{ page.content | safe }}
{% if page.backlinks %}
<aside><h2>Mentioned by</h2>
<ul>{% for b in page.backlinks %}<li><a href="{{ b.permalink }}">{{ b.title }}</a></li>{% endfor %}</ul>
</aside>
{% endif %}
<nav class="siblings">
{% if page.lower %}<a href="{{ page.lower.permalink }}">{{ page.lower.title }}</a>{% endif %}
{% if page.higher %}<a href="{{ page.higher.permalink }}">{{ page.higher.title }}</a>{% endif %}
</nav>
</article>
{% endblock %}
"""

SECTION_SIMPLE = """{% extends 'base.html' %}
{% block title %}{{ section.title }}{% endblock %}
{% block content %}
<h1>{{ section.title }}</h1>
{{ section.content | safe }}
<ul>
{% for page in section.pages %}<li><a href="{{ page.permalink }}">{{ page.title }}</a></li>{% endfor %}
</ul>
<ul>
{% for sub in section.subsections %}
  {% set s = get_section(path=sub) %}
  <li><a href="{{ s.permalink }}">{{ s.title }}</a></li>
{% endfor %}
</ul>
{% endblock %}
"""

SECTION_RICH = """{% extends 'base.html' %}
{% block title %}{{ section.title }} — {{ config.title }}{% endblock %}
{% block content %}
<h1>{{ section.title }}</h1>
{{ section.content | safe }}
<ul class="pages">
{% for page in section.pages %}
  <li>
    <a href="{{ page.permalink }}">{{ page.title }}</a>
    <span>{{ page.word_count }} words</span>
    {% if page.description %}<p>{{ page.description }}</p>{% endif %}
  </li>
{% endfor %}
</ul>
<ul class="subsections">
{% for sub in section.subsections %}
  {% set s = get_section(path=sub) %}
  <li><a href="{{ s.permalink }}">{{ s.title }}</a> ({{ s.pages | length }})</li>
{% endfor %}
</ul>
{% endblock %}
"""

VIEW_MODEL_BLOCK = """{% if page.extra.view_model %}
{% set vm = load_data(path=page.extra.view_model) %}
<section class="view-model">
<h2>{{ vm.title }}</h2>
<ul>{% for row in vm.rows %}<li>{{ row.id }}: {{ row.label }}</li>{% endfor %}</ul>
</section>
{% endif %}
"""

INDEX_TPL = """{% extends 'base.html' %}
{% block content %}
<h1>{{ config.title }}</h1>
{{ section.content | safe }}
<ul>
{% for sub in section.subsections %}
  {% set s = get_section(path=sub) %}
  <li><a href="{{ s.permalink }}">{{ s.title }}</a></li>
{% endfor %}
</ul>
{% endblock %}
"""

PAGINATED_SECTION = """{% extends 'base.html' %}
{% block content %}
<h1>{{ section.title }}</h1>
<ul>
{% for page in paginator.pages %}<li><a href="{{ page.permalink }}">{{ page.title }}</a></li>{% endfor %}
</ul>
{% if paginator.previous %}<a href="{{ paginator.previous }}">prev</a>{% endif %}
{% if paginator.next %}<a href="{{ paginator.next }}">next</a>{% endif %}
{% endblock %}
"""

TAXONOMY_LIST = """{% extends 'base.html' %}
{% block content %}
<h1>{{ taxonomy.name }}</h1>
<ul>
{% for term in terms %}<li><a href="{{ term.permalink }}">{{ term.name }}</a> ({{ term.page_count }})</li>{% endfor %}
</ul>
{% endblock %}
"""

TAXONOMY_SINGLE = """{% extends 'base.html' %}
{% block content %}
<h1>{{ term.name }}</h1>
<ul>
{% for page in term.pages %}<li><a href="{{ page.permalink }}">{{ page.title }}</a></li>{% endfor %}
</ul>
{% endblock %}
"""


# --------------------------------------------------------------------------
# generation
# --------------------------------------------------------------------------


@dataclass
class Manifest:
    scenario: str
    seed: int
    requested_pages: int
    pages: int = 0
    sections: int = 0
    taxonomies: int = 0
    taxonomy_terms: int = 0
    internal_links: int = 0
    aliases: int = 0
    assets: int = 0
    templated_pages: int = 0
    languages: int = 1
    input_bytes: int = 0
    md_files: int = 0
    knobs: dict = field(default_factory=dict)


def title_for(rng: Rng, n: int) -> str:
    return f"{rng.pick(WORDS).capitalize()} {rng.pick(TOPICS)} {n}"


def build_section_tree(cfg: Scenario) -> list[tuple[str, ...]]:
    """Return the list of section paths (as component tuples), excluding the root.

    The tree is deterministic: sections are distributed breadth-first so that a
    depth of D produces roughly equal numbers of nodes at each level.
    """
    if cfg.sections <= 0:
        return []
    depth = max(1, cfg.depth)
    paths: list[tuple[str, ...]] = []
    # level 1 gets ceil(sections / (2^0 + ... )) — simple deterministic split:
    per_level = max(1, cfg.sections // depth)
    level_nodes: list[tuple[str, ...]] = [()]
    counter = 0
    for level in range(depth):
        next_level: list[tuple[str, ...]] = []
        wanted = per_level if level < depth - 1 else cfg.sections - len(paths)
        if wanted <= 0:
            break
        for i in range(wanted):
            parent = level_nodes[i % len(level_nodes)]
            name = f"s{level}-{i}"
            node = parent + (name,)
            paths.append(node)
            next_level.append(node)
        level_nodes = next_level or level_nodes
    return paths[: cfg.sections]


def markdown_body(rng: Rng, cfg: Scenario, page_index: int, link_targets: list[str]) -> tuple[str, int]:
    """Return (markdown, number_of_internal_links)."""
    parts: list[str] = []
    n_links = 0
    # The first heading is fixed so that cross-page anchor links (`@/x.md#...`)
    # always resolve; broken anchors would otherwise change what the internal
    # link checker does and make the benchmark measure warning formatting.
    parts.append("## Heading 0 for architecture\n")
    for p in range(1, cfg.markdown_paragraphs + 1):
        if p % 3 == 0:
            parts.append(f"## Heading {p} for {rng.pick(TOPICS)}\n")
        sentence = " ".join(rng.pick(WORDS) for _ in range(18))
        parts.append(f"{sentence.capitalize()}.\n")
        if p % 5 == 4:
            parts.append("```rust\nfn demo() -> usize {\n    (0..10).map(|i| i * 2).sum()\n}\n```\n")
        if p % 7 == 6:
            parts.append("| col a | col b |\n| ----- | ----- |\n| 1 | 2 |\n| 3 | 4 |\n")

    if cfg.links_per_page and link_targets:
        links = []
        for _ in range(cfg.links_per_page):
            target = rng.pick(link_targets)
            # half of the links carry an anchor, exercising the anchor checker
            if rng.chance(0.5):
                links.append(f"- [ref](@/{target}#heading-0-for-architecture)")
            else:
                links.append(f"- [ref](@/{target})")
            n_links += 1
        parts.append("\n".join(links) + "\n")

    if cfg.templating_frequency and rng.chance(cfg.templating_frequency):
        parts.append(
            "\n{% for i in range(end=3) %}\n"
            "- generated row {{ i }} on {{ page.title }}\n"
            "{% endfor %}\n"
        )

    return "\n".join(parts), n_links


def front_matter(rng: Rng, cfg: Scenario, title: str, index: int, taxonomy_terms: dict) -> tuple[str, int]:
    lines = ["+++", f'title = "{title}"', f'date = 2024-{(index % 12) + 1:02d}-{(index % 27) + 1:02d}']
    lines.append(f'description = "Synthetic page {index} used for benchmarking."')
    aliases = 0
    if cfg.alias_frequency and rng.chance(cfg.alias_frequency):
        lines.append(f'aliases = ["/legacy/p{index}/"]')
        aliases = 1
    if taxonomy_terms:
        lines.append("[taxonomies]")
        for name, terms in taxonomy_terms.items():
            rendered = ", ".join(f'"{t}"' for t in terms)
            lines.append(f"{name} = [{rendered}]")
    if cfg.view_model_kb:
        lines.append("[extra]")
        lines.append(f'view_model = "data/generated/vm-{index}.json"')
    lines.append("+++")
    lines.append("")
    return "\n".join(lines), aliases


def generate(cfg: Scenario, pages: int, seed: int, out: Path) -> Manifest:
    rng = Rng(seed)
    manifest = Manifest(scenario=cfg.name, seed=seed, requested_pages=pages,
                        languages=cfg.languages, knobs=asdict(cfg))

    if out.exists():
        shutil.rmtree(out)
    (out / "content").mkdir(parents=True)
    (out / "templates").mkdir(parents=True)
    (out / "static").mkdir(parents=True)
    (out / "data").mkdir(parents=True)

    section_paths = build_section_tree(cfg)
    # Ensure every intermediate directory is a real section too.
    all_sections: set[tuple[str, ...]] = set()
    for path in section_paths:
        for i in range(1, len(path) + 1):
            all_sections.add(path[:i])
    ordered_sections = sorted(all_sections)

    taxonomy_names = [f"tax{i}" for i in range(cfg.taxonomy_count)]
    taxonomy_terms_pool = {
        name: [f"{name}-term-{i}" for i in range(max(1, cfg.taxonomy_cardinality))]
        for name in taxonomy_names
    }

    # --- page path plan (needed up front so links can target real pages) ----
    # Colocation is decided here, before any content is generated, so that the
    # link targets below always point at the file that will actually exist.
    # `plan_rng` is a separate stream: page bodies must not shift when the
    # colocation decisions change.
    plan_rng = Rng(seed ^ 0x9E3779B97F4A7C15)
    page_slots: list[tuple[tuple[str, ...], int, bool]] = []
    for i in range(pages):
        section = ordered_sections[i % len(ordered_sections)] if ordered_sections else ()
        colocated = bool(cfg.assets_frequency) and plan_rng.chance(cfg.assets_frequency)
        page_slots.append((section, i, colocated))

    link_targets = [
        "/".join(section + ((f"p{i}/index.md",) if colocated else (f"p{i}.md",)))
        if section else (f"p{i}/index.md" if colocated else f"p{i}.md")
        for section, i, colocated in page_slots
    ]

    # --- config -------------------------------------------------------------
    config_lines = [
        'base_url = "https://perf.example.com"',
        'title = "Perf benchmark site"',
        'description = "Synthetic site generated by scripts/perf/gen_site.py"',
        "compile_sass = false",
        f"build_search_index = {'true' if cfg.search else 'false'}",
        f"generate_feeds = {'true' if cfg.feeds else 'false'}",
        f"minify_html = {'true' if cfg.minify else 'false'}",
        "generate_sitemap = true",
        "generate_robots_txt = true",
    ]
    if taxonomy_names:
        rendered = ", ".join(
            "{ name = \"%s\", paginate_by = 0, feed = false }" % n if False else '{ name = "%s" }' % n
            for n in taxonomy_names
        )
        config_lines.append(f"taxonomies = [{rendered}]")
    if cfg.languages > 1:
        for li in range(1, cfg.languages):
            code = f"l{li}"
            config_lines.append(f"[languages.{code}]")
            config_lines.append(f'title = "Perf benchmark site {code}"')
            config_lines.append(f"build_search_index = {'true' if cfg.search else 'false'}")
    config_lines.append("[markdown.highlighting]")
    config_lines.append('theme = "github-dark"')
    (out / "config.toml").write_text("\n".join(config_lines) + "\n", encoding="utf-8")

    # --- templates ----------------------------------------------------------
    rich = cfg.template_complexity == "rich"
    base = (RICH_PRELUDE + BASE_RICH) if rich else BASE_SIMPLE
    (out / "templates" / "base.html").write_text(base, encoding="utf-8")
    (out / "templates" / "index.html").write_text(INDEX_TPL, encoding="utf-8")
    page_tpl = PAGE_RICH if rich else PAGE_SIMPLE
    if cfg.view_model_kb:
        page_tpl = page_tpl.replace("</article>\n{% endblock %}",
                                    VIEW_MODEL_BLOCK + "</article>\n{% endblock %}")
    (out / "templates" / "page.html").write_text(page_tpl, encoding="utf-8")
    section_tpl = PAGINATED_SECTION if cfg.paginate_by else (SECTION_RICH if rich else SECTION_SIMPLE)
    (out / "templates" / "section.html").write_text(section_tpl, encoding="utf-8")
    if taxonomy_names:
        (out / "templates" / "taxonomy_list.html").write_text(TAXONOMY_LIST, encoding="utf-8")
        (out / "templates" / "taxonomy_single.html").write_text(TAXONOMY_SINGLE, encoding="utf-8")

    nav_entries = "\n".join(
        '[[items]]\nlabel = "%s"\npath = "/%s/"' % (s[-1], "/".join(s))
        for s in ordered_sections[:12]
    )
    (out / "data" / "nav.toml").write_text(nav_entries + "\n", encoding="utf-8")

    # --- root section -------------------------------------------------------
    root_index = (
        "+++\n"
        'title = "Perf benchmark site"\n'
        "sort_by = \"date\"\n"
        + (f"paginate_by = {cfg.paginate_by}\n" if cfg.paginate_by else "")
        + "+++\n\nRoot section of the synthetic benchmark site.\n"
    )
    (out / "content" / "_index.md").write_text(root_index, encoding="utf-8")
    manifest.sections += 1

    # --- sections -----------------------------------------------------------
    for components in ordered_sections:
        directory = out / "content" / Path(*components)
        directory.mkdir(parents=True, exist_ok=True)
        body = [
            "+++",
            f'title = "Section {"/".join(components)}"',
            'sort_by = "date"',
        ]
        if cfg.paginate_by:
            body.append(f"paginate_by = {cfg.paginate_by}")
        body += ["+++", "", f"Section {'/'.join(components)} of the synthetic site.", ""]
        (directory / "_index.md").write_text("\n".join(body), encoding="utf-8")
        manifest.sections += 1

    # --- pages --------------------------------------------------------------
    for section, i, colocated in page_slots:
        directory = out / "content" / (Path(*section) if section else Path("."))
        title = title_for(rng, i)

        terms: dict[str, list[str]] = {}
        if taxonomy_names and cfg.taxonomies_per_page:
            # `taxonomies_per_page` is the total number of (taxonomy, term)
            # entries on the page, spread round-robin over the taxonomies, so
            # the knob keeps meaning something when it is smaller or larger than
            # the taxonomy count.
            wanted: dict[str, int] = {name: 0 for name in taxonomy_names}
            for slot in range(cfg.taxonomies_per_page):
                wanted[taxonomy_names[slot % len(taxonomy_names)]] += 1
            for name, count in wanted.items():
                if not count:
                    continue
                pool = taxonomy_terms_pool[name]
                chosen = sorted({rng.pick(pool) for _ in range(count)})
                terms[name] = chosen

        fm, aliases = front_matter(rng, cfg, title, i, terms)
        body, n_links = markdown_body(rng, cfg, i, link_targets)
        has_templating = "{%" in body or "{{" in body

        if colocated:
            page_dir = directory / f"p{i}"
            page_dir.mkdir(parents=True, exist_ok=True)
            (page_dir / "index.md").write_text(fm + body, encoding="utf-8")
            (page_dir / "asset.txt").write_text(f"asset for page {i}\n", encoding="utf-8")
            manifest.assets += 1
        else:
            (directory / f"p{i}.md").write_text(fm + body, encoding="utf-8")

        if cfg.view_model_kb:
            rows = []
            # ~48 bytes per row, so kb * 1024 / 48 rows lands near the target size
            for r in range(max(1, (cfg.view_model_kb * 1024) // 48)):
                rows.append({"id": f"r{r}", "label": f"{rng.pick(WORDS)}-{rng.pick(TOPICS)}"})
            vm = {"title": title, "page": i, "rows": rows}
            vm_dir = out / "data" / "generated"
            vm_dir.mkdir(parents=True, exist_ok=True)
            (vm_dir / f"vm-{i}.json").write_text(
                json.dumps(vm, indent=None, sort_keys=True), encoding="utf-8")

        manifest.pages += 1
        manifest.internal_links += n_links
        manifest.aliases += aliases
        if has_templating:
            manifest.templated_pages += 1

    (out / "static" / "style.css").write_text("body{font-family:sans-serif}\n", encoding="utf-8")

    manifest.taxonomies = len(taxonomy_names)
    manifest.taxonomy_terms = sum(len(v) for v in taxonomy_terms_pool.values())
    total = 0
    md = 0
    for root, _dirs, files in os.walk(out):
        for f in files:
            p = Path(root) / f
            total += p.stat().st_size
            if f.endswith(".md"):
                md += 1
    manifest.input_bytes = total
    manifest.md_files = md

    (out / "manifest.json").write_text(json.dumps(asdict(manifest), indent=2, sort_keys=True) + "\n",
                                       encoding="utf-8")
    return manifest


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--scenario", default="mixed-realistic", choices=sorted(SCENARIOS))
    parser.add_argument("--pages", type=int, default=1000)
    parser.add_argument("--seed", type=int, default=20260811)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--list-scenarios", action="store_true")
    for knob, kind in [
        ("links-per-page", int), ("sections", int), ("depth", int),
        ("taxonomies-per-page", int), ("taxonomy-count", int),
        ("taxonomy-cardinality", int), ("markdown-paragraphs", int),
        ("templating-frequency", float), ("alias-frequency", float),
        ("paginate-by", int), ("languages", int), ("assets-frequency", float),
    ]:
        parser.add_argument(f"--{knob}", type=kind, default=None)
    parser.add_argument("--template-complexity", choices=["simple", "rich"], default=None)
    parser.add_argument("--search", action="store_true", default=None)
    parser.add_argument("--minify", action="store_true", default=None)
    parser.add_argument("--feeds", action="store_true", default=None)
    args = parser.parse_args()

    if args.list_scenarios:
        for name, s in sorted(SCENARIOS.items()):
            print(f"{name:24s} {s.description}")
        return 0

    cfg = Scenario(**asdict(SCENARIOS[args.scenario]))
    for knob in ["links_per_page", "sections", "depth", "taxonomies_per_page",
                 "taxonomy_count", "taxonomy_cardinality", "markdown_paragraphs",
                 "templating_frequency", "alias_frequency", "paginate_by",
                 "languages", "assets_frequency", "template_complexity",
                 "search", "minify", "feeds"]:
        value = getattr(args, knob, None)
        if value is not None:
            setattr(cfg, knob, value)

    out = args.out or Path("benchmarks/sites") / f"{cfg.name}-{args.pages}"
    manifest = generate(cfg, args.pages, args.seed, out)
    print(json.dumps(asdict(manifest), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
