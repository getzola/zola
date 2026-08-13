<!-- derived from ../paper.md; ZPERF-002 -->

**Parallelising the filesystem made it slower.**

We ran a performance program on a fork of the static-site generator Zola,
profiled it, and built a ranked backlog of hotspots. Four of those items were
implemented and then thrown away. Three of them were filesystem work, and all
three failed the same way.

**Caching the directories a build has already created.** The CPU profile
attributed 18.9% of busy CPU to `create_dir_all`. Two variants — a shared set
behind a mutex, and a thread-local one — moved the write phase from 7.71 s to
7.67 s and the build from 1.31 s to 1.39 s, on a binary whose own run-to-run
spread was 1.03–1.37 s. Nothing.

**Parallelising the delete of the previous output.** It is 663 ms and 36.3% of
wall on our reference workload, and its top-level entries are independent
subtrees. Parallel deletion was slower on two of three phase samples and on all
three whole-build samples. Then we renamed the tree aside and deleted it on a
background thread instead: the phase went 929.8 ms → 0.2 ms and wall time did
not move at all, because the build already saturates every core and the same
disk. Moving work is not removing it.

**Parallelising the copy of the static tree.** On the real tree it did nothing —
55 MB in ~190 ms is ~290 MB/s, which is the disk, not the loop. So we re-ran it
on the case parallelism should win, 5000 files of 1 KB, where per-file syscall
latency dominates. It was 10–30% slower, in every round.

Three experiments, three phases, one conclusion:

> On this platform, filesystem metadata operations do not parallelise. They
> anti-parallelise. Bulk data throughput is the disk's business, and the loop
> around it is not what costs.

The honest caveats: one machine (Apple M4 Pro, macOS, APFS), and these are
transcribed phase timings rather than committed benchmark JSON, because the
rejected code was reverted. A different filesystem — or network storage, where
per-operation latency is high enough for concurrency to hide it — may behave
completely differently.

The fourth rejection is a different lesson. A backlog item to remove one `stat`
per `load_data` call was correct when it was filed and wrong by the time it came
up: after the fix ranked above it landed, a re-profile showed that *every*
`stat` in the whole build is 439 ms of self time. A ranked hotspot list is a
snapshot, and fixing its top item invalidates the rows below.

What we kept is the record. The ideas that fail are the attractive ones, so they
recur — and without the write-up, the next person to have the parallel-copy idea
pays for the whole experiment again.

Full paper, with every round-level number and where it came from: ZPERF-002 in
the *Zola at Scale* series. This is work in our own fork; it is not affiliated
with or endorsed by the upstream Zola project, and none of these changes shipped
anywhere.
