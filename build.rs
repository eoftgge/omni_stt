use image::{ImageFormat, open};
use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target = env::var("TARGET").unwrap_or_default();
    let host = env::var("HOST").unwrap_or_default();

    if target_os == "windows" {
        let img = open("assets/icon.png").expect("Couldn't open icon.png");
        let out_dir = env::var("OUT_DIR").unwrap();
        let ico_path = std::path::PathBuf::from(&out_dir).join("icon.ico");
        img.save_with_format(&ico_path, ImageFormat::Ico)
            .expect("Failed to write icon.ico");

        let mut res = winres::WindowsResource::new();
        res.set_icon(
            ico_path
                .to_str()
                .expect("Couldn't convert icon path to string"),
        );

        if !host.contains("windows") {
            if target.contains("gnu") {
                let windres = find_cross_tool(&target, "windres")
                    .unwrap_or_else(|| "x86_64-w64-mingw32-windres".to_string());

                res.set_windres_path(&windres);
                res.set_toolkit_path("");
            } else if target.contains("msvc") {
                let _ = find_llvm_tool("llvm-rc")
                    .expect("llvm-rc not found. Run: sudo apt install llvm");

                let fake_toolkit = std::path::PathBuf::from(&out_dir).join("fake_toolkit");
                std::fs::create_dir_all(&fake_toolkit)
                    .expect("Failed to create fake_toolkit dir");

                let rc_link = fake_toolkit.join("rc");
                let _ = std::fs::remove_file(&rc_link);

                #[cfg(unix)]
                std::os::unix::fs::symlink(&llvm_rc, &rc_link)
                    .expect("Failed to create symlink rc -> llvm-rc");

                res.set_toolkit_path(
                    fake_toolkit.to_str().expect("Invalid fake_toolkit path"),
                );
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

fn find_cross_tool(target: &str, tool: &str) -> Option<String> {
    let candidates: &[&str] = if target.contains("x86_64") {
        &["x86_64-w64-mingw32", "x86_64-pc-windows-gnu"]
    } else if target.contains("i686") || target.contains("i586") {
        &["i686-w64-mingw32", "i686-pc-windows-gnu"]
    } else if target.contains("aarch64") {
        &["aarch64-w64-mingw32"]
    } else {
        &[]
    };

    for prefix in candidates {
        let candidate = format!("{prefix}-{tool}");
        if command_exists(&candidate) {
            return Some(candidate);
        }
    }

    if command_exists(tool) {
        return Some(tool.to_string());
    }

    None
}

fn find_llvm_tool(tool: &str) -> Option<String> {
    if command_exists(tool) {
        return Some(tool.to_string());
    }

    for version in (10u32..=18).rev() {
        let candidate = format!("{tool}-{version}");
        if command_exists(&candidate) {
            return Some(candidate);
        }
    }

    None
}

fn command_exists(cmd: &str) -> bool {
    std::process::Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}