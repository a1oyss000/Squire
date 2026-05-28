#![allow(non_camel_case_types, non_snake_case)]

use std::ffi::{c_char, c_float, c_int, c_uchar, c_void};

pub const RIL_WORD: c_int = 3;

#[repr(C)]
pub struct TessBaseAPI {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct TessResultIterator {
    _opaque: [u8; 0],
}

extern "C" {
    pub fn TessBaseAPICreate() -> *mut TessBaseAPI;
    pub fn TessBaseAPIDelete(handle: *mut TessBaseAPI);
    pub fn TessBaseAPIInit3(
        handle: *mut TessBaseAPI,
        datapath: *const c_char,
        language: *const c_char,
    ) -> c_int;
    pub fn TessBaseAPISetImage(
        handle: *mut TessBaseAPI,
        imagedata: *const c_uchar,
        width: c_int,
        height: c_int,
        bytes_per_pixel: c_int,
        bytes_per_line: c_int,
    );
    pub fn TessBaseAPISetSourceResolution(handle: *mut TessBaseAPI, ppi: c_int);
    pub fn TessBaseAPIRecognize(handle: *mut TessBaseAPI, monitor: *mut c_void) -> c_int;
    pub fn TessBaseAPIGetUTF8Text(handle: *mut TessBaseAPI) -> *mut c_char;

    pub fn TessBaseAPIGetIterator(handle: *mut TessBaseAPI) -> *mut TessResultIterator;
    pub fn TessResultIteratorNext(
        handle: *mut TessResultIterator,
        level: c_int,
    ) -> c_int;
    pub fn TessResultIteratorGetUTF8Text(
        handle: *mut TessResultIterator,
        level: c_int,
    ) -> *mut c_char;
    pub fn TessResultIteratorBoundingBox(
        handle: *mut TessResultIterator,
        level: c_int,
        left: *mut c_int,
        top: *mut c_int,
        right: *mut c_int,
        bottom: *mut c_int,
    ) -> c_int;
    pub fn TessResultIteratorConfidence(
        handle: *mut TessResultIterator,
        level: c_int,
    ) -> c_float;
    pub fn TessResultIteratorDelete(handle: *mut TessResultIterator);
    pub fn TessDeleteText(text: *mut c_char);
}

pub struct OcrHandle {
    ptr: *mut TessBaseAPI,
}

impl OcrHandle {
    pub unsafe fn new(datapath: *const c_char, language: *const c_char) -> Option<Self> {
        let ptr = TessBaseAPICreate();
        if ptr.is_null() {
            return None;
        }
        let rc = TessBaseAPIInit3(ptr, datapath, language);
        if rc != 0 {
            TessBaseAPIDelete(ptr);
            return None;
        }
        Some(Self { ptr })
    }

    pub fn as_ptr(&self) -> *mut TessBaseAPI {
        self.ptr
    }
}

impl Drop for OcrHandle {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { TessBaseAPIDelete(self.ptr) };
        }
    }
}

unsafe impl Send for OcrHandle {}
