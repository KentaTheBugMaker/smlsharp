use std::{
    cmp::Ordering, mem::transmute, os::raw::c_void, process::abort, sync::atomic::AtomicPtr,
};

use libc::{c_char, c_uint, intptr_t, size_t, uint16_t, uintptr_t};

#[repr(C)]
struct SMLFrameLayout {
    num_safe_points: uint16_t,
    frame_size: uint16_t,
    num_roots: uint16_t,
    root_offsets: [uint16_t; 0],
}
#[repr(C)]
struct ToplevelObjects {
    num_objects: libc::uintptr_t,
    object_offsets: [intptr_t; 0],
}
#[derive(Clone, Copy)]
#[repr(C)]
struct SafePoint {
    /* addr is either a struct toplevel_objects (if layout is null)
     * or address of a call instruction (otherwise). */
    addr: *const c_void,
    layout: *const SMLFrameLayout,
}

const PAGE_SIZE: usize = 4096;
const GCROOT_ITEM_MAXLEN: usize = (PAGE_SIZE - size_of::<SMLGCRoot>()) / (size_of::<SafePoint>());
const PAD_SIZE: usize = PAGE_SIZE
    - size_of::<Option<Box<SMLGCRoot>>>()
    - size_of::<size_t>()
    - size_of::<size_t>()
    - size_of::<size_t>()
    - GCROOT_ITEM_MAXLEN * size_of::<SafePoint>();
#[repr(C)]
struct SMLGCRoot {
    next: AtomicPtr<SMLGCRoot>,
    num_points: size_t,
    num_tops: size_t,
    allocsize: size_t,
    item: [SafePoint; 0],
    /* item[0 ... num_points] is for safe_points sorted by addr
     * item[num_layouts ... (num_layouts + num_tops)] is for
     * toplevel_objects */
}

fn new_gcroot() -> *mut SMLGCRoot {
    tracing::info!("new_gcroot enter");
    //SMLGCRootはSMLSharpプログラム側がいじるので元のC言語の実装と同一にする
    //アロケーション回数を減らすためにある程度余計に確保しておく
    if let Ok(gcroot_layout) = std::alloc::Layout::from_size_align(PAGE_SIZE, PAGE_SIZE) {
        let gcroot = unsafe { std::alloc::alloc(gcroot_layout) };
        if gcroot.is_null() {
            tracing::error!("Allocation Error");
            std::alloc::handle_alloc_error(gcroot_layout);
        }

        let gcroot = gcroot as *mut SMLGCRoot;
        unsafe {
            gcroot.write(SMLGCRoot {
                next: AtomicPtr::new(std::ptr::null_mut()),
                num_points: 0,
                num_tops: 0,
                allocsize: PAGE_SIZE,
                item: [],
            });
        };

        tracing::info!("new_gcroot exit");
        gcroot
    } else {
        tracing::error!("Failed to Allocate new GCRoot");
        panic!();
    }
}

fn rest_safe_points(allocsize: usize, num_points: usize, num_tops: usize) -> usize {
    //割り当てた領域から 管理領域32バイト を引いてSafePointの大きさで割ると
    //現在のSafePointの容量となる
    //ここから今使っている pointとtopの分を引いたのが残っている容量である
    (allocsize - size_of::<SMLGCRoot>()) / size_of::<SafePoint>() - (num_points + num_tops)
}
/// 内部でreallocするのでSMLGCRootのポインタが変わる
/// そのため2重ポインタとする
fn alloc_safe_points(gcroot: &mut Option<&mut SMLGCRoot>, inc: size_t) -> *mut SafePoint {
    tracing::info!("alloc_safe_points enter");
    if let Some(p) = gcroot {
        let num_points = p.num_points;
        let num_tops = p.num_tops;
        let allocsize = p.allocsize;
        if rest_safe_points(allocsize, num_points, num_tops) < inc {
            tracing::info!("alloc_safe_point extend start");
            //スロットはinc分は必要
            let minsize = num_points + num_tops + inc;
            //今までのgcrootにある分と合わせた最低限必要な領域
            let minsize = minsize * size_of::<SafePoint>();
            //管理領域も合わせた最低限必要な領域
            let minsize = size_of::<SMLGCRoot>() + minsize;
            //
            let allocsize = minsize.checked_next_multiple_of(PAGE_SIZE).unwrap();
            // アロケータに渡すためのレイアウトの復元
            if let Ok(layout) = std::alloc::Layout::from_size_align(p.allocsize, PAGE_SIZE) {
                tracing::info!("alloc_safe_point layout Ok");
                //拡張
                p.num_points += inc;
                let raw_ptr = (*p as *mut SMLGCRoot).addr();

                let p = unsafe { std::alloc::realloc(transmute(raw_ptr), layout, allocsize) };
                let view = unsafe { p.add(std::mem::size_of::<SMLGCRoot>()) as *mut SafePoint };
                let p: &mut SMLGCRoot = unsafe { transmute(p) };
                //拡張した分を反映する
                p.allocsize = allocsize;

                let view = unsafe { view.add(num_points + num_tops) };
                *gcroot = Some(p);

                tracing::info!("alloc_safe_point exit");
                view
            } else {
                tracing::error!("Failed to Extend capacity of SMLGCRoot");
                panic!();
            }
        } else {
            p.num_points += inc;
            let ret= unsafe {
                let ptr:*mut SafePoint = std::mem::transmute(p);
                ptr.byte_add(size_of::<SMLGCRoot>())
                .add(num_points+num_tops)
            };

            tracing::info!("alloc_safe_point exit");
            ret
        }
    } else {
        tracing::warn!("gcroot is null");
        std::ptr::null_mut()
    }
}
fn layout_size(layout: &SMLFrameLayout) -> usize {
    size_of::<SMLFrameLayout>() + size_of::<u16>() * (layout.num_roots as usize)
}
fn safe_point_offset(layout: &SMLFrameLayout) -> usize {
    layout_size(layout)
        .checked_next_multiple_of(size_of::<intptr_t>())
        .unwrap()
}
fn safe_points(layout: &SMLFrameLayout) -> *const intptr_t {
    (unsafe {
        std::mem::transmute::<*const SMLFrameLayout, *const c_char>(layout)
            .add(safe_point_offset(layout))
    }) as *const intptr_t
}
fn next_layout(layout: &SMLFrameLayout) -> *const intptr_t {
    unsafe { safe_points(layout).add(layout.num_safe_points as usize) }
}

#[unsafe(no_mangle)]
extern "C" fn sml_gcroot(
    gcroot_p: *mut *mut SMLGCRoot,
    sml_tabb: extern "C" fn() -> (),
    smlftab: Option<&SMLFrameLayout>,
    sml_root: Option<&ToplevelObjects>,
) {
    tracing::info!("sml_gcroot enter ");
    let gcroot: &mut Option<&mut SMLGCRoot> = unsafe { transmute(gcroot_p) };
    let mut points: *const intptr_t;
    let mut inc = 0;
    let i = 0;
    if let Some(ftab) = smlftab {
        let mut t = ftab;
        loop {
            if t.num_safe_points > 0 {
                inc += t.num_safe_points;
                t = unsafe { std::mem::transmute(next_layout(t)) };
            } else {
                break;
            }
        }
    }
    if sml_root.is_some() {
        inc += 1;
    }
    let mut dst = alloc_safe_points(gcroot, inc as usize);
    if let Some(ftab) = smlftab {
        let mut t = ftab;
        loop {
            if t.num_safe_points > 0 {
                points = safe_points(t);
                for i in 0..t.num_safe_points {
                    if let Some(dst) = unsafe { dst.as_mut() } {
                        dst.addr = unsafe {
                            transmute(
                                transmute::<extern "C" fn() -> (), *const c_char>(sml_tabb)
                                    .offset(unsafe { points.add(i as usize).read() }),
                            )
                        };
                        dst.layout = t;
                    }
                    dst = unsafe { dst.add(1) };
                }
                t = unsafe { std::mem::transmute(next_layout(t)) };
            } else {
                break;
            }
        }
    }
    if let Some(tops) = sml_root {
        if let Some(dst) = unsafe { dst.as_mut() } {
            dst.addr = unsafe { transmute(tops) };
            dst.layout = std::ptr::null();
        }
        dst = unsafe { dst.add(1) };
        if let Some(gcroot) = gcroot {
            gcroot.num_tops += 1;
            gcroot.num_points -= 1;
        }
    }
    tracing::info!("sml_gcroot exit");
}

fn cmp_safe_point(s1: &SafePoint, s2: &SafePoint) -> Ordering {
    tracing::info!("cmp_safe_point enter");
    let res = if s1.layout.is_null() && !s2.layout.is_null() {
        std::cmp::Ordering::Less
    } else if s1.layout.is_null() && !s2.layout.is_null() {
        std::cmp::Ordering::Greater
    } else {
        let n1 = s1.addr;
        let n2 = s2.addr;
        if n1 > n2 {
            std::cmp::Ordering::Greater
        } else if n1 < n2 {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    };
    tracing::info!("cmp_safe_point exit");
    res
}

fn sort_safe_points(gcroot: &mut SMLGCRoot) {
    tracing::info!("sort_safe_point enter");
    let ptr = gcroot.item.as_mut_ptr();
    let view = unsafe { std::slice::from_raw_parts_mut(ptr, gcroot.num_points + gcroot.num_tops) };
    view.sort_by(cmp_safe_point);

    tracing::info!("sort_safe_point exit");
}
static GCROOT_LIST: AtomicPtr<SMLGCRoot> = AtomicPtr::new(std::ptr::null_mut());

fn register_gcroot(gcroot: &mut SMLGCRoot) {
    tracing::info!("register_gcroot enter");
    let first = GCROOT_LIST.load(std::sync::atomic::Ordering::Relaxed);
    if first.is_null()
        && GCROOT_LIST
            .compare_exchange(
                first,
                gcroot,
                std::sync::atomic::Ordering::Release,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_ok()
    {
        return;
    };
    loop {
        if let Some(first) = unsafe { first.as_mut() } {
            let old = first.next.load(std::sync::atomic::Ordering::Relaxed);
            gcroot.next.store(old, std::sync::atomic::Ordering::Relaxed);
            match first.next.compare_exchange_weak(
                old,
                gcroot,
                std::sync::atomic::Ordering::Release,
                std::sync::atomic::Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(_) => {}
            }
        }
    }

    tracing::info!("register_gcroot exit");
}

#[unsafe(no_mangle)]
extern "C" fn sml_gcroot_load(
    sml_loads: *const extern "C" fn(*mut SMLGCRoot),
    count: c_uint,
) -> *mut SMLGCRoot {
    tracing::info!("sml_gcroot_load enter");
    let mut gcroot = new_gcroot();
    if gcroot.is_null(){
        tracing::error!("gcroot returned null");
    }
    let count = count as usize;
    for i in 0..count {
        if let Some(ldr) = unsafe { sml_loads.add(i).as_ref() } {
            ldr(gcroot);
        }
    }
    if let Some(gcroot) = unsafe { gcroot.as_mut() } {
        sort_safe_points(gcroot);
        register_gcroot(gcroot);
    }else{
        tracing::warn!("new");
    }
    tracing::info!("sml_gcroot_load exit");
    return gcroot;
}

#[unsafe(no_mangle)]
extern "C" fn sml_gcroot_unload(gcroot: &mut SMLGCRoot) {
    tracing::info!("sml_gcroot_unload enter");
    gcroot.num_points = 0;
    gcroot.num_tops = 0;
    tracing::info!("sml_gcroot_unload exit");
}

fn binary_search(keyaddr: *const c_void, b: &[SafePoint]) -> Option<&SafePoint> {
    if let Ok(x) = b.binary_search_by(|s| {
        if s.addr == keyaddr {
            Ordering::Equal
        } else if s.addr < keyaddr {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }) {
        b.get(x)
    } else {
        None
    }
}

fn view(x: &SMLGCRoot) -> &[SafePoint] {
    let len = x.num_points + x.num_tops;
    let addr: *const SafePoint = unsafe { transmute(x) };
    unsafe {
        let start = addr.byte_add(size_of::<SMLGCRoot>());
        std::slice::from_raw_parts(start, len)
    }
}

fn view_mut(x: &mut SMLGCRoot) -> &mut [SafePoint] {
    let len = x.num_points + x.num_tops;
    let addr: *mut SafePoint = unsafe { transmute(x) };
    unsafe {
        let start = addr.byte_add(size_of::<SMLGCRoot>());
        std::slice::from_raw_parts_mut(start, len)
    }
}

#[unsafe(no_mangle)]
extern "C" fn sml_lookup_frametable(retaddr: *mut c_void) -> Option<&'static SMLFrameLayout> {
    tracing::info!("sml_lookup_frametable enter");
    let mut p_raw = GCROOT_LIST.load(std::sync::atomic::Ordering::Acquire);
    loop {
        let p = unsafe { p_raw.as_ref() };
        if let Some(p) = p {
            let view = &view(p)[0..];
            let s = binary_search(retaddr, view);
            if let Some(s) = s {
                tracing::info!("sml_lookup_frametable exit");
                return unsafe { s.layout.as_ref() };
            }
            p_raw = p.next.load(std::sync::atomic::Ordering::Acquire);
        } else {
            break;
        }
    }
    tracing::error!("frametable lookup failed {retaddr:p}");
    abort();
}

#[unsafe(no_mangle)]
extern "C" fn sml_global_enum_ptr(
    trace: extern "C" fn(*mut *mut c_void, *mut c_void),
    data: *mut c_void,
) {
    tracing::info!("sml_global_enum_ptr enter");
    let mut p_raw = GCROOT_LIST.load(std::sync::atomic::Ordering::Acquire);
    loop {
        let p = unsafe { p_raw.as_mut() };
        if let Some(p) = p {
            let items = &view(p)[p.num_points..];
            for i in 0..p.num_tops {
                assert!(items[i].layout.is_null());
                let tops_raw: *const ToplevelObjects = unsafe { transmute(items[i].addr) };
                let tops = unsafe { tops_raw.as_ref() };
                if let Some(tops) = tops {
                    for j in 0..tops.num_objects {
                        let offsets = unsafe {
                            tops_raw.byte_add(size_of::<ToplevelObjects>()) as *const uintptr_t
                        };
                        let view = unsafe { std::slice::from_raw_parts(offsets, tops.num_objects) };
                        let obj = unsafe { tops_raw.byte_add(view[j]) } as *mut c_void;
                        unsafe { crate::sml_obj_enum_ptr(obj, trace, data) };
                    }
                }
            }
            p_raw = p.next.load(std::sync::atomic::Ordering::Acquire);
        } else {
            break;
        }
    }
    tracing::info!("sml_global_enum_ptr exit");
}
