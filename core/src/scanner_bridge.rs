use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[repr(C)]
pub struct ScannerHandle {
    _private: [u8; 0],
}

extern "C" {
    fn scanner_open_process(process_name: *const c_char) -> *mut ScannerHandle;
    fn scanner_close_process(handle: *mut ScannerHandle);
    fn scanner_read_int(handle: *mut ScannerHandle, address: usize, out: *mut i32) -> bool;
    fn scanner_read_uint(handle: *mut ScannerHandle, address: usize, out: *mut u32) -> bool;
    fn scanner_read_float(handle: *mut ScannerHandle, address: usize, out: *mut f32) -> bool;
    fn scanner_read_string(
        handle: *mut ScannerHandle,
        address: usize,
        buffer: *mut c_char,
        size: i32,
    ) -> bool;
    fn scanner_find_module_base(handle: *mut ScannerHandle, module_name: *const c_char) -> usize;
    fn scanner_resolve_pointer_chain(
        handle: *mut ScannerHandle,
        base_address: usize,
        offsets: *const u32,
        offset_count: i32,
        out: *mut usize,
    ) -> bool;
    fn scanner_read_by_config(
        handle: *mut ScannerHandle,
        module_name: *const c_char,
        base_offset: usize,
        offsets: *const u32,
        offset_count: i32,
        value_type: i32,
        out_value: *mut std::ffi::c_void,
    ) -> bool;
    fn scanner_aob_scan(
        handle: *mut ScannerHandle,
        pattern: *const u8,
        mask: *const c_char,
        start_address: usize,
        scan_size: usize,
    ) -> usize;
}

pub struct Scanner {
    handle: *mut ScannerHandle,
}

unsafe impl Send for Scanner {}
unsafe impl Sync for Scanner {}

impl Scanner {
    pub fn open(process_name: &str) -> Option<Self> {
        let c_name = CString::new(process_name).ok()?;
        let handle = unsafe { scanner_open_process(c_name.as_ptr()) };
        if handle.is_null() {
            None
        } else {
            Some(Scanner { handle })
        }
    }

    pub fn read_int(&self, address: usize) -> Option<i32> {
        let mut value: i32 = 0;
        if unsafe { scanner_read_int(self.handle, address, &mut value) } {
            Some(value)
        } else {
            None
        }
    }

    pub fn read_uint(&self, address: usize) -> Option<u32> {
        let mut value: u32 = 0;
        if unsafe { scanner_read_uint(self.handle, address, &mut value) } {
            Some(value)
        } else {
            None
        }
    }

    pub fn read_float(&self, address: usize) -> Option<f32> {
        let mut value: f32 = 0.0;
        if unsafe { scanner_read_float(self.handle, address, &mut value) } {
            Some(value)
        } else {
            None
        }
    }

    pub fn read_string(&self, address: usize, buffer_size: usize) -> Option<String> {
        let mut buffer = vec![0i8; buffer_size];
        if unsafe {
            scanner_read_string(
                self.handle,
                address,
                buffer.as_mut_ptr(),
                buffer_size as i32,
            )
        } {
            let cstr = unsafe { CStr::from_ptr(buffer.as_ptr()) };
            cstr.to_str().ok().map(|s| s.to_string())
        } else {
            None
        }
    }

    pub fn find_module_base(&self, module_name: &str) -> Option<usize> {
        let c_name = CString::new(module_name).ok()?;
        let base = unsafe { scanner_find_module_base(self.handle, c_name.as_ptr()) };
        if base != 0 {
            Some(base)
        } else {
            None
        }
    }

    pub fn resolve_pointer_chain(&self, base_address: usize, offsets: &[u32]) -> Option<usize> {
        let mut resolved: usize = 0;
        if unsafe {
            scanner_resolve_pointer_chain(
                self.handle,
                base_address,
                offsets.as_ptr(),
                offsets.len() as i32,
                &mut resolved,
            )
        } {
            Some(resolved)
        } else {
            None
        }
    }

    pub fn read_by_config(
        &self,
        module_name: &str,
        base_offset: usize,
        offsets: &[u32],
        value_type: i32,
    ) -> Option<Box<dyn std::any::Any>> {
        let c_name = CString::new(module_name).ok()?;
        let mut int_val: i32 = 0;
        let mut uint_val: u32 = 0;
        let mut float_val: f32 = 0.0;
        let mut str_val = [0i8; 256];

        let out_ptr = match value_type {
            0 => &mut int_val as *mut i32 as *mut std::ffi::c_void,
            1 => &mut uint_val as *mut u32 as *mut std::ffi::c_void,
            2 => &mut float_val as *mut f32 as *mut std::ffi::c_void,
            3 => str_val.as_mut_ptr() as *mut std::ffi::c_void,
            _ => return None,
        };

        let success = unsafe {
            scanner_read_by_config(
                self.handle,
                c_name.as_ptr(),
                base_offset,
                offsets.as_ptr(),
                offsets.len() as i32,
                value_type,
                out_ptr,
            )
        };

        if !success {
            return None;
        }

        match value_type {
            0 => Some(Box::new(int_val)),
            1 => Some(Box::new(uint_val)),
            2 => Some(Box::new(float_val)),
            3 => {
                let cstr = unsafe { CStr::from_ptr(str_val.as_ptr()) };
                Some(Box::new(cstr.to_str().unwrap_or("").to_string()))
            }
            _ => None,
        }
    }

    pub fn aob_scan(&self, pattern: &[u8], mask: &str, start_address: usize, scan_size: usize) -> Option<usize> {
        let c_mask = CString::new(mask).ok()?;
        let addr = unsafe {
            scanner_aob_scan(
                self.handle,
                pattern.as_ptr(),
                c_mask.as_ptr(),
                start_address,
                scan_size,
            )
        };
        if addr != 0 {
            Some(addr)
        } else {
            None
        }
    }
}

impl Drop for Scanner {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                scanner_close_process(self.handle);
            }
        }
    }
}
