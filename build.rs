use std::{env, path::Path};

use bindgen::{EnumVariation, builder};

fn main() {
    let bindings = builder()
        .use_core()
        .layout_tests(false)
        .generate_comments(false)
        .default_enum_style(EnumVariation::ModuleConsts)
        .prepend_enum_name(false)
        .header("/usr/include/linux/tcp.h")
        .allowlist_type("tcp_info")
        .generate()
        .expect("Failed to build tcp_info bindings");

    let output_path = Path::new(&env::var("OUT_DIR").expect("OUT_DIR env var was not defined"))
        .join("linux_tcp_info.rs");

    bindings
        .write_to_file(output_path)
        .expect("Failed to write tcp_info bindings");
}
