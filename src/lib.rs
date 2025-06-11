#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::{
    fmt::Display,
    marker::PhantomData,
    sync::atomic::{AtomicBool, Ordering},
};

pub mod sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
fn resume_profiler() {
    // unsafe { sys::amdProfileResumeImpl() };
    unsafe { sys::amdProfileStrictResumeImpl() };
}
fn pause_profiler() {
    // unsafe { sys::amdProfilePauseImpl() };
    unsafe { sys::amdProfileStrictPauseImpl() };
}

pub enum UProfMode {
    Global,
    Scopes,
}
pub struct UProf {
    state: UProfMode,
    enabled: bool,
}

#[derive(Debug)]
pub struct ErrorAlreadyCreated;

impl Display for ErrorAlreadyCreated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Already created")
    }
}
impl UProf {
    pub fn new(state: UProfMode, enabled: bool) -> Result<Self, ErrorAlreadyCreated> {
        static CREATED: AtomicBool = AtomicBool::new(false);
        if CREATED.swap(true, Ordering::Relaxed) {
            return Err(ErrorAlreadyCreated);
        }

        if let UProfMode::Global = state {
            if enabled {
                resume_profiler()
            }
        }
        Ok(Self { state, enabled })
    }
    pub fn toggle_enabled(&mut self) {
        match self.state {
            UProfMode::Global => {
                if self.enabled {
                    pause_profiler();
                } else {
                    resume_profiler();
                }
            }
            UProfMode::Scopes => {}
        }
        self.enabled = !self.enabled;
    }

    pub fn profile_scope(&mut self) -> Option<UprofSamplingScope> {
        if let UProfMode::Scopes = self.state {
            if self.enabled {
                resume_profiler();
                return Some(UprofSamplingScope {
                    prevent_construct: PhantomData,
                });
            }
        }
        None
    }
}

pub struct UprofSamplingScope {
    prevent_construct: PhantomData<()>,
}
impl Drop for UprofSamplingScope {
    fn drop(&mut self) {
        pause_profiler();
    }
}
