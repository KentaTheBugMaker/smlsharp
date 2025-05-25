use std::{fs::OpenOptions, process::abort, sync::OnceLock};

use libc::{c_char, c_int};
use printf_compat::output;
use tracing::Level;

unsafe extern "C" fn print_syserror(
    level: Level,
    err: c_int,
    format: *const c_char,
    mut args: ...
) {
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
            tracing::error!("Rust Runtime raw_os_error returned none");
            unreachable!("");
        }
    };
    match level {
        Level::ERROR => tracing::error!(msg),
        Level::WARN => tracing::warn!(msg),
        Level::INFO => tracing::info!(msg),
        Level::DEBUG => tracing::debug!(msg),
        Level::TRACE => tracing::trace!(msg),
    }
}

unsafe extern "C" fn print_error(level: Level, err: c_int, format: *const c_char, mut args: ...) {
    if err != 0 {
        unsafe { print_syserror(level, err, format, args) };
        return;
    }
    let mut msg = String::new();
    unsafe { printf_compat::format(format, args.as_va_list(), output::fmt_write(&mut msg)) };
    match level {
        Level::ERROR => tracing::error!(msg),
        Level::WARN => tracing::warn!(msg),
        Level::INFO => tracing::info!(msg),
        Level::DEBUG => tracing::debug!(msg),
        Level::TRACE => tracing::trace!(msg),
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_fatal(err: c_int, format: *const c_char, args: ...) {
    unsafe { print_error(Level::ERROR, err, format, args) };
    abort();
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_error(err: c_int, format: *const c_char, args: ...) {
    unsafe { print_error(Level::ERROR, err, format, args) };
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_warn(err: c_int, format: *const c_char, args: ...) {
    unsafe { print_error(Level::WARN, err, format, args) };
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_notice(format: *const c_char, args: ...) {
    unsafe { print_error(Level::INFO, 0, format, args) };
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_sysfatal(format: *const c_char, args: ...) {
    if let Some(errno) = std::io::Error::last_os_error().raw_os_error() {
        unsafe { print_syserror(Level::ERROR, errno, format, args) };
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_syserror(format: *const c_char, args: ...) {
    if let Some(errno) = std::io::Error::last_os_error().raw_os_error() {
        unsafe { print_syserror(Level::ERROR, errno, format, args) };
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_syswarn(format: *const c_char, args: ...) {
    if let Some(errno) = std::io::Error::last_os_error().raw_os_error() {
        unsafe { print_syserror(Level::WARN, errno, format, args) };
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn sml_debug(format: *const c_char, mut args: ...) {

    let mut s = String::new();
    unsafe { printf_compat::format(format, args.as_va_list(), output::fmt_write(&mut s)) };
    tracing::debug!(target:"sml_debug",s);
}

#[unsafe(no_mangle)]
extern "C" fn sml_msg_init() {
    let l = match std::env::var("SMLSHARP_VERBOSE") {
        Ok(vl) => {
            let vl: Result<u64, _> = vl.parse();
            if let Ok(vl) = vl {
                match vl {
                    0 => Level::ERROR,
                    1 => Level::WARN,
                    2 => Level::DEBUG,
                    3 => Level::INFO,
                    4 => Level::TRACE,
                    _ => Level::DEBUG,
                }
            } else {
                Level::DEBUG
            }
        }
        Err(_) => Level::DEBUG,
    };
    match std::env::var("SMLSHARP_LOGFILE") {
        Ok(s) => {
            if let Ok(debug_file) = OpenOptions::new().append(true).create(true).open(s) {
                tracing_subscriber::fmt()
                    .with_writer(debug_file)
                    .with_max_level(l)
                    .init();
            } else {
                tracing_subscriber::fmt().with_max_level(l).init();
            }
        }
        Err(_) => {
            tracing_subscriber::fmt().with_max_level(l).init();
        }
    };
    tracing::info!("logging have been initialized.")
}
