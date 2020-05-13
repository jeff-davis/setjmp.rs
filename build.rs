extern crate bindgen;
extern crate clang_sys;
extern crate libc;

use std::env;
use std::ffi::{CStr, CString};
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;

use clang_sys::*;

use libc::c_void;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    // Generate rust definitions for the jmp_buf and sigjmp_buf
    // types. Do NOT generate bindings for the function declarations;
    // that's done below.
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // we only want these two type definitions
        .whitelist_type("jmp_buf")
        .whitelist_type("sigjmp_buf")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/jmpbuf.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("jmpbuf.rs"))
        .expect("Couldn't write bindings!");

    // We link against the platform's libc, but to do so we need the
    // proper symbol names for setjmp, sigsetjmp, etc. Sometimes,
    // these names are actually a macro that's rewritten to the real
    // symbol name, e.g. __sigsetjmp.
    //
    // To find the real symbol names, we create temporary C file that
    // refers to the names, then use libclang to parse it to get the
    // actual libc symbols after macro expansion.
    let c_file_path = out_path.join("find_symbols.c");
    let mut c_file = File::create(&c_file_path).unwrap();
    c_file.write_all(&c_contents()).unwrap();
    let symbols = find_symbols(&c_file_path);
    remove_file(&c_file_path).unwrap();

    // write out a rust file containing declarations including link
    // names that point to the correct libc symbols found above
    let mut file = File::create(out_path.join("decls.rs")).unwrap();
    file.write_all(&decls_contents(symbols)).unwrap();
}

// Parse the given C file with libclang, extracting the symbols from
// the four call sites.
fn find_symbols(filename: &PathBuf) -> Vec<String> {
    let mut vec: Vec<String> = Vec::new();

    unsafe {
        let filename_cstr = CString::new(filename.to_str().unwrap()).unwrap();
        let index = clang_createIndex(0, 0);
        let tu = clang_parseTranslationUnit(
            index,
            filename_cstr.as_ptr(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            0,
            CXTranslationUnit_None,
        );
        let cursor = clang_getTranslationUnitCursor(tu);
        clang_visitChildren(cursor, visitor, &mut vec as *mut Vec<String> as *mut c_void);
    }

    assert!(vec.len() == 4);
    vec
}

// Walk the AST, extracting the CallExprs and pushing them into the
// given vector.
extern "C" fn visitor(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    unsafe {
        let symbols: &mut Vec<String> = &mut *(client_data as *mut Vec<String>);
        let cursor_name = clang_getCursorSpelling(cursor);
        let cursor_cstr = CStr::from_ptr(clang_getCString(cursor_name));
        let cursor_str = cursor_cstr.to_str().unwrap();

        let cursorkind: CXCursorKind = clang_getCursorKind(cursor);

        if cursorkind == CXCursor_CallExpr {
            symbols.push(cursor_str.to_string());
        }
    }
    CXChildVisit_Recurse
}

// The contents of the temporary C file that we need to create. This
// will be parsed with libclang to find the real symbol names for
// setjmp (etc.) after macro expansion.
//
// The order of calls in this function must match the order of
// declarations in decls_contents().
fn c_contents() -> Vec<u8> {
    return r###"
#include <setjmp.h>

int find_symbols()
{
    jmp_buf jmpbuf;
    sigjmp_buf sigjmpbuf;
    setjmp(jmpbuf);
    sigsetjmp(sigjmpbuf, 0);
    longjmp(jmpbuf, 1);
    siglongjmp(sigjmpbuf, 1);
}
"###
    .as_bytes()
    .to_vec();
}

// The contents of the function declarations for setjmp, etc. The
// order of declarations must match the order of calls in
// c_contents().
fn decls_contents(symbols: Vec<String>) -> Vec<u8> {
    return format!(
        r###"
extern "C" {{
    #[link_name="{}"]
    pub fn setjmp(env: *mut jmp_buf) -> c_int;
    #[link_name="{}"]
    pub fn sigsetjmp(env: *mut sigjmp_buf, savesigs: c_int) -> c_int;
    #[link_name="{}"]
    pub fn longjmp(env: *mut jmp_buf, val: c_int) -> !;
    #[link_name="{}"]
    pub fn siglongjmp(env: *mut sigjmp_buf, val: c_int) -> !;
}}
"###,
        symbols[0], symbols[1], symbols[2], symbols[3]
    )
    .as_bytes()
    .to_vec();
}
