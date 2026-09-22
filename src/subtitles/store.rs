#[cfg(test)]
mod tests;

use crate::event::TranscriptData;
use crate::subtitles::block::SubtitleBlock;
use crate::subtitles::text::{is_cjk, is_punctuation_or_symbol};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// How long a block may grow before the next final is allowed to start a new
/// one. Counted in characters: `String::len` is bytes, which would make the
/// limit depend on the language — twice as tight for Cyrillic, three times
/// for CJK.
const SOFT_LIMIT_CHARS: usize = 200;
/// Marks a gap where the connection dropped. The trailing spaces keep the
/// ellipsis from running into whatever arrives when the stream resumes.
const GAP_MARKER: &str = "... ";

pub struct TranscriptionStore {
    blocks: VecDeque<SubtitleBlock>,
    interim_blocks: Vec<SubtitleBlock>,
    max_blocks: usize,
    last_activity: Option<Instant>,
    completed: Vec<SubtitleBlock>,
}

impl TranscriptionStore {
    pub fn new(max_blocks: usize) -> Self {
        Self {
            blocks: VecDeque::with_capacity(max_blocks),
            interim_blocks: Vec::with_capacity(max_blocks),
            max_blocks,
            last_activity: None,
            completed: Vec::new(),
        }
    }

    pub fn update(&mut self, data: TranscriptData) {
        self.last_activity = Some(Instant::now());

        self.interim_blocks.clear();

        let speaker = data.speaker;
        let needs_new = match self.blocks.back() {
            None => true,
            Some(last) if last.speaker != speaker => true,
            Some(last) if last.text.chars().count() > SOFT_LIMIT_CHARS => {
                can_break_before(last, &data.text)
            }
            _ => false,
        };

        if needs_new {
            self.push_block(SubtitleBlock::new(speaker.clone()));
        }

        if let Some(block) = self.blocks.back_mut() {
            block.text.push_str(&data.text);
        }
    }

    pub fn update_interim(&mut self, segments: Vec<TranscriptData>) {
        self.interim_blocks.clear();
        if segments.is_empty() {
            return;
        }
        self.last_activity = Some(Instant::now());
        for seg in segments {
            let mut block = SubtitleBlock::new(seg.speaker);
            block.text = seg.text;
            self.interim_blocks.push(block);
        }
    }

    pub fn ensure_separator(&mut self) {
        for block in std::mem::take(&mut self.interim_blocks) {
            self.push_block(block);
        }

        self.pop_if_overflow();
        if let Some(block) = self.blocks.back_mut() {
            if block.text.is_empty() {
                return;
            }
            let trimmed_len = block.text.trim_end().len();
            block.text.truncate(trimmed_len);
            block.text.push_str(GAP_MARKER);
            self.last_activity = Some(Instant::now());
        }
    }

    pub fn max_blocks(&self) -> usize {
        self.max_blocks
    }

    pub fn pop_if_overflow(&mut self) {
        while self.blocks.len() > self.max_blocks {
            self.blocks.pop_front();
        }
    }

    pub fn resize(&mut self, new_max_blocks: usize) {
        self.max_blocks = new_max_blocks;
        self.pop_if_overflow();
    }

    pub fn last_activity(&self) -> Option<Instant> {
        self.last_activity
    }

    pub fn blocks(&self) -> impl Iterator<Item = &SubtitleBlock> {
        self.blocks.iter()
    }

    pub fn interim(&self) -> impl Iterator<Item = &SubtitleBlock> {
        self.interim_blocks.iter()
    }

    pub fn clear_if_silent(&mut self, timeout: Duration) {
        if let Some(last_activity) = self.last_activity
            && last_activity.elapsed() >= timeout
        {
            self.finish();
        }
    }

    pub fn finish(&mut self) {
        self.finish_last();
        self.blocks.clear();
        self.interim_blocks.clear();
        self.last_activity = None;
    }

    pub fn take_completed(&mut self) -> impl Iterator<Item = SubtitleBlock> + '_ {
        self.completed.drain(..)
    }

    fn finish_last(&mut self) {
        let Some(block) = self.blocks.back() else {
            return;
        };

        let text = block.text.trim();
        if text.is_empty() {
            return;
        }

        self.completed.push(SubtitleBlock {
            speaker: block.speaker.clone(),
            text: text.to_owned(),
        });
    }

    fn push_block(&mut self, block: SubtitleBlock) {
        self.finish_last();
        self.blocks.push_back(block);
        self.pop_if_overflow();
    }
}

/// Whether a new block may start before `incoming`.
///
/// Only at a seam: a break in the middle of a word would show up on screen as
/// one line ending mid-syllable and the next beginning mid-syllable. A space
/// on either side is a seam; so is a CJK character, where words are not spaced
/// and any boundary between glyphs will do. Punctuation on its own never
/// starts a line.
fn can_break_before(last: &SubtitleBlock, incoming: &str) -> bool {
    let trimmed = incoming.trim();
    if trimmed.is_empty() || is_punctuation_or_symbol(trimmed) {
        return false;
    }

    let last_char = last.text.chars().last().unwrap_or(' ');
    let first_char = incoming.chars().next().unwrap_or(' ');

    last_char.is_whitespace()
        || first_char.is_whitespace()
        || is_cjk(last_char)
        || is_cjk(first_char)
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
    let final_blocks = store.blocks().map(|b| (b, false));
    let interim_blocks = store.interim().map(|b| (b, true));
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

pub struct VisualReplica<'a> {
    pub speaker: Option<&'a str>,
    pub elements: Vec<TextElement<'a>>,
}
