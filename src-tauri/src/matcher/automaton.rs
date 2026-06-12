//! Aho-Corasick matcher implementation.

use std::sync::Arc;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};

use crate::store::Snippet;

use super::{classify_boundary_char, BoundaryState, MatchBuffer};

const MAX_BUFFER_CHARS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchHit {
    pub snippet_id: Arc<str>,
    pub trigger_len_chars: usize,
}

#[derive(Default)]
pub struct Matcher {
    automaton: Option<AhoCorasick>,
    patterns: Vec<PatternMeta>,
    scratch: String,
}

#[derive(Debug, Clone)]
struct PatternMeta {
    snippet_id: Arc<str>,
    trigger_len_chars: usize,
}

impl Matcher {
    pub fn new() -> Self {
        Self {
            automaton: None,
            patterns: Vec::new(),
            scratch: String::with_capacity(MAX_BUFFER_CHARS * 4),
        }
    }

    #[tracing::instrument(skip(self, snippets), fields(snippet_count = snippets.len()))]
    pub fn rebuild(&mut self, snippets: &[Snippet]) -> Result<(), aho_corasick::BuildError> {
        tracing::debug!(snippet_count = snippets.len(), "rebuilding matcher");
        self.patterns.clear();
        if snippets.is_empty() {
            self.automaton = None;
            return Ok(());
        }

        let patterns: Vec<&str> = snippets
            .iter()
            .map(|snippet| snippet.trigger.as_str())
            .collect();
        self.patterns = snippets
            .iter()
            .map(|snippet| PatternMeta {
                snippet_id: Arc::<str>::from(snippet.id.clone()),
                trigger_len_chars: snippet.trigger.chars().count(),
            })
            .collect();
        self.automaton = Some(
            AhoCorasickBuilder::new()
                .match_kind(MatchKind::LeftmostLongest)
                .build(patterns)?,
        );
        tracing::debug!(pattern_count = self.patterns.len(), "matcher rebuilt");
        Ok(())
    }

    #[tracing::instrument(skip(self, buffer, ch), fields(buffer_len = buffer.len()))]
    pub fn on_char(&mut self, buffer: &mut MatchBuffer, ch: char) -> Option<MatchHit> {
        buffer.push_char(ch);

        let automaton = self.automaton.as_ref()?;
        let buffer_len = buffer.len();
        if buffer_len == 0 {
            return None;
        }

        let mut byte_offsets = [0usize; MAX_BUFFER_CHARS + 1];
        self.scratch.clear();
        for (index, offset_slot) in byte_offsets.iter_mut().enumerate().take(buffer_len) {
            *offset_slot = self.scratch.len();
            if let Some(ch) = buffer.char_at(index) {
                self.scratch.push(ch);
            }
        }
        byte_offsets[buffer_len] = self.scratch.len();

        let haystack = self.scratch.as_str();
        let mut best_hit = None;
        for mat in automaton.find_iter(haystack) {
            if mat.end() != haystack.len() {
                continue;
            }

            let Some(start_char_index) =
                byte_start_to_char_index(&byte_offsets, buffer_len, mat.start())
            else {
                continue;
            };

            let boundary_state = if start_char_index == 0 {
                buffer.leading_boundary_state()
            } else {
                buffer
                    .char_at(start_char_index - 1)
                    .map(classify_boundary_char)
                    .unwrap_or(buffer.leading_boundary_state())
            };

            if !boundary_allows_match(boundary_state) {
                continue;
            }

            let pattern = &self.patterns[mat.pattern().as_usize()];
            let candidate = MatchHit {
                snippet_id: pattern.snippet_id.clone(),
                trigger_len_chars: pattern.trigger_len_chars,
            };

            if best_hit
                .as_ref()
                .map(|hit: &MatchHit| candidate.trigger_len_chars > hit.trigger_len_chars)
                .unwrap_or(true)
            {
                best_hit = Some(candidate);
            }
        }

        if let Some(hit) = &best_hit {
            tracing::debug!(
                snippet_id = %hit.snippet_id,
                trigger_len_chars = hit.trigger_len_chars,
                "trigger matched"
            );
        }

        best_hit
    }
}

fn byte_start_to_char_index(
    byte_offsets: &[usize; MAX_BUFFER_CHARS + 1],
    char_count: usize,
    byte_start: usize,
) -> Option<usize> {
    (0..char_count).find(|index| byte_offsets[*index] == byte_start)
}

fn boundary_allows_match(state: BoundaryState) -> bool {
    matches!(
        state,
        BoundaryState::StartOfBuffer | BoundaryState::Whitespace | BoundaryState::Punctuation
    )
}
