use std::{alloc::GlobalAlloc, ffi::c_void, process::abort};

use libc::{malloc,realloc, size_t};

#[unsafe(no_mangle)]
pub extern "C" fn sml_xmalloc(size: size_t) -> *mut c_void {
    let p = unsafe { malloc(size) };
    if p.is_null() {
        tracing::error!("malloc");
        abort();
    }
    return p;
}

#[unsafe(no_mangle)]
pub extern "C" fn sml_xrealloc(p: *mut c_void, size: size_t) -> *mut c_void {
    let p = unsafe { realloc(p, size) };
    if p.is_null() {
        tracing::error!("realloc");
        abort();
    }
    return p;
}
