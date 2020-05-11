//!
//! Expose C standard library functions in <setjmp.h> to rust.
//!
//! **WARNING: this crate is experimental and even careful use is
//! likely undefined behavior.**
//!
//! See
//! [setjmp(3)](https://manpages.debian.org/unstable/manpages-dev/setjmp.3.en.html)
//! and [RFC #2625](https://github.com/rust-lang/rfcs/issues/2625).

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use libc::*;

include!(concat!(env!("OUT_DIR"), "/jmpbuf.rs"));

extern "C" {
    #[link_name="setjmp"]
    pub fn setjmp(env: *mut jmp_buf) -> c_int;
    #[link_name="__sigsetjmp"]
    pub fn sigsetjmp(env: *mut sigjmp_buf, savesigs: c_int) -> c_int;
    pub fn longjmp(env: *mut jmp_buf, val: c_int) -> c_void;
    pub fn siglongjmp(env: *mut sigjmp_buf, val: c_int) -> c_void;
}
