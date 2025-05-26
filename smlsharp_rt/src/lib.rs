#![feature(c_variadic)]
use std::os::raw::{c_ulong, c_void};

use libc::{c_char, c_int};
use signal::SmlCheckHookFn;

mod error;
mod signal;
mod xmalloc;
mod top;
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

unsafe extern "C" {
    fn sml_gc(greedy: i32) -> c_ulong;
    fn sml_set_check_hook(hook: SmlCheckHookFn);
    fn sml_obj_enum_ptr(obj :*mut c_void,trace: extern "C" fn (*mut *mut c_void,*mut c_void), data:*mut c_void);
}
