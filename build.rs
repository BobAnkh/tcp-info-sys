use std::env;
use std::path::{Path, PathBuf};

use bindgen_helpers::{EnumVariation, Renamer, builder};

fn find_linux_headers_dir() -> PathBuf {
    let header_dir = PathBuf::from(
        env::var("LINUX_HEADERS_DIR").unwrap_or_else(|_| "/usr/include/".to_string()),
    );

    if header_dir.exists() {
        return header_dir;
    }
    let detailed_error_msg = "Could not find path to linux headers, and this `-sys` crate cannot
proceed without this header. If the header is present and this crate had
trouble finding it, you can set the `LINUX_HEADERS_DIR` environment variable for the
compilation process.";
    println!("cargo:warning=Could not find Linux headers, see stderr below for more info");
    eprintln!("{}", detailed_error_msg.replace('\n', " "));

    std::process::exit(101); // same as panic
}

fn main() {
    let mut renamer = Renamer::new(false);
    renamer.rename_item("tcp_info", "TcpInfo");

    let linux_headers_dir = find_linux_headers_dir();

    let bindings = builder()
        .use_core()
        .layout_tests(false)
        .generate_comments(false)
        .default_enum_style(EnumVariation::ModuleConsts)
        .prepend_enum_name(false)
        .clang_arg(format!("-I{}", linux_headers_dir.display()))
        .header(format!(
            "{}",
            linux_headers_dir.join("linux").join("tcp.h").display()
        ))
        .allowlist_type("tcp_info")
        .derive_hash(true)
        .derive_eq(true)
        .derive_partialeq(true)
        .derive_ord(true)
        .derive_partialord(true)
        .parse_callbacks(Box::new(renamer))
        .generate()
        .expect("Failed to build tcp_info bindings");

    let output_path = Path::new(&env::var("OUT_DIR").expect("OUT_DIR env var was not defined"))
        .join("linux_tcp_info.rs");

    bindings
        .write_to_file(output_path)
        .expect("Failed to write tcp_info bindings");
}
