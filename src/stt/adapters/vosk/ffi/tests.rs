use super::{VoskApi, candidates, lib_file_name};
use std::path::{Path, PathBuf};

#[test]
fn load_reports_every_path_it_tried() {
	let missing = [
		PathBuf::from("/omni_stt_no_such_dir/libvosk.so"),
		PathBuf::from("/omni_stt_also_missing/libvosk.so"),
	];

	let Err(err) = VoskApi::load_from(&missing) else {
		panic!("libvosk loaded from a path that cannot exist");
	};

	// the user types library_path by hand and mistypes it; a message naming
	// only the last fallback leaves them with no idea what went wrong
	for path in &missing {
		assert!(
			err.contains(&path.display().to_string()),
			"{err}\ndoes not mention {}",
			path.display()
		);
	}
}

#[test]
fn explicit_file_is_tried_first() {
	let explicit = PathBuf::from("/opt/vosk/custom_name.so");

	assert_eq!(candidates(Some(&explicit)).first(), Some(&explicit));
}

#[test]
fn explicit_directory_gets_the_platform_file_name() {
	let dir = std::env::temp_dir();

	assert_eq!(
		candidates(Some(&dir)).first(),
		Some(&dir.join(lib_file_name()))
	);
}

#[test]
fn bare_name_is_always_the_last_resort() {
	let bare = PathBuf::from(lib_file_name());

	assert_eq!(candidates(None).last(), Some(&bare));
	assert_eq!(
		candidates(Some(Path::new("/opt/vosk/libvosk.so"))).last(),
		Some(&bare)
	);
}

#[test]
fn explicit_path_prepends_and_does_not_replace_the_fallbacks() {
	let explicit = PathBuf::from("/opt/vosk/libvosk.so");

	assert_eq!(
		candidates(Some(&explicit)).len(),
		candidates(None).len() + 1
	);
}