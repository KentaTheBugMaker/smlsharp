use std::{
    ffi::{CStr, CString},
    fs::{File, OpenOptions},
    io::stderr,
    process::abort,
    str::FromStr,
    sync::{Mutex, OnceLock},
};

use libc::{FILE, c_char, c_int, fprintf};
use printf_compat::{format, output};
use tracing::Level;
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt};

#[repr(C)]
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
enum SMLMessageLevel {
    Fatal,
    Error,
    Warn,
    Notice,
    Debug,
    Full,
}

#[cfg(feature = "debug")]
const DEFAULT_VERBOSE_LEVEL: SMLMessageLevel = SMLMessageLevel::Debug;
#[cfg(not(feature = "debug"))]
const DEFAULT_VERBOSE_LEVEL: SMLMessageLevel = SMLMessageLevel::Notice;

static MSG_LEVEL: OnceLock<SMLMessageLevel> = OnceLock::new();

unsafe extern "C" fn print_syserror(
    level: SMLMessageLevel,
    err: c_int,
    format: *const c_char,
    mut args: ...
) {
    if *MSG_LEVEL.get().unwrap() < level {
        return;
    }
    let mut s = String::new();
    unsafe { printf_compat::format(format, args.as_va_list(), output::fmt_write(&mut s)) };

    let e = std::io::Error::from_raw_os_error(err);
    let msg = match e.raw_os_error() {
        Some(err) => match err {
            0 => {
                format!("{s}: Success\n")
            }
            x => {
                if err > 0 {
                    format!("{s}{}\n", e.to_string())
                } else {
                    format!("{s}: Failed ({err})\n")
                }
            }
        },
        None => {
            unreachable!("");
        }
    };
    match *MSG_LEVEL.get().unwrap() {
        SMLMessageLevel::Fatal => tracing::error!(msg),
        SMLMessageLevel::Error => tracing::error!(msg),
        SMLMessageLevel::Warn => tracing::warn!(msg),
        SMLMessageLevel::Notice => tracing::info!(msg),
        SMLMessageLevel::Debug => tracing::debug!(msg),
        SMLMessageLevel::Full => tracing::trace!(msg),
    };
}

unsafe extern "C" fn print_error(
    level: SMLMessageLevel,
    err: c_int,
    format: *const c_char,
    mut args: ...
) {
    if *MSG_LEVEL.get().unwrap() < level {
        return;
    }
    if err != 0 {
        unsafe { print_syserror(level, err, format, args) };
        return;
    }
    let mut msg = String::new();
    unsafe { printf_compat::format(format, args.as_va_list(), output::fmt_write(&mut msg)) };
    match *MSG_LEVEL.get().unwrap() {
        SMLMessageLevel::Fatal => tracing::error!(msg),
        SMLMessageLevel::Error => tracing::error!(msg),
        SMLMessageLevel::Warn => tracing::warn!(msg),
        SMLMessageLevel::Notice => tracing::info!(msg),
        SMLMessageLevel::Debug => tracing::debug!(msg),
        SMLMessageLevel::Full => tracing::trace!(msg),
    };
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_fatal(err: c_int, format: *const c_char, mut args: ...) {
    unsafe { print_error(SMLMessageLevel::Fatal, err, format, args) };
    abort();
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_error(err: c_int, format: *const c_char, mut args: ...) {
    unsafe { print_error(SMLMessageLevel::Error, err, format, args) };
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_warn(err: c_int, format: *const c_char, mut args: ...) {
    unsafe { print_error(SMLMessageLevel::Warn, err, format, args) };
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_notice(format: *const c_char, mut args: ...) {
    unsafe { print_error(SMLMessageLevel::Notice, 0, format, args) };
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_sysfatal(format: *const c_char, mut args: ...) {
    if let Some(errno) = std::io::Error::last_os_error().raw_os_error() {
        unsafe { print_syserror(SMLMessageLevel::Fatal, errno, format, args) };
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_syserror(format: *const c_char, mut args: ...) {
    if let Some(errno) = std::io::Error::last_os_error().raw_os_error() {
        unsafe { print_syserror(SMLMessageLevel::Error, errno, format, args) };
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_syswarn(format: *const c_char, mut args: ...) {
    if let Some(errno) = std::io::Error::last_os_error().raw_os_error() {
        unsafe { print_syserror(SMLMessageLevel::Warn, errno, format, args) };
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_debug(format: *const c_char, mut args: ...) {
    if *MSG_LEVEL.get_or_init(|| DEFAULT_VERBOSE_LEVEL) != SMLMessageLevel::Full {
        return;
    }
    let mut s = String::new();
    unsafe { printf_compat::format(format, args.as_va_list(), output::fmt_write(&mut s)) };
    tracing::debug!(target:"sml_debug",s);
}

#[unsafe(no_mangle)]
extern "C" fn sml_msg_init() {
    let _ = match std::env::var("SMLSHARP_VERBOSE") {
        Ok(vl) => {
            let vl: Result<u64, _> = vl.parse();
            if let Ok(vl) = vl {
                MSG_LEVEL.set(match vl {
                    0 => SMLMessageLevel::Fatal,
                    1 => SMLMessageLevel::Error,
                    2 => SMLMessageLevel::Warn,
                    3 => SMLMessageLevel::Notice,
                    4 => SMLMessageLevel::Debug,
                    _ => SMLMessageLevel::Full,
                })
            } else {
                MSG_LEVEL.set(DEFAULT_VERBOSE_LEVEL)
            }
        }
        Err(_) => MSG_LEVEL.set(DEFAULT_VERBOSE_LEVEL),
    };
    match std::env::var("SMLSHARP_LOGFILE") {
        Ok(s) => {
            let debug_file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(s)
                .unwrap();
            tracing_subscriber::fmt().with_writer(debug_file).with_max_level(Level::TRACE).init();
        }
        Err(_) => {
            tracing_subscriber::fmt().with_max_level(Level::TRACE).init();
        }
    };
}
