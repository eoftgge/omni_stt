use image::{ImageFormat, open};
use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target = env::var("TARGET").unwrap_or_default();
    let host = env::var("HOST").unwrap_or_default();

    if target_os == "windows" {
        let img = open("assets/icon.png").expect("Couldn't open icon.png");
        let out_dir = env::var("OUT_DIR").unwrap();
        let ico_path = std::path::PathBuf::from(out_dir).join("icon.ico");
        img.save_with_format(&ico_path, ImageFormat::Ico)
            .expect("Failed to write icon.png");

        let mut res = winres::WindowsResource::new();
        res.set_icon(
            ico_path
                .to_str()
                .expect("Couldn't convert icon path to string"),
        );

        if !host.contains("windows") {
            if target.contains("gnu") {
                res.set_toolkit_path("x86_64-w64-mingw32-windres");
            } else if target.contains("msvc") {
                // res.set_toolkit_path("llvm-rc");
            }
        }

        res.compile().expect("Failed to compile resource");
    } else if target_os == "macos" {
        println!("cargo:rustc-link-arg=-Wl,-sectcreate,__TEXT,__info_plist,Info.plist");
    }

    if env::var("CARGO_FEATURE_VOSK").is_ok() {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let platform = if target_os == "windows" {
            "windows"
        } else if target_os == "macos" {
            "macos"
        } else {
            "linux"
        };

        let lib_dir = format!("{manifest_dir}/assets/vosk/{platform}");
        println!("cargo:rustc-link-search=native={lib_dir}");

        if target_os == "linux" {
            println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
        } else if target_os == "macos" {
            println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
        }
    }
}