//! Developer-only build phase timing.
//!
//! Enabled at runtime by `zola build --timings`. When disabled — which is the
//! case for every normal build — each instrumentation point costs one relaxed
//! atomic load and nothing else: no allocation, no locking, no clock read.
//!
//! Two kinds of measurement are recorded, because a build has two kinds of
//! phase:
//!
//! * **Sequential spans** ([`span`]) form a tree. Their duration is wall-clock
//!   time on the thread that opened them, and parent/child links are tracked
//!   with a per-thread stack, so the report reads like a call tree.
//! * **Parallel accumulators** ([`accumulate`], [`measure`]) sum per-item
//!   durations inside rayon loops. Their total is CPU time summed across worker
//!   threads and will normally exceed the wall time of the enclosing span; they
//!   are reported separately to make that explicit.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

static ENABLED: AtomicBool = AtomicBool::new(false);
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/// Allocation counters, when the binary was built with a counting allocator.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AllocSnapshot {
    pub allocations: u64,
    pub bytes: u64,
    pub live_bytes: u64,
    pub peak_live_bytes: u64,
}

static ALLOC_PROBE: OnceLock<fn() -> AllocSnapshot> = OnceLock::new();

/// Register a function that reports current allocation counters. When set,
/// every span additionally reports how many allocations and bytes happened
/// inside it. Used by the `alloc-stats` build of the CLI.
pub fn set_alloc_probe(probe: fn() -> AllocSnapshot) {
    let _ = ALLOC_PROBE.set(probe);
}

fn alloc_snapshot() -> Option<AllocSnapshot> {
    ALLOC_PROBE.get().map(|probe| probe())
}

#[derive(Debug)]
struct SpanRecord {
    id: usize,
    parent: Option<usize>,
    name: &'static str,
    duration: Duration,
    /// (allocations, bytes) that happened while this span was open, including
    /// its children and any work other threads did meanwhile.
    alloc_delta: Option<(u64, u64)>,
    live_bytes_after: Option<u64>,
}

#[derive(Debug)]
struct Accumulator {
    name: &'static str,
    total: Duration,
    calls: u64,
}

#[derive(Default)]
struct Registry {
    spans: Vec<SpanRecord>,
    accumulators: Vec<Accumulator>,
}

fn registry() -> &'static Mutex<Registry> {
    static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
}

thread_local! {
    /// Stack of open span ids on this thread.
    static STACK: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

/// Turn timing collection on. Called once, from the CLI.
pub fn enable() {
    ENABLED.store(true, Ordering::Relaxed);
}

#[inline]
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Open a sequential span; the guard records elapsed time when dropped.
#[inline]
pub fn span(name: &'static str) -> Span {
    if !is_enabled() {
        return Span { name, id: 0, parent: None, start: None, alloc_start: None };
    }
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let parent = STACK.with(|s| {
        let mut stack = s.borrow_mut();
        let parent = stack.last().copied();
        stack.push(id);
        parent
    });
    Span { name, id, parent, start: Some(Instant::now()), alloc_start: alloc_snapshot() }
}

pub struct Span {
    name: &'static str,
    id: usize,
    parent: Option<usize>,
    start: Option<Instant>,
    alloc_start: Option<AllocSnapshot>,
}

impl Drop for Span {
    fn drop(&mut self) {
        let Some(start) = self.start else { return };
        let duration = start.elapsed();
        let end = alloc_snapshot();
        let alloc_delta = match (self.alloc_start, end) {
            (Some(a), Some(b)) => {
                Some((b.allocations.saturating_sub(a.allocations), b.bytes.saturating_sub(a.bytes)))
            }
            _ => None,
        };
        STACK.with(|s| {
            let mut stack = s.borrow_mut();
            if let Some(pos) = stack.iter().rposition(|&id| id == self.id) {
                stack.truncate(pos);
            }
        });
        registry().lock().expect("timings registry").spans.push(SpanRecord {
            id: self.id,
            parent: self.parent,
            name: self.name,
            duration,
            alloc_delta,
            live_bytes_after: end.map(|e| e.live_bytes),
        });
    }
}

/// Add `duration` to a named accumulator. Use inside parallel loops, where a
/// span tree would be meaningless.
#[inline]
pub fn accumulate(name: &'static str, duration: Duration) {
    if !is_enabled() {
        return;
    }
    let mut reg = registry().lock().expect("timings registry");
    if let Some(entry) = reg.accumulators.iter_mut().find(|a| a.name == name) {
        entry.total += duration;
        entry.calls += 1;
    } else {
        reg.accumulators.push(Accumulator { name, total: duration, calls: 1 });
    }
}

/// Time a closure into an accumulator, returning its value.
#[inline]
pub fn measure<T>(name: &'static str, f: impl FnOnce() -> T) -> T {
    if !is_enabled() {
        return f();
    }
    let start = Instant::now();
    let out = f();
    accumulate(name, start.elapsed());
    out
}

fn fmt_duration(d: Duration) -> String {
    let secs = d.as_secs_f64();
    if secs >= 1.0 { format!("{secs:.3} s") } else { format!("{:.1} ms", secs * 1000.0) }
}

fn fmt_bytes(bytes: u64) -> String {
    const UNITS: [(&str, f64); 4] = [("GB", 1e9), ("MB", 1e6), ("KB", 1e3), ("B", 1.0)];
    for (suffix, scale) in UNITS {
        if bytes as f64 >= scale {
            return format!("{:.1} {suffix}", bytes as f64 / scale);
        }
    }
    "0 B".to_string()
}

fn fmt_count(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1e6)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1e3)
    } else {
        n.to_string()
    }
}

fn write_tree(
    out: &mut String,
    spans: &[SpanRecord],
    parent: Option<usize>,
    depth: usize,
    total: f64,
    with_alloc: bool,
) {
    // Spans are pushed on completion, so ids (assigned on entry) give start order.
    let mut children: Vec<&SpanRecord> = spans.iter().filter(|s| s.parent == parent).collect();
    children.sort_by_key(|s| s.id);
    for child in children {
        let share = if total > 0.0 {
            format!("{:>7.1}%", child.duration.as_secs_f64() / total * 100.0)
        } else {
            "      -".to_string()
        };
        out.push_str(&format!(
            "{:<46}{:>12}{:>9}",
            format!("{}{}", "  ".repeat(depth), child.name),
            fmt_duration(child.duration),
            share
        ));
        if with_alloc {
            let (allocs, bytes) = child.alloc_delta.unwrap_or((0, 0));
            out.push_str(&format!(
                "{:>10}{:>12}{:>12}",
                fmt_count(allocs),
                fmt_bytes(bytes),
                child.live_bytes_after.map(fmt_bytes).unwrap_or_else(|| "-".into())
            ));
        }
        out.push('\n');
        write_tree(out, spans, Some(child.id), depth + 1, total, with_alloc);
    }
}

/// Render the collected timings. Returns `None` if timings are off or nothing
/// was recorded.
pub fn report() -> Option<String> {
    if !is_enabled() {
        return None;
    }
    let reg = registry().lock().expect("timings registry");
    if reg.spans.is_empty() && reg.accumulators.is_empty() {
        return None;
    }

    let total: f64 =
        reg.spans.iter().filter(|s| s.parent.is_none()).map(|s| s.duration.as_secs_f64()).sum();

    let with_alloc = reg.spans.iter().any(|s| s.alloc_delta.is_some());
    let mut out = String::from("\nBuild timings\n");
    out.push_str(&format!("{:<46}{:>12}{:>9}", "phase", "time", "share"));
    if with_alloc {
        out.push_str(&format!("{:>10}{:>12}{:>12}", "allocs", "alloc'd", "live after"));
    }
    out.push('\n');
    out.push_str(&format!("{}\n", "-".repeat(if with_alloc { 101 } else { 67 })));
    write_tree(&mut out, &reg.spans, None, 0, total, with_alloc);
    if let Some(peak) = alloc_snapshot().map(|s| s.peak_live_bytes) {
        out.push_str(&format!("\npeak live heap: {}\n", fmt_bytes(peak)));
    }

    if !reg.accumulators.is_empty() {
        let mut accs: Vec<&Accumulator> = reg.accumulators.iter().collect();
        accs.sort_by(|a, b| b.total.cmp(&a.total));
        out.push_str(&format!(
            "\n{:<46}{:>12}{:>9}{:>12}\n",
            "parallel work (CPU time summed over threads)", "time", "calls", "per call"
        ));
        out.push_str(&format!("{}\n", "-".repeat(79)));
        for a in accs {
            let per_call =
                if a.calls > 0 { fmt_duration(a.total / a.calls as u32) } else { "-".to_string() };
            out.push_str(&format!(
                "{:<46}{:>12}{:>9}{:>12}\n",
                a.name,
                fmt_duration(a.total),
                a.calls,
                per_call
            ));
        }
    }

    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_returns_inner_value() {
        assert_eq!(measure("x", || 41 + 1), 42);
    }

    #[test]
    fn disabled_instrumentation_records_nothing() {
        // ENABLED is process-global; this test is meaningful only while it is off,
        // which is the case unless another test enables it.
        if !is_enabled() {
            let _s = span("noop");
            accumulate("noop", Duration::from_millis(1));
            assert!(report().is_none());
        }
    }
}
