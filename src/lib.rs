//! This library acts as a shim to prevent static linking the Boehm GC directly
//! inside library/alloc which causes surprising and hard to debug errors.

#![no_std]
#![feature(rustc_private)]
#![feature(allocator_api)]
#![feature(nonnull_slice_from_raw_parts)]
#![cfg_attr(feature = "rustgc_internal", feature(gc))]
#![feature(specialization)]
#![feature(negative_impls)]
#![allow(incomplete_features)]

use core::{
    alloc::{AllocError, Allocator, GlobalAlloc, Layout},
    ptr::NonNull,
};

pub struct GcAllocator;

mod boehm;
#[cfg(feature = "rustgc_internal")]
mod specializer;

unsafe impl GlobalAlloc for GcAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        #[cfg(feature = "rustgc_internal")]
        return boehm::GC_malloc(layout.size()) as *mut u8;
        #[cfg(not(feature = "rustgc_internal"))]
        return boehm::GC_malloc_uncollectable(layout.size()) as *mut u8;
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        boehm::GC_free(ptr);
    }

    unsafe fn realloc(&self, ptr: *mut u8, _: Layout, new_size: usize) -> *mut u8 {
        boehm::GC_realloc(ptr, new_size) as *mut u8
    }
}

unsafe impl Allocator for GcAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = unsafe { boehm::GC_malloc(layout.size()) } as *mut u8;
        assert!(!ptr.is_null());
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn deallocate(&self, _: NonNull<u8>, _: Layout) {}
}

impl GcAllocator {
    #[cfg(feature = "rustgc_internal")]
    pub fn maybe_optimised_alloc<T>(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let sp = specializer::AllocationSpecializer::new();
        sp.maybe_optimised_alloc::<T>(layout)
    }

    pub fn force_gc() {
        unsafe { boehm::GC_gcollect() }
    }

    pub unsafe fn register_finalizer(
        &self,
        obj: *mut u8,
        finalizer: Option<unsafe extern "C" fn(*mut u8, *mut u8)>,
        client_data: *mut u8,
        old_finalizer: *mut extern "C" fn(*mut u8, *mut u8),
        old_client_data: *mut *mut u8,
    ) {
        boehm::GC_register_finalizer_no_order(
            obj,
            finalizer,
            client_data,
            old_finalizer,
            old_client_data,
        )
    }

    pub fn unregister_finalizer(&self, gcbox: *mut u8) {
        unsafe {
            boehm::GC_register_finalizer(
                gcbox,
                None,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
            );
        }
    }

    pub fn get_stats() -> GcStats {
        let mut ps = ProfileStats::default();
        unsafe {
            boehm::GC_get_prof_stats(
                &mut ps as *mut ProfileStats,
                core::mem::size_of::<ProfileStats>(),
            );
        }
        let total_gc_time = unsafe { boehm::GC_get_full_gc_total_time() };

        GcStats {
            total_gc_time,
            num_collections: ps.gc_no,
            total_freed: ps.bytes_reclaimed_since_gc,
            total_alloced: ps.bytes_allocd_since_gc,
        }
    }

    pub fn init() {
        unsafe { boehm::GC_start_performance_measurement() };
    }
}

#[repr(C)]
#[derive(Default)]
pub struct ProfileStats {
    /// Heap size in bytes (including area unmapped to OS).
    pub(crate) heapsize_full: usize,
    /// Total bytes contained in free and unmapped blocks.
    pub(crate) free_bytes_full: usize,
    /// Amount of memory unmapped to OS.
    pub(crate) unmapped_bytes: usize,
    /// Number of bytes allocated since the recent collection.
    pub(crate) bytes_allocd_since_gc: usize,
    /// Number of bytes allocated before the recent collection.
    /// The value may wrap.
    pub(crate) allocd_bytes_before_gc: usize,
    /// Number of bytes not considered candidates for garbage collection.
    pub(crate) non_gc_bytes: usize,
    /// Garbage collection cycle number.
    /// The value may wrap.
    pub(crate) gc_no: usize,
    /// Number of marker threads (excluding the initiating one).
    pub(crate) markers_m1: usize,
    /// Approximate number of reclaimed bytes after recent collection.
    pub(crate) bytes_reclaimed_since_gc: usize,
    /// Approximate number of bytes reclaimed before the recent collection.
    /// The value may wrap.
    pub(crate) reclaimed_bytes_before_gc: usize,
    /// Number of bytes freed explicitly since the recent GC.
    pub(crate) expl_freed_bytes_since_gc: usize,
}

#[derive(Debug)]
pub struct GcStats {
    total_gc_time: usize, // In milliseconds.
    num_collections: usize,
    total_freed: usize,   // In bytes
    total_alloced: usize, // In bytes
}
