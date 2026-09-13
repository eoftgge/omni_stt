use super::*;

fn tok(text: &str, is_final: bool, speaker: Option<&str>) -> SonioxTranscriptionToken {
    SonioxTranscriptionToken {
        text: text.to_owned(),
        is_final,
        speaker: speaker.map(str::to_owned),
        ..Default::default()
    }
}

#[derive(Debug, PartialEq)]
enum Seen {
    Final(Option<String>, String),
    Interim(Vec<(Option<String>, String)>),
}

fn seen_events(queue: &mut VecDeque<SttEvent>) -> Vec<Seen> {
    queue
        .drain(..)
        .filter_map(|e| match e {
            SttEvent::Transcript(d) => Some(Seen::Final(d.speaker, d.text)),
            SttEvent::Interim(v) => Some(Seen::Interim(
                v.into_iter().map(|d| (d.speaker, d.text)).collect(),
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn tokens_without_a_speaker_stay_one_segment() {
    let mut q = VecDeque::new();
    push_token_events(
        vec![tok("одна ", false, None), tok("реплика", false, None)],
        &mut q,
    );

    assert_eq!(
        seen_events(&mut q),
        vec![Seen::Interim(vec![(None, "одна реплика".into())])]
    );
}

#[test]
fn interim_is_split_per_speaker_and_sent_as_one_event() {
    let mut q = VecDeque::new();
    push_token_events(
        vec![
            tok(" Нет, нет", false, Some("1")),
            tok(" Да, да", false, Some("2")),
            tok(" Нет", false, Some("1")),
        ],
        &mut q,
    );

    assert_eq!(
        seen_events(&mut q),
        vec![Seen::Interim(vec![
            (Some("1".into()), " Нет, нет".into()),
            (Some("2".into()), " Да, да".into()),
            (Some("1".into()), " Нет".into()),
        ])]
    );
}

#[test]
fn finals_go_out_before_the_interim_tail() {
    let mut q = VecDeque::new();
    push_token_events(
        vec![
            tok("Мне так проще,", true, Some("1")),
            tok(" лучше и так.", false, Some("1")),
        ],
        &mut q,
    );

    assert_eq!(
        seen_events(&mut q),
        vec![
            Seen::Final(Some("1".into()), "Мне так проще,".into()),
            Seen::Interim(vec![(Some("1".into()), " лучше и так.".into())]),
        ]
    );
}

#[test]
fn a_fully_finalised_message_still_clears_the_tail() {
    let mut q = VecDeque::new();
    push_token_events(vec![tok("всё.", true, Some("1"))], &mut q);

    assert_eq!(
        seen_events(&mut q),
        vec![
            Seen::Final(Some("1".into()), "всё.".into()),
            Seen::Interim(vec![]),
        ]
    );
}

#[test]
fn an_empty_message_emits_nothing() {
    let mut q = VecDeque::new();
    push_token_events(vec![], &mut q);
    assert!(seen_events(&mut q).is_empty());
}

#[test]
fn original_translation_tokens_are_skipped() {
    let mut original = tok("original text", false, Some("1"));
    original.translation_status = Some("original".to_owned());

    let mut q = VecDeque::new();
    push_token_events(vec![original, tok("перевод", false, Some("1"))], &mut q);

    assert_eq!(
        seen_events(&mut q),
        vec![Seen::Interim(vec![(Some("1".into()), "перевод".into())])]
    );
}
