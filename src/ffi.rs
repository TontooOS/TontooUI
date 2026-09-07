//! C FFI exports for TontooUI.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use glib::translate::ToGlibPtr;
use uikit::prelude::*;

unsafe fn read_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok().map(str::to_owned)
}

/// The framework version as a static C string.
#[no_mangle]
pub extern "C" fn tontooui_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

// ── ProgressView ────────────────────────────────────────────────────────

/// Create a spinner-style progress view.
/// Free with `tontoo_tontooui_progress_view_free`.
#[no_mangle]
pub extern "C" fn tontooui_progress_view_new() -> *mut crate::ProgressView {
    Box::into_raw(Box::new(crate::ProgressView::new()))
}

/// The underlying GTK4 widget. Borrowed; do not free.
///
/// # Safety
///
/// `view` must be a handle returned by `tontooui_progress_view_new`.
#[no_mangle]
pub unsafe extern "C" fn tontooui_progress_view_widget(
    view: *mut crate::ProgressView,
) -> *mut gtk::ffi::GtkWidget {
    if view.is_null() {
        return std::ptr::null_mut();
    }
    (*view).to_gtk().to_glib_none().0
}

/// Destroy a progress view handle.
///
/// # Safety
///
/// `view` must be a handle returned by `tontooui_progress_view_new` and
/// must not be used afterwards.
#[no_mangle]
pub unsafe extern "C" fn tontooui_progress_view_free(view: *mut crate::ProgressView) {
    if !view.is_null() {
        drop(Box::from_raw(view));
    }
}

// ── TextInput ───────────────────────────────────────────────────────────

/// Create a text input with a placeholder.
/// Free with `tontoo_tontooui_text_input_free`.
///
/// # Safety
///
/// `placeholder` may be null for an empty placeholder.
#[no_mangle]
pub unsafe extern "C" fn tontoo_tontooui_text_input_new(
    placeholder: *const c_char,
) -> *mut crate::TextInput {
    let placeholder = read_str(placeholder).unwrap_or_default();
    Box::into_raw(Box::new(crate::TextInput::new(placeholder)))
}

/// Read back the current input text. Free with `tontooui_string_free`.
///
/// # Safety
///
/// `input` must be a valid handle.
#[no_mangle]
pub unsafe extern "C" fn tontoo_tontooui_text_input_text(
    input: *mut crate::TextInput,
) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    CString::new((*input).text_value())
        .unwrap_or_default()
        .into_raw()
}

/// The underlying GTK4 widget. Borrowed; do not free.
///
/// # Safety
///
/// `input` must be a handle returned by `tontoo_tontooui_text_input_new`.
#[no_mangle]
pub unsafe extern "C" fn tontooui_text_input_widget(
    input: *mut crate::TextInput,
) -> *mut gtk::ffi::GtkWidget {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    (*input).to_gtk().to_glib_none().0
}

/// Destroy a text input handle.
///
/// # Safety
///
/// `input` must be a handle returned by `tontoo_tontooui_text_input_new`
/// and must not be used afterwards.
#[no_mangle]
pub unsafe extern "C" fn tontoo_tontooui_text_input_free(input: *mut crate::TextInput) {
    if !input.is_null() {
        drop(Box::from_raw(input));
    }
}

/// Free a string returned by this library.
///
/// # Safety
///
/// `s` must be a pointer returned by this API or null.
#[no_mangle]
pub unsafe extern "C" fn tontooui_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}
