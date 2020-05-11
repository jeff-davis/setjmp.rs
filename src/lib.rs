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
