use super::ffi::VoskApi;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct VoskProbe {
    cache: Option<(PathBuf, Result<(), String>)>,
}

impl VoskProbe {
    pub fn check(&mut self, library_path: &Path) -> &Result<(), String> {
        let fresh = self
            .cache
            .as_ref()
            .is_some_and(|(probed, _)| probed.as_path() == library_path);

        if !fresh {
            let explicit = (!library_path.as_os_str().is_empty()).then_some(library_path);
            let status = VoskApi::load(explicit).map(|_| ());
            self.cache = Some((library_path.to_path_buf(), status));
        }

        match &self.cache {
            Some((_, status)) => status,
            None => unreachable!("cache filled above"),
        }
    }
}