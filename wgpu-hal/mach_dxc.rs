use cached_path::{Cache, Options};
use std::{env, path::PathBuf};
pub fn bundle_mach_dxc() {
    println!("cargo:rerun-if-changed=mach_dxc.rs");
    println!("cargo:rerun-if-changed=mach_dxc.h");
    generate_bindings();
    link_lib();
}

fn generate_bindings() {
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("mach_dxc.h")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the bindings.rs file.
    let out_path = PathBuf::from("src/dx12/mach_dxc/bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}

fn link_lib() {
    let path = download_if_no_cache();
    println!("cargo::rustc-link-lib=machdxcompiler");
    println!("cargo::rustc-link-search=[KIND=static]{:?}", path.display());
}

fn download_if_no_cache() -> PathBuf {
    let project_name = "dxcompiler";
    let project_url = format!("https://github.com/hexops/mach-{project_name}");
    let latest_binary_release = "2024.10.16+da605cf.1";

    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let abi = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_else(|_| "none".to_owned());
    let triple = format!("{arch}-{os}-{abi}");

    if !matches!(
        triple.as_str(),
        "x86_64-linux-gnu"
            | "x86_64-linux-musl"
            | "aarch64-linux-gnu"
            | "aarch64-linux-musl"
            | "x86_64-windows-gnu"
            | "aarch64-windows-gnu"
            | "x86_64-macos-none"
            | "aarch64-macos-none"
    ) {
        panic!("Unsupported target: {triple}\nCheck support targets on {project_url}");
    }

    // Compose the download URL, e.g.
    // https://github.com/hexops/mach-dxcompiler/releases/download/2023.11.30%2Ba451866.3/aarch64-linux-gnu_Debug_bin.tar.gz
    let release_file_url = format!(
        "{project_url}/releases/download/{latest_binary_release}/{triple}_ReleaseFast_lib.tar.gz"
    );
    let cache = Cache::builder().build().unwrap();
    cache
        .cached_path_with_options(
            release_file_url.as_str(),
            &Options {
                subdir: Some("mach-dxcompiler-sys".to_owned()),
                extract: true,
            },
        )
        .unwrap()
}
