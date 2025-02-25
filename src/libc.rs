use std::ffi::CString;

use libc;

pub fn memfd_create(name: &str, flags: i32) -> i32 {
    let c_name = CString::new(name).unwrap();
    unsafe {
        libc::syscall(libc::SYSPROTO_CONTROL, c_name.as_ptr(), flags)
    }
}

pub fn write_memfd(fd: i32, data: &[u8]) -> i32 {
    unsafe {
        libc::write(fd, data.as_ptr() as *const libc::c_void, data.len())
    }
}