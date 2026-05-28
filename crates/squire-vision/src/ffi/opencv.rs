#![allow(non_camel_case_types, non_snake_case)]

use std::ffi::{c_char, c_double, c_int, c_void};

pub const CV_8UC1: c_int = 0;
pub const CV_8UC3: c_int = 16;
pub const CV_8UC4: c_int = 24;
pub const CV_32FC1: c_int = 5;
pub const CV_TM_CCOEFF_NORMED: c_int = 5;
pub const IMREAD_COLOR: c_int = 1;
pub const COLOR_BGRA2GRAY: c_int = 10;
pub const COLOR_BGR2GRAY: c_int = 6;

#[repr(C)]
pub struct CvMat {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct CvPoint {
    pub x: c_int,
    pub y: c_int,
}

extern "C" {
    pub fn cvCreateMatHeader(rows: c_int, cols: c_int, mat_type: c_int) -> *mut CvMat;
    pub fn cvSetData(arr: *mut CvMat, data: *mut c_void, step: c_int);
    pub fn cvCreateMat(rows: c_int, cols: c_int, mat_type: c_int) -> *mut CvMat;
    pub fn cvReleaseMat(mat: *mut *mut CvMat);

    pub fn cvLoadImage(filename: *const c_char, iscolor: c_int) -> *mut CvMat;
    pub fn cvCvtColor(src: *const CvMat, dst: *mut CvMat, code: c_int);
    pub fn cvMatchTemplate(image: *const CvMat, templ: *const CvMat, result: *mut CvMat, method: c_int);
    pub fn cvMinMaxLoc(
        arr: *const CvMat,
        min_val: *mut c_double,
        max_val: *mut c_double,
        min_loc: *mut CvPoint,
        max_loc: *mut CvPoint,
        mask: *const CvMat,
    );

    pub fn cvGetSize(arr: *const CvMat) -> CvSize;
    pub fn cvGet2D(arr: *const CvMat, idx0: c_int, idx1: c_int) -> CvScalar;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CvSize {
    pub width: c_int,
    pub height: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CvScalar {
    pub val: [c_double; 4],
}

pub struct OwnedCvMat {
    ptr: *mut CvMat,
}

impl OwnedCvMat {
    pub unsafe fn from_raw(ptr: *mut CvMat) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    pub fn as_ptr(&self) -> *const CvMat {
        self.ptr
    }

    pub fn as_mut_ptr(&mut self) -> *mut CvMat {
        self.ptr
    }
}

impl Drop for OwnedCvMat {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { cvReleaseMat(&mut self.ptr) };
        }
    }
}

pub struct HeaderCvMat {
    ptr: *mut CvMat,
}

impl HeaderCvMat {
    pub unsafe fn new(rows: c_int, cols: c_int, mat_type: c_int, data: *mut c_void, step: c_int) -> Option<Self> {
        let ptr = cvCreateMatHeader(rows, cols, mat_type);
        if ptr.is_null() {
            return None;
        }
        cvSetData(ptr, data, step);
        Some(Self { ptr })
    }

    pub fn as_ptr(&self) -> *const CvMat {
        self.ptr
    }
}

impl Drop for HeaderCvMat {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { cvReleaseMat(&mut self.ptr) };
        }
    }
}
