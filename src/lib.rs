//!
//! Expose C standard library functions in ``<setjmp.h>`` to rust.
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

mod sys {
    use libc::*;

    include!(concat!(env!("OUT_DIR"), "/jmpbuf.rs"));
    include!(concat!(env!("OUT_DIR"), "/decls.rs"));
}

pub use crate::sys::{jmp_buf, longjmp, setjmp, sigjmp_buf, siglongjmp, sigsetjmp};
