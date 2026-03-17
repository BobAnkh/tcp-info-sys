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

include!(concat!(env!("OUT_DIR"), "/serde_impl.rs"));

/// Get TCP_INFO for a socket (with its fd)
///
/// ## Examples
///
/// ```no_run
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

#[cfg(test)]
mod tests {
    #[cfg(feature = "serde")]
    use super::TcpInfo;

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrip_json() {
        // Build a TcpInfo with known values
        let original = TcpInfo {
            tcpi_state: 1,
            tcpi_ca_state: 2,
            tcpi_retransmits: 3,
            tcpi_probes: 4,
            tcpi_backoff: 5,
            tcpi_options: 6,
            _bitfield_align_1: [],
            _bitfield_1: TcpInfo::new_bitfield_1(7, 8, 1, 2),
            tcpi_rto: 100,
            tcpi_ato: 200,
            tcpi_snd_mss: 1460,
            tcpi_rcv_mss: 1460,
            tcpi_unacked: 0,
            tcpi_sacked: 0,
            tcpi_lost: 0,
            tcpi_retrans: 0,
            tcpi_fackets: 0,
            tcpi_last_data_sent: 10,
            tcpi_last_ack_sent: 11,
            tcpi_last_data_recv: 12,
            tcpi_last_ack_recv: 13,
            tcpi_pmtu: 1500,
            tcpi_rcv_ssthresh: 65535,
            tcpi_rtt: 5000,
            tcpi_rttvar: 1000,
            tcpi_snd_ssthresh: 2147483647,
            tcpi_snd_cwnd: 10,
            tcpi_advmss: 1448,
            tcpi_reordering: 3,
            tcpi_rcv_rtt: 0,
            tcpi_rcv_space: 87380,
            tcpi_total_retrans: 0,
            tcpi_pacing_rate: 1_000_000,
            tcpi_max_pacing_rate: u64::MAX,
            tcpi_bytes_acked: 12345,
            tcpi_bytes_received: 67890,
            tcpi_segs_out: 50,
            tcpi_segs_in: 60,
            tcpi_notsent_bytes: 0,
            tcpi_min_rtt: 4000,
            tcpi_data_segs_in: 55,
            tcpi_data_segs_out: 45,
            tcpi_delivery_rate: 500_000,
            tcpi_busy_time: 1000,
            tcpi_rwnd_limited: 0,
            tcpi_sndbuf_limited: 0,
            tcpi_delivered: 44,
            tcpi_delivered_ce: 0,
            tcpi_bytes_sent: 99999,
            tcpi_bytes_retrans: 0,
            tcpi_dsack_dups: 0,
            tcpi_reord_seen: 0,
            tcpi_rcv_ooopack: 0,
            tcpi_snd_wnd: 65535,
        };
        let _ = original; // Silence unused warning if fields differ on older kernels

        let json = serde_json::to_string(&original).expect("serialize failed");
        let restored: TcpInfo = serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(original, restored, "round-trip mismatch");

        // Verify bitfields were preserved correctly
        assert_eq!(restored.tcpi_snd_wscale(), 7);
        assert_eq!(restored.tcpi_rcv_wscale(), 8);
        assert_eq!(restored.tcpi_delivery_rate_app_limited(), 1);
        assert_eq!(restored.tcpi_fastopen_client_fail(), 2);
    }
}
