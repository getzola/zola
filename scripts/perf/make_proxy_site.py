#!/usr/bin/env python3
"""Build a *content-faithful proxy* of an external Zola site.

Why this exists
---------------
The reference workload for this program (`~/dev/vomaste.cz`, ~3.7k pages,
~1.6k sections) targets Zola 0.22: 39 of its 40 templates use Tera 1
`{% import %}` macros, which Zola 0.23 removed. It therefore cannot be built
with the binary we are optimising, so it cannot be used directly as an A/B
benchmark.

A proxy keeps everything that determines *load, parse, markdown, index, cache,
render-count and write* cost — the real content tree, the real front matter, the
real internal-link graph, the real section shape, the real static files — and
substitutes the one part that cannot be reused: the templates.

What this measures and what it does NOT
---------------------------------------
MEASURES faithfully : content discovery, front matter parsing, markdown
                      rendering, permalink/link resolution, section graph,
                      RenderCache construction, number of rendered outputs,
                      minification, output writing, static copying.
DOES NOT measure    : the real templates' cost. Template work is replaced by a
                      generic "rich" template set. With --emulate-view-models
                      the proxy does reproduce the real site's per-page
                      `load_data()` pattern, which is the dominant template-side
                      cost on data-driven sites.

The source site is opened read-only; nothing is ever written back to it.
The proxy lands in benchmarks/proxies/, which is gitignored — external content
must never be committed to this repository.
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
import tomllib
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

TEMPLATE_RE = re.compile(r'^\s*(?:page_)?template\s*=\s*"([^"]+)"', re.M)
VIEW_MODEL_RE = re.compile(r'^\s*view_model\s*=\s*"([^"]+)"', re.M)

BASE = """<!DOCTYPE html>
<html lang="{{ lang }}">
<head>
<meta charset="utf-8">
<title>{% block title %}{{ config.title }}{% endblock %}</title>
<meta name="description" content="{% block description %}{{ config.description }}{% endblock %}">
<link rel="canonical" href="{{ current_url | default(value='') }}">
</head>
<body>
<header><a href="{{ get_url(path='/') }}">{{ config.title }}</a></header>
<main>{% block content %}{% endblock %}</main>
<footer><p>{{ config.title }}</p></footer>
</body>
</html>
"""

PAGE = """{% extends 'base.html' %}
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
<div class="toc">{% for h in page.toc %}<a href="{{ h.permalink }}">{{ h.title }}</a>{% endfor %}</div>
__VIEW_MODEL__
{{ page.content | safe }}
{% if page.backlinks %}
<aside><ul>{% for b in page.backlinks %}<li><a href="{{ b.permalink }}">{{ b.title }}</a></li>{% endfor %}</ul></aside>
{% endif %}
</article>
{% endblock %}
"""

VIEW_MODEL_BLOCK = """{% if page.extra.view_model %}
{% set vm = load_data(path='data/' ~ page.extra.view_model, required=false) %}
{% if vm %}<section class="view-model" data-rows="{{ vm | length }}"></section>{% endif %}
{% endif %}"""

SECTION = """{% extends 'base.html' %}
{% block title %}{{ section.title }} — {{ config.title }}{% endblock %}
{% block content %}
<h1>{{ section.title }}</h1>
{{ section.content | safe }}
<ul class="pages">
{% for page in section.pages %}<li><a href="{{ page.permalink }}">{{ page.title }}</a></li>{% endfor %}
</ul>
<ul class="subsections">
{% for sub in section.subsections %}
  {% set s = get_section(path=sub) %}
  <li><a href="{{ s.permalink }}">{{ s.title }}</a></li>
{% endfor %}
</ul>
{% endblock %}
"""

INDEX = """{% extends 'base.html' %}
{% block content %}
<h1>{{ config.title }}</h1>
{{ section.content | safe }}
<ul>{% for sub in section.subsections %}{% set s = get_section(path=sub) %}<li><a href="{{ s.permalink }}">{{ s.title }}</a></li>{% endfor %}</ul>
{% endblock %}
"""

NOT_FOUND = """{% extends 'base.html' %}
{% block content %}<h1>Not found</h1>{% endblock %}
"""

TAXONOMY_LIST = """{% extends 'base.html' %}
{% block content %}<ul>{% for term in terms %}<li><a href="{{ term.permalink }}">{{ term.name }}</a></li>{% endfor %}</ul>{% endblock %}
"""

TAXONOMY_SINGLE = """{% extends 'base.html' %}
{% block content %}<ul>{% for page in term.pages %}<li><a href="{{ page.permalink }}">{{ page.title }}</a></li>{% endfor %}</ul>{% endblock %}
"""

# Config keys that are safe (and relevant) to carry over verbatim.
CARRIED_SCALARS = [
    "base_url", "title", "description", "default_language", "output_dir",
    "compile_sass", "minify_html", "build_search_index", "generate_feeds",
    "generate_sitemap", "generate_robots_txt", "hard_link_static",
    "feed_limit", "preserve_dotfiles_in_output", "author", "taxonomy_root",
]


def load_config(site: Path) -> tuple[dict, Path]:
    for name in ("zola.toml", "config.toml"):
        p = site / name
        if p.exists():
            return tomllib.loads(p.read_text(encoding="utf-8")), p
    raise SystemExit(f"no zola.toml/config.toml in {site}")


def scan_content(content: Path) -> tuple[Counter, int, list[str], int]:
    templates: Counter = Counter()
    view_models = 0
    tera_syntax: list[str] = []
    md_files = 0
    for md in sorted(content.rglob("*.md")):
        md_files += 1
        try:
            text = md.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        head = text[:4000]
        for m in TEMPLATE_RE.finditer(head):
            templates[m.group(1)] += 1
        if VIEW_MODEL_RE.search(head):
            view_models += 1
        if "{{" in text or "{%" in text:
            tera_syntax.append(str(md.relative_to(content)))
    return templates, view_models, tera_syntax, md_files


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--source", type=Path, required=True)
    ap.add_argument("--out", type=Path, default=None)
    ap.add_argument("--with-static", action="store_true", default=True)
    ap.add_argument("--no-static", dest="with_static", action="store_false")
    ap.add_argument("--with-data", action="store_true",
                    help="copy data/ (needed for --emulate-view-models)")
    ap.add_argument("--emulate-view-models", action="store_true",
                    help="make page.html call load_data() on page.extra.view_model, "
                         "reproducing the reference site's per-page data loading")
    args = ap.parse_args()

    source = args.source.expanduser().resolve()
    content = source / "content"
    if not content.exists():
        raise SystemExit(f"{source} has no content/ directory")

    out = (args.out or REPO / "benchmarks" / "proxies" / f"{source.name}-proxy").resolve()
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)

    cfg, cfg_path = load_config(source)
    templates, view_models, tera_syntax, md_files = scan_content(content)

    print(f"[proxy] source        : {source}")
    print(f"[proxy] markdown files: {md_files}")
    print(f"[proxy] templates ref'd: {len(templates)}")
    print(f"[proxy] pages with view_model: {view_models}")
    print(f"[proxy] md files containing Tera syntax: {len(tera_syntax)}")

    # ---- content (verbatim, including colocated assets) --------------------
    shutil.copytree(content, out / "content", symlinks=False)

    if args.with_static and (source / "static").exists():
        shutil.copytree(source / "static", out / "static", symlinks=False)
    if args.with_data and (source / "data").exists():
        shutil.copytree(source / "data", out / "data", symlinks=False)

    # ---- config ------------------------------------------------------------
    lines = []
    for key in CARRIED_SCALARS:
        if key in cfg:
            value = cfg[key]
            if isinstance(value, bool):
                lines.append(f"{key} = {'true' if value else 'false'}")
            elif isinstance(value, (int, float)):
                lines.append(f"{key} = {value}")
            else:
                lines.append(f'{key} = "{value}"')
    # A 0.22-era site may still contain shortcode calls (`{% callout(...) %}`) or
    # literal Tera in markdown. Zola 0.23 templates markdown before parsing it, so
    # those files would abort the build. `skip_content_templating` is the exact
    # escape hatch for that; it affects only these files and is recorded in the
    # manifest so the substitution is never invisible.
    if tera_syntax:
        rendered = ", ".join(f'"{p}"' for p in tera_syntax)
        lines.append(f"skip_content_templating = [{rendered}]")

    taxonomies = cfg.get("taxonomies", [])
    if taxonomies:
        rendered = ", ".join(
            "{ " + ", ".join(
                f'{k} = "{v}"' if isinstance(v, str)
                else f"{k} = {'true' if v is True else 'false' if v is False else v}"
                for k, v in t.items()
            ) + " }"
            for t in taxonomies
        )
        lines.append(f"taxonomies = [{rendered}]")
    else:
        lines.append("taxonomies = []")
    if "link_checker" in cfg:
        lines.append("[link_checker]")
        # Keep the checker permissive: the proxy is a benchmark, not a link audit,
        # and the real site's own templates are what normally satisfy some links.
        lines.append('internal_level = "warn"')
        lines.append('external_level = "warn"')
    if "markdown" in cfg:
        md_cfg = cfg["markdown"]
        highlighting = md_cfg.get("highlighting")
        if isinstance(highlighting, dict):
            lines.append("[markdown.highlighting]")
            for k, v in highlighting.items():
                if isinstance(v, str):
                    lines.append(f'{k} = "{v}"')
                elif isinstance(v, bool):
                    lines.append(f"{k} = {'true' if v else 'false'}")
        elif md_cfg:
            lines.append("[markdown]")
            for k, v in md_cfg.items():
                if isinstance(v, bool):
                    lines.append(f"{k} = {'true' if v else 'false'}")
                elif isinstance(v, str):
                    lines.append(f'{k} = "{v}"')
    # `extra` is carried over because content/templates may read config.extra.*
    if "extra" in cfg:
        lines.append("")
        lines.append("[extra]")
        for k, v in cfg["extra"].items():
            if isinstance(v, bool):
                lines.append(f"{k} = {'true' if v else 'false'}")
            elif isinstance(v, (int, float)):
                lines.append(f"{k} = {v}")
            elif isinstance(v, str):
                lines.append(f'{k} = "{v}"')
    (out / "config.toml").write_text("\n".join(lines) + "\n", encoding="utf-8")

    # ---- templates ---------------------------------------------------------
    tpl_dir = out / "templates"
    tpl_dir.mkdir()
    page_tpl = PAGE.replace("__VIEW_MODEL__",
                            VIEW_MODEL_BLOCK if args.emulate_view_models else "")
    (tpl_dir / "base.html").write_text(BASE, encoding="utf-8")
    (tpl_dir / "page.html").write_text(page_tpl, encoding="utf-8")
    (tpl_dir / "section.html").write_text(SECTION, encoding="utf-8")
    (tpl_dir / "index.html").write_text(INDEX, encoding="utf-8")
    (tpl_dir / "404.html").write_text(NOT_FOUND, encoding="utf-8")
    if taxonomies:
        (tpl_dir / "taxonomy_list.html").write_text(TAXONOMY_LIST, encoding="utf-8")
        (tpl_dir / "taxonomy_single.html").write_text(TAXONOMY_SINGLE, encoding="utf-8")

    # Every template name referenced from front matter must exist. We cannot know
    # whether the original was a page or a section template, so we look at how it
    # is used: names referenced by `_index.md` files get the section template.
    section_template_names: set[str] = set()
    for md in content.rglob("_index*.md"):
        try:
            head = md.read_text(encoding="utf-8", errors="replace")[:4000]
        except OSError:
            continue
        for m in TEMPLATE_RE.finditer(head):
            section_template_names.add(m.group(1))

    generated = 0
    for name in sorted(templates):
        target = tpl_dir / name
        if target.exists():
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(SECTION if name in section_template_names else page_tpl,
                          encoding="utf-8")
        generated += 1

    manifest = {
        "source": str(source),
        "source_config": str(cfg_path.name),
        "md_files": md_files,
        "templates_referenced": dict(templates.most_common()),
        "templates_generated": generated,
        "pages_with_view_model": view_models,
        "md_files_with_tera_syntax": len(tera_syntax),
        "skip_content_templating": tera_syntax,
        "emulate_view_models": args.emulate_view_models,
        "copied_static": args.with_static,
        "copied_data": args.with_data,
        "caveat": "Templates are synthetic. Content, front matter, links, section "
                  "shape and static assets are the real ones.",
    }
    (out / "proxy-manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n",
                                             encoding="utf-8")
    print(f"[proxy] generated {generated} substitute template(s)")
    print(f"[proxy] wrote {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
