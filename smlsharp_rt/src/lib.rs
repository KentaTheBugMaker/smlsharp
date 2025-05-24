use std::os::raw::c_ulong;

use libc::{c_char, c_int};
use signal::SmlCheckHookFn;

mod signal;

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
    fn sml_warn(err:c_int,format : *const c_char,...);
    fn sml_debug(format : *const c_char,...);
    fn sml_set_check_hook(hook:SmlCheckHookFn);
}
