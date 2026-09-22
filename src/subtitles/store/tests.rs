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

#[test]
fn separator_promotes_every_interim_block() {
    let mut store = TranscriptionStore::new(8);
    store.update_interim(vec![data("первый ", Some("1")), data("второй ", Some("2"))]);
    store.ensure_separator();

    assert!(store.interim_blocks.is_empty());
    assert_eq!(store.blocks.len(), 2, "промотались не все интерим-блоки");
    assert_eq!(store.blocks[0].text, "первый ");
    assert!(
        store.blocks[1].text.starts_with("второй..."),
        "маркер продолжения не на последнем блоке: {:?}",
        store.blocks[1].text
    );
}

#[test]
fn a_block_is_logged_only_once_it_can_no_longer_grow() {
    let mut store = TranscriptionStore::new(8);

    store.update(data("Привет.", Some("1")));
    assert!(
        taken(&mut store).is_empty(),
        "открытый блок ещё может дорасти"
    );

    store.update(data("И тебе.", Some("2")));
    assert_eq!(
        taken(&mut store),
        vec![(Some("1".into()), "Привет.".into())]
    );
}

#[test]
fn overflow_does_not_swallow_blocks() {
    let mut store = TranscriptionStore::new(1);

    store.update(data("первый", Some("1")));
    store.update(data("второй", Some("2")));
    store.update(data("третий", Some("3")));

    assert_eq!(store.blocks.len(), 1);
    assert_eq!(
        taken(&mut store),
        vec![
            (Some("1".into()), "первый".into()),
            (Some("2".into()), "второй".into()),
        ]
    );
}

#[test]
fn finish_closes_the_last_open_block() {
    let mut store = TranscriptionStore::new(8);
    store.update(data("последняя фраза", Some("1")));

    store.finish();

    assert_eq!(
        taken(&mut store),
        vec![(Some("1".into()), "последняя фраза".into())]
    );
    assert!(store.blocks.is_empty());
}

#[test]
fn finishing_twice_does_not_duplicate_anything() {
    let mut store = TranscriptionStore::new(8);
    store.update(data("фраза", Some("1")));

    store.finish();
    store.finish();

    assert_eq!(taken(&mut store).len(), 1);
}

#[test]
fn promoted_interim_blocks_reach_the_log() {
    let mut store = TranscriptionStore::new(8);
    store.update(data("финал", Some("1")));
    store.update_interim(vec![data("промежуточный", Some("2"))]);

    store.ensure_separator();
    store.finish();

    assert_eq!(
        taken(&mut store),
        vec![
            (Some("1".into()), "финал".into()),
            (Some("2".into()), "промежуточный...".into()),
        ]
    );
}

fn taken(store: &mut TranscriptionStore) -> Vec<(Option<String>, String)> {
    store
        .take_completed()
        .map(|b| (b.speaker, b.text))
        .collect()
}
