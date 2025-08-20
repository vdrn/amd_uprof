#![cfg_attr(feature = "nightly", feature(file_with_nul))]

use core::{
    ffi::{CStr, c_void},
    marker::PhantomData,
    mem::ManuallyDrop,
    panic::Location,
    sync::atomic::{AtomicBool, Ordering},
};
use std::{
    collections::HashMap,
    sync::{LazyLock, MutexGuard, RwLock},
};

static ENABLED: AtomicBool = AtomicBool::new(false);
pub fn enable(enabled: bool) {
    ENABLED.store(enabled, Ordering::Release);
}
fn enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}

pub fn resume_profiler() {
    if enabled() {
        unsafe { amd_uprof_sys::amdProfileStrictResumeImpl() };
    }
}
pub fn pause_profiler() {
    if enabled() {
        unsafe { amd_uprof_sys::amdProfileStrictPauseImpl() };
    }
}
pub fn resume_profiler_async() {
    if enabled() {
        unsafe { amd_uprof_sys::amdProfileResumeImpl() };
    }
}
pub fn pause_profiler_async() {
    if enabled() {
        unsafe { amd_uprof_sys::amdProfilePauseImpl() };
    }
}

#[derive(Clone, Copy)]
struct UProfHandle(*mut c_void);
unsafe impl Send for UProfHandle {}
unsafe impl Sync for UProfHandle {}

static EMPTY_CSTR: &CStr = c"";
static DOMAINS: LazyLock<RwLock<HashMap<String, UProfHandle>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
static NAMES: LazyLock<RwLock<HashMap<String, UProfHandle>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub struct TaskScope {
    /// Scopes must be dropped on the same thread that created them.
    /// (Using `MutexGuard` to out out of `Send`)
    _no_send: PhantomData<MutexGuard<'static, ()>>,

    domain: Option<UProfHandle>,
}

#[track_caller]
pub fn scope(domain_str: &str, name_str: &str) -> TaskScope {
    if !enabled() {
        return TaskScope {
            domain: None,
            _no_send: PhantomData,
        };
    }

    let domain = if let Some(domain) = DOMAINS.read().unwrap().get(domain_str) {
        *domain
    } else {
        let domain_cstr = std::ffi::CString::new(domain_str).unwrap();
        let domain_handle =
            UProfHandle(unsafe { amd_uprof_sys::amdDomainCreate(domain_cstr.as_ptr()) });
        DOMAINS
            .write()
            .unwrap()
            .insert(domain_str.to_string(), domain_handle);
        domain_handle
    };

    let name = if let Some(name) = NAMES.read().unwrap().get(name_str) {
        name.0
    } else {
        let name_cstr = std::ffi::CString::new(name_str).unwrap();
        let name_handle =
            UProfHandle(
                unsafe { amd_uprof_sys::amdStringHandleCreate(name_cstr.as_ptr()) } as *mut c_void,
            );
        NAMES
            .write()
            .unwrap()
            .insert(name_str.to_string(), name_handle);
        name_handle.0
    };

    let loc = Location::caller();
    #[cfg(feature = "nightly")]
    let file = loc.file_with_nul().as_ptr();
    #[cfg(not(feature = "nightly"))]
    let file = EMPTY_CSTR.as_ptr();

    unsafe {
        amd_uprof_sys::AMDTaskBegin(
            EMPTY_CSTR.as_ptr(),
            file,
            loc.line() as i32,
            domain.0,
            0,
            0,
            name as *mut _,
        );
    }
    TaskScope {
        domain: Some(domain),
        _no_send: PhantomData,
    }
}

impl TaskScope {
    #[track_caller]
    pub fn finish(self) {
        let self_ = ManuallyDrop::new(self);
        if let Some(domain) = self_.domain {
            let loc = Location::caller();

            #[cfg(feature = "nightly")]
            let file = loc.file_with_nul().as_ptr();
            #[cfg(not(feature = "nightly"))]
            let file = EMPTY_CSTR.as_ptr();

            unsafe {
                amd_uprof_sys::AMDTaskEnd(EMPTY_CSTR.as_ptr(), file, loc.line() as i32, domain.0);
            }
        }
    }
}
impl Drop for TaskScope {
    fn drop(&mut self) {
        if let Some(domain) = self.domain {
            unsafe {
                amd_uprof_sys::AMDTaskEnd(EMPTY_CSTR.as_ptr(), EMPTY_CSTR.as_ptr(), 0, domain.0);
            }
        }
    }
}
