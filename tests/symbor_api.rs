extern crate dynlib;
#[macro_use]
extern crate dynlib_derive;
extern crate libc;
#[macro_use]
extern crate const_cstr;
use dynlib::symbor::{Library, SymBorApi, Symbol, RefMut, Ref, PtrOrNull};
use libc::{c_int, c_char};
use std::ffi::CStr;

use std::io::Write;
mod commons;
use commons::{example_lib_path, SomeData};

macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

#[derive(SymBorApi)]
struct Api<'a> {
    pub rust_fun_print_something: Symbol<'a, fn()>,
    pub rust_fun_add_one: Symbol<'a, fn(i32) -> i32>,
    pub c_fun_print_something_else: Symbol<'a, unsafe extern "C" fn()>,
    pub c_fun_add_two: Symbol<'a, unsafe extern "C" fn(c_int) -> c_int>,
    pub rust_i32: Ref<'a, i32>,
    pub rust_i32_mut: RefMut<'a, i32>,
    #[dynlib_name="rust_i32_mut"]
    pub rust_i32_ptr: Symbol<'a, * const i32>,
    pub c_int: Ref<'a, c_int>,
    pub c_struct: Ref<'a, SomeData>,
    pub rust_str: Ref<'a, &'static str>,
    pub c_const_char_ptr: PtrOrNull<'a, c_char>
}

//#[cfg(not(any(target_os="macos", target_os="ios")))]
#[test]
fn open_play_close_symbor_api(){
    let lib_path = example_lib_path();
    let lib = Library::open(lib_path).expect("Could not open library");
    let mut api = unsafe{Api::load(&lib)}.expect("Could not load symbols");
    (api.rust_fun_print_something)(); //should not crash
    assert_eq!((api.rust_fun_add_one)(5), 6);
    unsafe{ (api.c_fun_print_something_else)()}; //should not crash
    println_stderr!("something else call OK");
    assert_eq!(unsafe{(api.c_fun_add_two)(2)}, 4);
    println_stderr!("add_two called OK");
    assert_eq!(43, *api.rust_i32);
    println_stderr!("obtaining const data OK");
    assert_eq!(42, *api.rust_i32_mut);
    println_stderr!("obtaining mutable data OK");
    *api.rust_i32_mut = 55; //should not crash
    println_stderr!("assigning mutable data OK");
    assert_eq!(55, unsafe{**api.rust_i32_ptr});
    println_stderr!("obtaining pointer OK");
    //the same with C
    assert_eq!(45, *api.c_int);
    println_stderr!("obtaining C data OK");
    //now static c struct

    assert_eq!(1, api.c_struct.first);
    assert_eq!(2, api.c_struct.second);
    println_stderr!("obtaining C structure OK");
    //let's play with strings

    assert_eq!("Hello!", *api.rust_str);
    println_stderr!("obtaining str OK");
    let converted = unsafe{CStr::from_ptr(*api.c_const_char_ptr)}.to_str().unwrap();
    assert_eq!(converted, "Hi!");
    println_stderr!("obtaining C string OK");

    //It turns out that there is a bug in rust.
    //On OSX calls to dynamic libraries written in Rust causes segmentation fault
    //please note that this ia a problem with the example library, not this library
    //maybe converting the example library into cdylib would help?
    //https://github.com/rust-lang/rust/issues/28794
    //#[cfg(any(target_os="macos", target_os="ios"))]
    //::std::mem::forget(lib);
}
