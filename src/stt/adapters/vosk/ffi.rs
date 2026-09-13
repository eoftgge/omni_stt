use libloading::{Library, Symbol};
use std::ffi::{c_char, c_int};
use std::path::{Path, PathBuf};

#[repr(C)]
pub struct VoskModel {
    _private: [u8; 0],
}

#[repr(C)]
pub struct VoskRecognizer {
    _private: [u8; 0],
}

pub type FnResult = unsafe extern "C" fn(*mut VoskRecognizer) -> *const c_char;

pub struct VoskApi {
    pub set_log_level: unsafe extern "C" fn(c_int),
    pub model_new: unsafe extern "C" fn(*const c_char) -> *mut VoskModel,
    pub model_free: unsafe extern "C" fn(*mut VoskModel),
    pub recognizer_new: unsafe extern "C" fn(*mut VoskModel, f32) -> *mut VoskRecognizer,
    pub recognizer_free: unsafe extern "C" fn(*mut VoskRecognizer),
    pub accept_waveform_s: unsafe extern "C" fn(*mut VoskRecognizer, *const i16, c_int) -> c_int,
    pub result: FnResult,
    pub partial_result: FnResult,
    pub final_result: FnResult,
    _lib: Library,
}

impl VoskApi {
    pub fn load(explicit: Option<&Path>) -> Result<Self, String> {
        let candidates = candidates(explicit);
        let mut last = String::new();

        for path in &candidates {
            match unsafe { Library::new(path) } {
                Ok(lib) => {
                    let api = unsafe { Self::from_library(lib) }?;
                    unsafe { (api.set_log_level)(-1) };
                    return Ok(api);
                }
                Err(e) => last = format!("{}: {e}", path.display()),
            }
        }

        Err(format!("libvosk not found, last attempt — {last}"))
    }

    unsafe fn from_library(lib: Library) -> Result<Self, String> {
        unsafe fn sym<T: Copy>(lib: &Library, name: &[u8]) -> Result<T, String> {
            let symbol: Symbol<T> = unsafe { lib.get(name) }
                .map_err(|e| format!("symbol {}: {e}", String::from_utf8_lossy(name)))?;
            Ok(*symbol)
        }

        unsafe {
            Ok(Self {
                set_log_level: sym(&lib, b"vosk_set_log_level\0")?,
                model_new: sym(&lib, b"vosk_model_new\0")?,
                model_free: sym(&lib, b"vosk_model_free\0")?,
                recognizer_new: sym(&lib, b"vosk_recognizer_new\0")?,
                recognizer_free: sym(&lib, b"vosk_recognizer_free\0")?,
                accept_waveform_s: sym(&lib, b"vosk_recognizer_accept_waveform_s\0")?,
                result: sym(&lib, b"vosk_recognizer_result\0")?,
                partial_result: sym(&lib, b"vosk_recognizer_partial_result\0")?,
                final_result: sym(&lib, b"vosk_recognizer_final_result\0")?,
                _lib: lib,
            })
        }
    }
}

fn lib_file_name() -> &'static str {
    #[cfg(target_os = "windows")]
    return "libvosk.dll";
    #[cfg(target_os = "macos")]
    return "libvosk.dylib";
    #[cfg(all(unix, not(target_os = "macos")))]
    return "libvosk.so";
}

fn candidates(explicit: Option<&Path>) -> Vec<PathBuf> {
    let name = lib_file_name();
    let mut out = Vec::new();

    if let Some(path) = explicit {
        out.push(if path.is_dir() {
            path.join(name)
        } else {
            path.to_path_buf()
        });
    }

    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        out.push(dir.join(name));
    }

    out.push(PathBuf::from(name));
    out
}