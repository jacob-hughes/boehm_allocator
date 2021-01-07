// Needed to avoid linking to gc twice. When rustgc is enabled, the test target
// will be built with a compiler already linked against libgc_internal. This
// stops duplicate symbol errors.
#![cfg_attr(not(all(test, feature = "rustgc_internal")), link(name = "gc"))]
extern "C" {
    pub(crate) fn GC_malloc(nbytes: usize) -> *mut u8;

    #[cfg(not(feature = "rustgc_internal"))]
    pub(crate) fn GC_malloc_uncollectable(nbytes: usize) -> *mut u8;

    pub(crate) fn GC_realloc(old: *mut u8, new_size: usize) -> *mut u8;

    pub(crate) fn GC_free(dead: *mut u8);

    pub(crate) fn GC_register_finalizer(
        ptr: *mut u8,
        finalizer: Option<unsafe extern "C" fn(*mut u8, *mut u8)>,
        client_data: *mut u8,
        old_finalizer: *mut extern "C" fn(*mut u8, *mut u8),
        old_client_data: *mut *mut u8,
    );

    pub(crate) fn GC_register_finalizer_no_order(
        ptr: *mut u8,
        finalizer: Option<unsafe extern "C" fn(*mut u8, *mut u8)>,
        client_data: *mut u8,
        old_finalizer: *mut extern "C" fn(*mut u8, *mut u8),
        old_client_data: *mut *mut u8,
    );

    pub(crate) fn GC_gcollect();

    pub(crate) fn GC_start_performance_measurement();

    pub(crate) fn GC_get_full_gc_total_time() -> usize;

    pub(crate) fn GC_get_prof_stats(
        prof_stats: *mut crate::ProfileStats,
        stats_size: usize,
    ) -> usize;

    #[cfg(feature = "rustgc_internal")]
    pub(crate) fn GC_malloc_explicitly_typed(size: usize, descriptor: usize) -> *mut u8;

    #[cfg(feature = "rustgc_internal")]
    pub(crate) fn GC_make_descriptor(bitmap: *const usize, len: usize) -> usize;

    #[cfg(feature = "rustgc_internal")]
    pub(crate) fn GC_malloc_atomic(nbytes: usize) -> *mut u8;
}
