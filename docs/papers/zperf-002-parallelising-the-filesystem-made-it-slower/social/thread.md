<!-- derived from ../paper.md; ZPERF-002 -->

**1/**
We profiled a static-site generator, built a ranked hotspot backlog, and then
implemented and threw away four items on it. Three were filesystem work. All
three failed the same way. Here is the write-up nobody publishes.

**2/**
Idea one: a build calls `create_dir_all` before every output file, and the
profile blamed it for 18.9% of busy CPU. Cache the directories already created
and skip the syscalls. Two variants, shared and thread-local. Write phase 7.71 s
→ 7.67 s. Build 1.31 s → 1.39 s. On a binary whose own spread was 1.03–1.37 s.
Samples parked in the kernel on a path are not time that path can give back.

**3/**
Idea two: deleting the previous output is 663 ms, 36.3% of wall. Its top-level
entries are independent subtrees, so delete them in parallel. Result: slower on
two of three phase samples and on all three whole-build samples. Concurrent
`unlink` storms contend instead of overlapping.

**4/**
Idea two, second attempt: don't make the delete faster, take it off the critical
path. Rename the old output aside, delete it on a background thread, join at the
end. The phase went 929.8 ms → 0.2 ms. Wall time: unchanged. The build already
saturates every core and the same disk, so the deleter competes with it.

**5/**
Idea three, the best one: parallelise the copy of the static tree. On the real
tree it did nothing — 55 MB in ~190 ms is ~290 MB/s, which is the disk, not the
loop. So we re-ran it on the case parallelism should win: 5000 files of 1 KB,
where per-file syscall latency dominates.

**6/**
It was 10–30% slower. Every round. Twelve threads creating files in the same
handful of directories contend on directory metadata, and each copy also probes
for its parent, so they hammer the same paths at once.

**7/**
Three experiments, three phases, one rule: on this platform, filesystem metadata
operations do not parallelise — they anti-parallelise. Bulk throughput is the
disk's business, and the loop around it is not what costs.

**8/**
Caveats, because they matter: one machine, macOS on APFS. Network storage, where
per-operation latency is high enough that concurrency hides it, is a different
question. And these are transcribed phase timings, not committed benchmark JSON
— the rejected code was reverted.

**9/**
The fourth rejection is a different lesson. An item to remove one `stat` per
`load_data` call was correct when filed and wrong when it came up: after the fix
ranked above it landed, every `stat` in the entire build measured 439 ms of self
time. A ranked backlog is a snapshot; fixing the top item invalidates rows
below.

**10/**
What we kept is the record, with the numbers that killed each one. The failing
ideas are the attractive ones, so they recur — and without the write-up, the
next person to have the parallel-copy idea pays for the whole experiment again.
Paper: ZPERF-002 in the *Zola at Scale* series. Our fork, not upstream, and
nothing here shipped.
