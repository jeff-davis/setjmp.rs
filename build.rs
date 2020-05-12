extern crate bindgen;
extern crate clang_sys;
extern crate libc;

use std::collections::HashMap;
use std::env;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clang_sys::*;

use libc::c_void;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Only generate the types, function declarations will be done
        // separately
        .whitelist_type("jmp_buf")
        .whitelist_type("sigjmp_buf")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("jmpbuf.rs"))
        .expect("Couldn't write bindings!");

    let map = find_symbols("find_symbols.c");
    let mut file = File::create(out_path.join("decls.rs")).unwrap();
    let contents = format!(
        "
extern \"C\" {{
    #[link_name=\"{}\"]
    pub fn setjmp(env: *mut jmp_buf) -> c_int;
    #[link_name=\"{}\"]
    pub fn sigsetjmp(env: *mut sigjmp_buf, savesigs: c_int) -> c_int;
    #[link_name=\"{}\"]
    pub fn longjmp(env: *mut jmp_buf, val: c_int) -> !;
    #[link_name=\"{}\"]
    pub fn siglongjmp(env: *mut sigjmp_buf, val: c_int) -> !;
}}
",
        map["setjmp"], map["sigsetjmp"], map["longjmp"], map["siglongjmp"]
    );
    file.write_all(contents.as_bytes()).unwrap();
}

fn find_symbols(filename: &str) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    let mut vec: Vec<String> = Vec::new();

    unsafe {
        let filename_cstr = CString::new(filename).unwrap();
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

    // C file designed to be visited in this order
    map.insert("setjmp".to_string(), vec[0].clone());
    map.insert("sigsetjmp".to_string(), vec[1].clone());
    map.insert("longjmp".to_string(), vec[2].clone());
    map.insert("siglongjmp".to_string(), vec[3].clone());
    map
}

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
