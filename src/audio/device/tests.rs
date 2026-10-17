use super::*;

#[test]
fn a_source_from_another_system_survives_a_round_trip() {
    let saved = "kind = \"output\"\nid = \"nosuchhost:speakers\"\n";

    let source: AudioSource = toml::from_str(saved).unwrap();
    assert!(!source.is_native());
    assert_eq!(toml::to_string(&source).unwrap(), saved);
}
