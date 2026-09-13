use super::*;

fn data(text: &str, speaker: Option<&str>) -> TranscriptData {
    TranscriptData {
        text: text.to_owned(),
        speaker: speaker.map(str::to_owned),
    }
}

fn seen<'a>(
    blocks: impl IntoIterator<Item = &'a SubtitleBlock>,
) -> Vec<(Option<&'a str>, &'a str)> {
    blocks
        .into_iter()
        .map(|b| (b.speaker.as_deref(), b.text.as_str()))
        .collect()
}

#[test]
fn interim_keeps_every_speaker_segment() {
    let mut store = TranscriptionStore::new(8);

    store.update_interim(vec![
        data("Ты прав в какой-то степени", Some("1")),
        data("Возможно", Some("2")),
        data("Но ведь это все не напрасно?", Some("1")),
    ]);

    assert_eq!(
        seen(&store.interim_blocks),
        vec![
            (Some("1"), "Ты прав в какой-то степени"),
            (Some("2"), "Возможно"),
            (Some("1"), "Но ведь это все не напрасно?"),
        ]
    );
}

#[test]
fn interim_replaces_the_previous_batch() {
    let mut store = TranscriptionStore::new(8);
    store.update_interim(vec![data("первый", Some("1"))]);
    store.update_interim(vec![data("второй", Some("1"))]);
    assert_eq!(seen(&store.interim_blocks), vec![(Some("1"), "второй")]);
}

#[test]
fn empty_interim_clears_the_tail() {
    let mut store = TranscriptionStore::new(8);
    store.update_interim(vec![data("хвост", Some("1"))]);
    store.update_interim(vec![]);
    assert!(store.interim_blocks.is_empty());
}

#[test]
fn final_starts_a_new_block_on_speaker_change() {
    let mut store = TranscriptionStore::new(8);
    store.update(data("Привет.", Some("1")));
    store.update(data(" И тебе.", Some("2")));
    assert_eq!(
        seen(&store.blocks),
        vec![(Some("1"), "Привет."), (Some("2"), " И тебе.")]
    );
}

#[test]
fn final_appends_within_one_speaker() {
    let mut store = TranscriptionStore::new(8);
    store.update(data("Привет", Some("1")));
    store.update(data(", как дела?", Some("1")));
    assert_eq!(seen(&store.blocks), vec![(Some("1"), "Привет, как дела?")]);
}

#[test]
fn final_drops_the_interim_tail() {
    let mut store = TranscriptionStore::new(8);
    store.update_interim(vec![data("черновик", Some("1"))]);
    store.update(data("итог", Some("1")));
    assert!(store.interim_blocks.is_empty());
}

#[test]
fn blocks_do_not_grow_past_the_limit() {
    let mut store = TranscriptionStore::new(2);
    for i in 0..5 {
        store.update(data("x", Some(&i.to_string())));
    }
    assert_eq!(store.blocks.len(), 2);
}
