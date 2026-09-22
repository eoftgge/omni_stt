use crate::stt::store::TranscriptionStore;

pub struct VisualReplica<'a> {
    pub speaker: Option<&'a str>,
    pub elements: Vec<TextElement<'a>>,
}

pub struct TextElement<'a> {
    pub text: &'a str,
    pub is_interim: bool,
}

impl<'a> VisualReplica<'a> {
    pub fn new(speaker: Option<&'a str>) -> Self {
        Self {
            speaker,
            elements: Vec::new(),
        }
    }

    pub fn add_text(&mut self, text: &'a str, is_interim: bool) {
        self.elements.push(TextElement { text, is_interim });
    }
}

pub fn prepare_replicas(store: &'_ TranscriptionStore) -> Vec<VisualReplica<'_>> {
    let mut replicas: Vec<VisualReplica> = Vec::with_capacity(store.max_blocks());
    let final_blocks = store.blocks.iter().map(|b| (b, false));
    let interim_blocks = store.interim_blocks.iter().map(|b| (b, true));
    let all_blocks = final_blocks.chain(interim_blocks);

    for (block, is_interim) in all_blocks {
        if block.text.is_empty() {
            continue;
        }

        let speaker = block.speaker.as_deref();
        let should_merge = replicas
            .last()
            .map(|last| last.speaker == speaker)
            .unwrap_or(false);

        if !should_merge {
            replicas.push(VisualReplica::new(speaker))
        }

        let Some(target) = replicas.last_mut() else {
            tracing::warn!("Replicas hadn't last element...");
            continue;
        };
        if !block.text.is_empty() {
            target.add_text(&block.text, is_interim);
        }
    }

    replicas
}
