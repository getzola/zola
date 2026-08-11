# Architecture

Engineering notes about how Zola is built, kept next to `docs/performance/`.
Neither directory is part of the published documentation site — the site is
`docs/content/`, `docs/templates/` and `docs/config.toml`.

| File | Contents |
| ---- | -------- |
| `COMPONENTS.md` | The workspace map: layer, responsibility, dependencies, tests, performance sensitivity. **Generated** — see below. |
| `decisions/` | Architecture decision records, and the template for writing one. |

`COMPONENTS.md` is produced by `scripts/dev/repo_map.py` from the crate
manifests plus `scripts/dev/components.toml`. Never edit it by hand:

```bash
scripts/dev.sh map        # verify invariants and check for drift
scripts/dev.sh generate   # regenerate after changing a manifest
```

The same command also enforces the invariants recorded in
[decisions/0001](decisions/0001-crate-layering.md): the crate graph stays a DAG,
every component is described, and the forbidden dependency edges stay absent.
