use std::{env, path::Path};

use bindgen_helpers::{EnumVariation, Renamer, builder};

fn main() {
    let mut renamer = Renamer::new(false);
    renamer.rename_item("tcp_info", "TcpInfo");
    let bindings = builder()
        .use_core()
        .layout_tests(false)
        .generate_comments(false)
        .default_enum_style(EnumVariation::ModuleConsts)
        .prepend_enum_name(false)
        .header("/usr/include/linux/tcp.h")
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
