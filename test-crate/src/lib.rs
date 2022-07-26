#[cfg(test)]
mod tests {
    use libz_sys::zlibVersion;
    use std::ffi::CStr;

    #[test]
    fn zlib_version() {
        let ver = unsafe { zlibVersion() };
        let ver_cstr = unsafe { CStr::from_ptr(ver) };
        let version = ver_cstr.to_str().unwrap();
        assert!(!version.is_empty());
    }
}
