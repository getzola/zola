//! Counting global allocator, compiled only with `--features alloc-stats`.
//!
//! Production builds never contain this: the feature is off by default, so the
//! binary keeps the system allocator with no wrapper at all. When enabled it
//! adds two relaxed atomic RMWs per allocation, which is enough to distort
//! absolute timings — use it to compare *allocation counts and bytes* between
//! phases and between revisions, not to measure wall time.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

use utils::timings::AllocSnapshot;

static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);
static LIVE: AtomicU64 = AtomicU64::new(0);
static PEAK: AtomicU64 = AtomicU64::new(0);

pub struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            record_alloc(layout.size() as u64);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let out = unsafe { System.realloc(ptr, layout, new_size) };
        if !out.is_null() {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
            let old = layout.size() as u64;
            let new = new_size as u64;
            if new > old {
                BYTES.fetch_add(new - old, Ordering::Relaxed);
                bump_live(new - old);
            } else {
                LIVE.fetch_sub(old - new, Ordering::Relaxed);
            }
        }
        out
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            record_alloc(layout.size() as u64);
        }
        ptr
    }
}

#[inline]
fn record_alloc(size: u64) {
    ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    BYTES.fetch_add(size, Ordering::Relaxed);
    bump_live(size);
}

#[inline]
fn bump_live(size: u64) {
    let live = LIVE.fetch_add(size, Ordering::Relaxed) + size;
    PEAK.fetch_max(live, Ordering::Relaxed);
}

pub fn snapshot() -> AllocSnapshot {
    AllocSnapshot {
        allocations: ALLOCATIONS.load(Ordering::Relaxed),
        bytes: BYTES.load(Ordering::Relaxed),
        live_bytes: LIVE.load(Ordering::Relaxed),
        peak_live_bytes: PEAK.load(Ordering::Relaxed),
    }
}
