#![allow(non_camel_case_types, non_snake_case)]

use std::ffi::c_int;

pub const CV_TM_CCOEFF_NORMED: c_int = 5;

extern "C" {
    pub fn squire_match_template_masked(
        img_data: *const u8, img_w: c_int, img_h: c_int, img_channels: c_int,
        tmpl_data: *const u8, tmpl_w: c_int, tmpl_h: c_int, tmpl_channels: c_int,
        mask_data: *const u8,
        method: c_int,
        out_max_val: *mut f64, out_max_x: *mut c_int, out_max_y: *mut c_int,
    ) -> c_int;
}
