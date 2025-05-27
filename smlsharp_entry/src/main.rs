#![no_main]

use std::{ffi::c_void,c_char, c_int, c_uint};

unsafe extern "C" {
    fn sml_init(argc: c_int, argv: *const *const c_char);
    fn sml_main();
    fn sml_load(a:*mut c_void);

    fn sml_exit(i: c_int);
    fn sml_gcroot_load(sml_loads: *const unsafe extern "C" fn(*mut c_void), count: c_uint);
}

#[cfg(target_os = "linux")] // other targets need other symbols
#[unsafe(no_mangle)]
extern "C" fn main(argc: c_int, argv: *const *const c_char) {
    use std::mem::transmute;
    let sml_load= [sml_load as unsafe extern "C" fn(*mut c_void)];
    unsafe { sml_init(argc, argv) };
    unsafe { sml_gcroot_load(sml_load.as_ptr(), 1) };
    unsafe { sml_main() };
    unsafe { sml_exit(0) };
}
