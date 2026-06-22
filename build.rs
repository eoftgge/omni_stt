use std::env;
use image::{ImageFormat, open};

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").is_ok() {
        let img = open("assets/icon.png").expect("Couldn't open icon.png");
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let ico_path = std::path::PathBuf::from(out_dir).join("icon.ico");
        img.save_with_format(&ico_path, ImageFormat::Ico)
            .expect("Failed to write icon.png");

        let mut res = winres::WindowsResource::new();
        res.set_icon(
            ico_path
                .to_str()
                .expect("Couldn't convert icon path to string"),
        );
        res.compile().expect("Failed to compile resource");
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-sectcreate,__TEXT,__info_plist,Info.plist");
    }

    if env::var("CARGO_FEATURE_VOSK").is_ok() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let platform = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        };

        let lib_dir = format!("{manifest_dir}/assets/vosk/{platform}");
        println!("cargo:rustc-link-search=native={lib_dir}");
        println!("cargo:rustc-link-lib=dylib=vosk");

        #[cfg(target_os = "linux")]
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
        #[cfg(target_os = "macos")]
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
    }
}
