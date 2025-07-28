use libc::{c_void, socklen_t};
use std::mem::MaybeUninit;
use std::os::fd::AsRawFd;

#[allow(
    non_camel_case_types,
    unsafe_op_in_unsafe_fn,
    clippy::useless_transmute,
    clippy::missing_safety_doc,
    clippy::ptr_offset_with_cast
)]
mod tcp_info {
    include!(concat!(env!("OUT_DIR"), "/linux_tcp_info.rs"));
}

/// The binding to the `tcp_info` struct in the kernel
pub use tcp_info::TcpInfo;

/// Get TCP_INFO for a socket (with its fd)
///
/// ## Examples
///
/// ```
/// use std::io::prelude::*;
/// use std::net::TcpStream;
/// use std::os::fd::AsRawFd;
/// use tcp_info_sys::get_tcp_info;
///
/// fn main() -> std::io::Result<()> {
///     let stream = TcpStream::connect("127.0.0.1:12345")?;
///     let tcp_info = get_tcp_info(stream.as_raw_fd())?;
///     println!("TCP Info: {:?}", tcp_info);
///     Ok(())
/// }
/// ```
pub fn get_tcp_info<T: AsRawFd>(sk_fd: T) -> Result<TcpInfo, std::io::Error> {
    let mut tcp_info: GetSockOptStruct<TcpInfo> = GetSockOptStruct::uninit();
    let res = unsafe {
        libc::getsockopt(
            sk_fd.as_raw_fd(),
            libc::SOL_TCP,
            libc::TCP_INFO,
            tcp_info.ffi_ptr(),
            tcp_info.ffi_len(),
        )
    };
    if res == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        let tcp_info = tcp_info.assume_init();
        Ok(tcp_info)
    }
}

struct GetSockOptStruct<T> {
    len: socklen_t,
    val: MaybeUninit<T>,
}

impl<T> GetSockOptStruct<T> {
    fn uninit() -> Self {
        GetSockOptStruct {
            len: std::mem::size_of::<T>() as socklen_t,
            val: MaybeUninit::uninit(),
        }
    }

    fn ffi_ptr(&mut self) -> *mut c_void {
        self.val.as_mut_ptr() as *mut c_void
    }

    fn ffi_len(&mut self) -> *mut socklen_t {
        &mut self.len
    }

    fn assume_init(self) -> T {
        assert_eq!(
            self.len as usize,
            std::mem::size_of::<T>(),
            "invalid getsockopt implementation"
        );
        unsafe { self.val.assume_init() }
    }
}
