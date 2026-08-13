# Paper review checklist

Run before moving a paper to `review`, and again before a human moves it to
`published`. `scripts/dev.sh papers validate` mechanises the parts a machine can
check; everything below the line it cannot.

## Mechanical — `scripts/dev.sh papers validate`

* [ ] metadata parses, required fields present, `id` and `slug` unique
* [ ] `status` and `type` are known values
* [ ] every referenced `PERF-*` item exists in `docs/performance/HOTSPOTS.md`
* [ ] every referenced file, figure and intra-tree link resolves
* [ ] every declared measurement with a benchmark source matches that artifact
* [ ] every declared figure string appears in `paper.md`
* [ ] every number in a social derivative appears in `paper.md`
* [ ] no absolute local path, home directory or private site path
* [ ] no `TODO`, `TBD`, `FIXME`, `XXX` or unfilled template marker
* [ ] `INDEX.md` is current

## Editorial — a human, or an agent being careful

### Claims

* [ ] Every quantitative claim traces to an artifact or a described observation.
* [ ] No claim is stated more strongly than its class allows.
* [ ] Hypotheses are not phrased as results; proposals are in the future tense.
* [ ] No predicted number is presented as a measured one.

### Measurement

* [ ] Baseline and candidate are identified by commit and by what they are.
* [ ] The workload is described: page count, output size, what it represents.
* [ ] Machine and its condition during the run are disclosed.
* [ ] Round count, interleaving and the statistic used are stated.
* [ ] Results the harness called unresolved are reported as unresolved.
* [ ] Scenarios where the change did nothing are shown, not omitted.

### Honesty

* [ ] Negative results that teach something are present.
* [ ] Measurement mistakes made along the way are disclosed.
* [ ] The costs of each change are stated next to its benefits.
* [ ] Nothing is rounded in the flattering direction.

### Attribution

* [ ] Upstream behaviour, fork changes, experiments and proposals are distinct.
* [ ] Any bug attributed to upstream names the baseline it was reproduced on.
* [ ] No implied affiliation with or endorsement by the upstream project.

### Correctness

* [ ] How output equivalence was established is described.
* [ ] Any deliberate behaviour change is called out as such.

### Reproduction

* [ ] Every command in the reproduction section exists and was run.
* [ ] Nothing depends on data that cannot be redistributed without saying so.

### Hygiene

* [ ] No private paths, usernames, hostnames, tokens or unrelated project names.
* [ ] Derivatives carry the same numbers as the paper, in the same units.
* [ ] Markdown renders: tables aligned, code fences closed, links resolve.

## Sign-off

An agent may complete everything above and set `status = "review"`. Only a human
who has read the paper and stands by it sets `status = "published"`.
