# tcp-info-sys

[![github-repo](https://img.shields.io/badge/github-BobAnkh/tcp--info--sys-f5dc23?logo=github)](https://github.com/BobAnkh/tcp-info-sys)
[![crates.io](https://img.shields.io/crates/v/tcp-info-sys.svg?logo=rust)](https://crates.io/crates/tcp-info-sys)
[![docs.rs](https://img.shields.io/badge/docs.rs-tcp--info--sys-blue?logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/tcp-info-sys)
[![LICENSE Apache-2.0](https://img.shields.io/github/license/BobAnkh/tcp-info-sys?logo=Apache)](https://github.com/BobAnkh/tcp-info-sys/blob/main/LICENSE)

A library to get TCP_INFO from the kernel for a TCP socket.
Provide the binding of TCP_INFO struct and a safe interface to get it from kernel with socket file descriptor.

## Examples

More detailed explanation can be found in [documentation](https://docs.rs/tcp-info-sys).

```rust
use std::io::prelude::*;
use std::net::TcpStream;
use std::os::fd::AsRawFd;
use tcp_info_sys::get_tcp_info;

fn main() -> std::io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:12345")?;
    let tcp_info = get_tcp_info(stream.as_raw_fd())?;
    println!("TCP Info: {:?}", tcp_info);
    Ok(())
}
```

## Maintainer

[@BobAnkh](https://github.com/BobAnkh)

## How to contribute

You should follow our [Code of Conduct](/CODE_OF_CONDUCT.md).

See [CONTRIBUTING GUIDELINES](/CONTRIBUTING.md) for contributing conventions.

Make sure to pass all the tests before submitting your code.

### Contributors

## LICENSE

[Apache-2.0](LICENSE) © BobAnkh
