use super::ffi::VoskApi;
use std::path::{Path, PathBuf};

/// Caches whether libvosk can be loaded from a given path.
#[derive(Default)]
pub struct VoskProbe {
    cache: Option<(PathBuf, Result<(), String>)>,
}

impl VoskProbe {
    /// Reports whether libvosk is available at `library_path`.
    ///
    /// This runs from the settings UI on every frame, so the library is only
    /// opened again when the path actually changed: libvosk is around 25 MB,
    /// and loading it per frame would stall the window.
    pub fn check(&mut self, library_path: &Path) -> &Result<(), String> {
        let fresh = self
            .cache
            .as_ref()
            .is_some_and(|(probed, _)| probed.as_path() == library_path);

        if !fresh {
            let explicit = (!library_path.as_os_str().is_empty()).then_some(library_path);
            // the loaded VoskApi is dropped immediately on purpose: all we want
            // to know is that the library opens and every symbol resolves
            let status = VoskApi::load(explicit).map(|_| ());
            self.cache = Some((library_path.to_path_buf(), status));
        }

        match &self.cache {
            Some((_, status)) => status,
            None => unreachable!("cache filled above"),
        }
    }
}