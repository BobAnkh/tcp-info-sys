use std::env;
use std::path::{Path, PathBuf};

use bindgen_helpers::{EnumVariation, Renamer, builder};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Fields, Item, Type, parse_file};

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
        .derive_default(true)
        .derive_hash(true)
        .derive_eq(true)
        .derive_partialeq(true)
        .derive_ord(true)
        .derive_partialord(true)
        .parse_callbacks(Box::new(renamer))
        .generate()
        .expect("Failed to build tcp_info bindings");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR env var was not defined");
    let output_path = Path::new(&out_dir).join("linux_tcp_info.rs");

    bindings
        .write_to_file(&output_path)
        .expect("Failed to write tcp_info bindings");

    generate_serde_impl(&output_path, &out_dir);
}

/// Parses the bindgen-generated file, finds the TcpInfo struct, and emits a
/// serde_impl.rs that implements Serialize and Deserialize exactly matching
/// whatever fields were generated for the current kernel version.
fn generate_serde_impl(bindings_path: &Path, out_dir: &str) {
    let src = std::fs::read_to_string(bindings_path).expect("Failed to read bindings file");
    let ast = parse_file(&src).expect("Failed to parse bindings file");

    // Collect regular (non-bitfield) fields and bitfield field names
    let mut regular_fields: Vec<(syn::Ident, Type)> = Vec::new();
    // Accessor names for the logical bitfield values (tcpi_snd_wscale, etc.)
    let mut bitfield_accessors: Vec<syn::Ident> = Vec::new();

    for item in &ast.items {
        let Item::Struct(s) = item else { continue };
        if s.ident != "TcpInfo" {
            continue;
        }
        let Fields::Named(named) = &s.fields else {
            continue;
        };
        for field in &named.named {
            let name = field.ident.as_ref().unwrap();
            let name_str = name.to_string();
            // Skip the zero-sized alignment marker inserted by bindgen for bitfields
            if name_str == "_bitfield_align_1" {
                continue;
            }
            // The bitfield storage unit — we replace it with logical accessor fields
            if name_str.starts_with("_bitfield_") {
                continue;
            }
            regular_fields.push((name.clone(), field.ty.clone()));
        }
    }

    // Find bitfield accessor methods (getters that take &self and return a value).
    // These are the logical bitfield fields we need to serialize.
    for item in &ast.items {
        let Item::Impl(impl_block) = item else {
            continue;
        };
        // Only look at the inherent impl for TcpInfo (no trait)
        if impl_block.trait_.is_some() {
            continue;
        }
        let syn::Type::Path(tp) = impl_block.self_ty.as_ref() else {
            continue;
        };
        if tp
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .as_deref()
            != Some("TcpInfo")
        {
            continue;
        }
        for impl_item in &impl_block.items {
            let syn::ImplItem::Fn(method) = impl_item else {
                continue;
            };
            let method_name = method.sig.ident.to_string();
            // Include getter methods: take &self, no additional args, not raw/set variants
            let is_getter = method.sig.inputs.len() == 1
                && !method_name.starts_with("set_")
                && !method_name.ends_with("_raw")
                && method_name != "new_bitfield_1";
            if is_getter {
                bitfield_accessors.push(method.sig.ident.clone());
            }
        }
    }

    assert!(
        !regular_fields.is_empty(),
        "Could not find TcpInfo struct fields in generated bindings"
    );

    let field_count = regular_fields.len() + bitfield_accessors.len();

    // --- Serialize impl ---
    let serialize_fields = regular_fields.iter().map(|(name, _ty)| {
        let name_str = name.to_string();
        quote! { state.serialize_field(#name_str, &self.#name)?; }
    });
    let serialize_bitfields = bitfield_accessors.iter().map(|name| {
        let name_str = name.to_string();
        quote! { state.serialize_field(#name_str, &self.#name())?; }
    });

    // --- Helper struct for Deserialize ---
    // Maps __u8/__u32/__u64 type aliases to plain Rust primitives in the helper struct
    let helper_fields = regular_fields.iter().map(|(name, ty)| {
        let plain_ty = map_to_plain_type(ty);
        quote! { #name: #plain_ty, }
    });
    let bitfield_helper_fields = bitfield_accessors.iter().map(|name| {
        quote! { #name: u8, }
    });

    // Reconstruct TcpInfo from the helper
    let reconstruct_regular = regular_fields.iter().map(|(name, _)| {
        quote! { #name: helper.#name, }
    });

    // Find the new_bitfield_1 method signature to know arg order
    let bitfield_args = bitfield_accessors.iter().map(|name| {
        quote! { helper.#name, }
    });

    let struct_name = syn::Ident::new("TcpInfoSerdeHelper", Span::call_site());

    let tokens: TokenStream = quote! {
        #[cfg(feature = "serde")]
        impl ::serde::Serialize for TcpInfo {
            fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                use ::serde::ser::SerializeStruct;
                let mut state = serializer.serialize_struct("TcpInfo", #field_count)?;
                #(#serialize_fields)*
                #(#serialize_bitfields)*
                state.end()
            }
        }

        #[cfg(feature = "serde")]
        #[derive(::serde::Deserialize)]
        #[serde(rename = "TcpInfo")]
        struct #struct_name {
            #(#helper_fields)*
            #(#bitfield_helper_fields)*
        }

        #[cfg(feature = "serde")]
        impl<'de> ::serde::Deserialize<'de> for TcpInfo {
            fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let helper = #struct_name::deserialize(deserializer)?;
                Ok(TcpInfo {
                    #(#reconstruct_regular)*
                    _bitfield_align_1: [],
                    _bitfield_1: TcpInfo::new_bitfield_1(#(#bitfield_args)*),
                })
            }
        }
    };

    let serde_impl_path = Path::new(out_dir).join("serde_impl.rs");
    std::fs::write(&serde_impl_path, tokens.to_string()).expect("Failed to write serde_impl.rs");
}

/// Maps bindgen C type aliases to plain Rust primitive types for the helper struct.
fn map_to_plain_type(ty: &syn::Type) -> proc_macro2::TokenStream {
    use quote::quote;
    // Stringify the type and map known aliases
    let type_str = quote!(#ty).to_string().replace(' ', "");
    match type_str.as_str() {
        "__u8" | "::core::ffi::c_uchar" => quote!(u8),
        "__u16" | "::core::ffi::c_ushort" => quote!(u16),
        "__u32" | "::core::ffi::c_uint" => quote!(u32),
        "__u64" | "::core::ffi::c_ulonglong" => quote!(u64),
        _ => quote!(#ty), // fall back to the original type
    }
}
