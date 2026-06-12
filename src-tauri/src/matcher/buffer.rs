//! Rolling character buffer used by the matcher.

use std::collections::VecDeque;

use super::{classify_boundary_char, BoundaryState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reset {
    ArrowKey,
    Home,
    End,
    PageUp,
    PageDown,
    ImeCompositionStart,
    CapsLockToggled,
    FocusChanged,
}

#[derive(Debug, Clone)]
pub struct MatchBuffer {
    capacity: usize,
    chars: VecDeque<char>,
    leading_boundary_state: BoundaryState,
    current_boundary_state: BoundaryState,
}

impl MatchBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            chars: VecDeque::with_capacity(capacity),
            leading_boundary_state: BoundaryState::StartOfBuffer,
            current_boundary_state: BoundaryState::StartOfBuffer,
        }
    }

    pub fn push_char(&mut self, ch: char) {
        self.current_boundary_state = self
            .chars
            .back()
            .copied()
            .map(classify_boundary_char)
            .unwrap_or(self.leading_boundary_state);

        if self.chars.len() == self.capacity {
            if let Some(removed) = self.chars.pop_front() {
                self.leading_boundary_state = classify_boundary_char(removed);
            }
        }
        self.chars.push_back(ch);
    }

    pub fn pop_char(&mut self) -> Option<char> {
        self.chars.pop_back()
    }

    pub fn reset(&mut self) {
        self.chars.clear();
        self.leading_boundary_state = BoundaryState::StartOfBuffer;
        self.current_boundary_state = BoundaryState::StartOfBuffer;
    }

    pub fn reset_with(&mut self, _reason: Reset) {
        self.reset();
    }

    pub fn as_str(&self) -> String {
        self.chars.iter().collect()
    }

    pub fn boundary_state(&self) -> BoundaryState {
        self.current_boundary_state
    }

    pub fn boundary_char_state(&self, start_index: usize) -> BoundaryState {
        if start_index == 0 {
            self.current_boundary_state
        } else {
            self.chars
                .get(start_index.saturating_sub(1))
                .copied()
                .map(classify_boundary_char)
                .unwrap_or(self.leading_boundary_state)
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.chars.len()
    }

    pub(crate) fn char_at(&self, index: usize) -> Option<char> {
        self.chars.get(index).copied()
    }

    pub(crate) fn leading_boundary_state(&self) -> BoundaryState {
        self.leading_boundary_state
    }
}
