/**
 * This file contains the FFI bindings to the libc functions that we need to use.
 *
 */
use std::ffi::CString;

use libc;

pub fn memfd_create(name: &str, flags: i32) -> i32 {
    let c_name = CString::new(name).unwrap();
    unsafe {
        libc::syscall(libc::SYS_memfd_create, c_name.as_ptr(), flags)
            .try_into()
            .unwrap()
    }
}

pub fn write_memfd(memfd: i32, data: &[u8]) {
    unsafe { libc::write(memfd, data.as_ptr() as *const libc::c_void, data.len()) };
}

#[allow(dead_code)]
pub fn close_memfd(memfd: i32) {
    unsafe { libc::close(memfd) };
}

pub fn execve_memfd(memfd: i32) -> i32 {
    let c_fd = CString::new(format!("/proc/self/fd/{}", memfd)).unwrap();
    let args = [c_fd.as_ptr(), std::ptr::null()];
    let env = [std::ptr::null()];
    unsafe { libc::execve(c_fd.as_ptr(), args.as_ptr(), env.as_ptr()) }
}
