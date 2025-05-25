use std::{
    ffi::{CString, c_char},
    mem::transmute,
    sync::atomic::Ordering::Relaxed,
};

use libc::{SIG_DFL, SIGHUP, SIGINT, SIGPIPE, SIGTERM, c_int, sigaction, sigemptyset};

use crate::{sml_gc, sml_set_check_hook};

static SIGNALS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
pub type SmlCheckHookFn = unsafe extern "C" fn() -> ();
static SIGNAL_HOOK: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[unsafe(no_mangle)]
/// src/compiler/main/main/SignalHandler.sml より以下のように定義している
/// ``` SML
///  val signum_SIGHUP = 0w1
///  val signum_SIGINT = 0w2
///  val signum_SIGPIPE = 0w13
///  val signum_SIGALRM = 0w14
///  val signum_SIGTERM = 0w15
/// ```
///signal_handler が処理したシグナルの種類を記録する.
/// 0x00_00_00_00 => 何も処理していない
/// 0x00_00_00_02 => SIGHUPを処理した
/// 0x00_00_00_04 => SIGINTを処理した
/// 0x00_00_40_00 => SIGPIPEを処理した
/// 0x00_00_80_00 => SIGALRMを処理した
/// 0x00_01_00_00 => SIGTERMを処理した
/// 戻り値  上で示した値の和
///
/// 副作用　この関数の実行後はシグナル集合が空になる
///
///
extern "C" fn sml_signal_check() -> u32 {
    tracing::debug!(target:"sml_signal_check","Rust impl signal.rs enter");
    let r = SIGNALS.load(Relaxed);
    tracing::debug!(target:"sml_signal_check","Rust impl signal.rs exit");
    if r != 0 { SIGNALS.swap(0, Relaxed) } else { 0 }
}

const CHAR_BIT: usize = 8;

extern "C" fn signal_handler(signum: i32) {
    tracing::debug!(target:"signal_handler","Rust impl signal.rs enter");
    if (signum
        < (size_of::<std::sync::atomic::AtomicU32>() * CHAR_BIT)
            .try_into()
            .unwrap())
        && (SIGNAL_HOOK.load(Relaxed) != 0)
    {
        SIGNALS.fetch_or(1u32 << signum, Relaxed);
        unsafe { sml_set_check_hook(transmute(SIGNAL_HOOK.load(Relaxed))) };
        unsafe { sml_gc(0) };
    }
    tracing::debug!(target:"signal_handler","Rust impl signal.rs exit");
}

fn do_sigaction(signum: i32, signame: &str, sa: *const sigaction) -> i32 {
    tracing::debug!(target:"do_sigaction","Rust impl signal.rs enter");
    let mut old = unsafe { std::mem::zeroed() };
    let mut r = unsafe { sigaction(signum, sa, &raw mut old) };
    if (r == 0) && (old.sa_sigaction != SIG_DFL) {
        tracing::warn!(target:"do_sigaction","{signame}Success handler is already set");
        r = unsafe { sigaction(signum, &raw const old, std::ptr::null_mut()) };
    }
    tracing::debug!(target:"do_sigaction","Rust impl signal.rs exit");
    return r;
}
#[unsafe(no_mangle)]
extern "C" fn sml_signal_sigaction(hook: SmlCheckHookFn) -> c_int {
    tracing::debug!(target:"sml_signal_sigaction","Rust impl signal.rs enter");
    SIGNAL_HOOK.store(unsafe { transmute(hook) }, Relaxed);
    let mut sa: sigaction = unsafe { std::mem::zeroed() };
    sa.sa_sigaction = unsafe { std::mem::transmute(signal_handler as *const usize) };
    sa.sa_flags = 0;
    let mut r = unsafe { sigemptyset(&mut sa.sa_mask) };
    if r != 0 {
        tracing::debug!(target:"sml_signal_sigaction","Rust impl signal.rs exit");
        return r;
    }
    r = do_sigaction(SIGINT, "SIGINT", &sa);
    if r != 0 {
        tracing::debug!(target:"sml_signal_sigaction","Rust impl signal.rs exit");
        return r;
    }
    r = do_sigaction(SIGHUP, "SIGHUP", &sa);
    if r != 0 {
        tracing::debug!(target:"sml_signal_sigaction","Rust impl signal.rs exit");
        return r;
    }
    r = do_sigaction(SIGPIPE, "SIGPIPE", &sa);
    if r != 0 {
        tracing::debug!(target:"sml_signal_sigaction","Rust impl signal.rs exit");
        return r;
    }
    r = do_sigaction(SIGTERM, "SIGTERM", &sa);
    if r != 0 {
        tracing::debug!(target:"sml_signal_sigaction","Rust impl signal.rs exit");
        return r;
    }
    return 0;
}
