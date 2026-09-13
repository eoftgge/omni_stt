use super::ffi::{FnResult, VoskApi, VoskModel, VoskRecognizer};
use std::ffi::{CStr, CString};
use std::path::Path;
use std::sync::Arc;

pub struct Model {
    api: Arc<VoskApi>,
    ptr: *mut VoskModel,
}

unsafe impl Send for Model {}
unsafe impl Sync for Model {}

impl Model {
    pub fn load(api: Arc<VoskApi>, path: &Path) -> Result<Self, String> {
        let c_path = CString::new(path.to_string_lossy().as_bytes())
            .map_err(|_| "model path contains a NUL byte".to_owned())?;

        let ptr = unsafe { (api.model_new)(c_path.as_ptr()) };
        if ptr.is_null() {
            return Err(format!("failed to load model from {}", path.display()));
        }
        Ok(Self { api, ptr })
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        unsafe { (self.api.model_free)(self.ptr) };
    }
}

pub enum Decoding {
    Final,
    Partial,
    Failed,
}

pub struct Recognizer {
    api: Arc<VoskApi>,
    /// keeps the model alive: can’t release it before the recognizer.
    _model: Arc<Model>,
    ptr: *mut VoskRecognizer,
}

unsafe impl Send for Recognizer {}

impl Recognizer {
    pub fn new(model: Arc<Model>, sample_rate: f32) -> Result<Self, String> {
        let api = Arc::clone(&model.api);
        let ptr = unsafe { (api.recognizer_new)(model.ptr, sample_rate) };
        if ptr.is_null() {
            return Err("failed to create recognizer".to_owned());
        }
        Ok(Self {
            api,
            _model: model,
            ptr,
        })
    }

    pub fn accept(&mut self, samples: &[i16]) -> Decoding {
        let len = samples.len().min(i32::MAX as usize) as i32;
        match unsafe { (self.api.accept_waveform_s)(self.ptr, samples.as_ptr(), len) } {
            1 => Decoding::Final,
            0 => Decoding::Partial,
            _ => Decoding::Failed,
        }
    }

    pub fn result(&mut self) -> String {
        self.take(self.api.result)
    }

    pub fn partial_result(&mut self) -> String {
        self.take(self.api.partial_result)
    }

    pub fn final_result(&mut self) -> String {
        self.take(self.api.final_result)
    }

    fn take(&self, f: FnResult) -> String {
        let raw = unsafe { f(self.ptr) };
        if raw.is_null() {
            return String::new();
        }
        unsafe { CStr::from_ptr(raw) }.to_string_lossy().into_owned()
    }
}

impl Drop for Recognizer {
    fn drop(&mut self) {
        unsafe { (self.api.recognizer_free)(self.ptr) };
    }
}