use crate::stt::data::TranscriptData;
use crate::transcription::subtitles::SubtitleBlock;
use crate::transcription::utils::{is_cjk, is_punctuation_or_symbol};
use eframe::egui::Context;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct TranscriptionStore {
    pub blocks: VecDeque<SubtitleBlock>,
    pub interim_blocks: Vec<SubtitleBlock>,
    max_blocks: usize,
    last_activity: Option<Instant>,
}

impl TranscriptionStore {
    pub fn new(max_blocks: usize) -> Self {
        Self {
            blocks: VecDeque::with_capacity(max_blocks),
            interim_blocks: Vec::with_capacity(max_blocks),
            max_blocks,
            last_activity: None,
        }
    }

    pub fn update(&mut self, data: TranscriptData) {
        self.last_activity = Some(Instant::now());

        if data.is_final {
            self.interim_blocks.clear();

            let speaker = data.speaker;
            let needs_new = match self.blocks.back() {
                Some(last) if last.speaker != speaker => true,
                Some(last) if last.text.len() > 200 => {
                    let trimmed = data.text.trim();
                    if trimmed.is_empty() || is_punctuation_or_symbol(trimmed) {
                        false
                    } else {
                        let last_char = last.text.chars().last().unwrap_or(' ');
                        let first_char = data.text.chars().next().unwrap_or(' ');

                        let is_space_boundary =
                            last_char.is_whitespace() || first_char.is_whitespace();
                        let is_cjk_boundary = is_cjk(last_char) || is_cjk(first_char);

                        is_space_boundary || is_cjk_boundary
                    }
                }
                None => true,
                _ => false,
            };

            if needs_new {
                self.blocks.push_back(SubtitleBlock::new(speaker.clone()));
                self.pop_if_overflow();
            }

            if let Some(block) = self.blocks.back_mut() {
                block.text.push_str(&data.text);
            }
        } else {
            let mut new_block = SubtitleBlock::new(data.speaker);
            new_block.text = data.text;
            self.interim_blocks = vec![new_block];
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
        for block in self.interim_blocks.drain(..) {
            let mut new_block = SubtitleBlock::new(block.speaker);
            new_block.text = block.text;
            self.blocks.push_back(new_block);
        }

        self.pop_if_overflow();
        if let Some(block) = self.blocks.back_mut() {
            if block.text.is_empty() {
                return;
            }
            let trimmed_len = block.text.trim_end().len();
            block.text.truncate(trimmed_len);
            block.text.push_str("...    ");
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

    pub fn clear_if_silent(&mut self, timeout: Duration) {
        if let Some(last_activity) = self.last_activity
            && last_activity.elapsed() >= timeout
        {
            self.blocks.clear();
            self.interim_blocks.clear();
            self.last_activity = None;
        }
    }

    pub fn schedule(&mut self, ctx: Context, timeout: Duration) {
        if let Some(last_activity) = self.last_activity() {
            let elapsed = last_activity.elapsed();
            if elapsed < timeout {
                ctx.request_repaint_after(timeout - elapsed);
            }
        }
    }
}
