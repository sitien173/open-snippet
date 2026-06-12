use std::path::PathBuf;

use openmacro_lib::{
    matcher::{BoundaryState, MatchBuffer, Matcher, Reset},
    store::Snippet,
};

fn snippet(trigger: &str, id: &str) -> Snippet {
    Snippet {
        id: id.to_string(),
        trigger: trigger.to_string(),
        raw_trigger: trigger.to_string(),
        trigger_literal: false,
        replace: format!("replace:{id}"),
        vars: Vec::new(),
        source_file: PathBuf::from("test.yaml"),
    }
}

#[test]
fn buffer_push_pop_and_reset_work() {
    let mut buffer = MatchBuffer::new(64);

    buffer.push_char(';');
    buffer.push_char('s');
    buffer.push_char('i');
    buffer.push_char('g');

    assert_eq!(buffer.as_str(), ";sig");

    buffer.pop_char();
    assert_eq!(buffer.as_str(), ";si");

    buffer.reset_with(Reset::FocusChanged);
    assert_eq!(buffer.as_str(), "");
    assert_eq!(buffer.boundary_state(), BoundaryState::StartOfBuffer);
}

#[test]
fn boundary_state_tracks_previous_char_when_buffer_rolls() {
    let mut buffer = MatchBuffer::new(4);

    buffer.push_char('a');
    buffer.push_char('b');
    buffer.push_char('c');
    buffer.push_char('d');
    buffer.push_char(' ');

    assert_eq!(buffer.as_str(), "bcd ");
    assert_eq!(buffer.boundary_state(), BoundaryState::Other);
}

#[test]
fn punctuation_and_whitespace_are_boundary_states() {
    let mut whitespace_buffer = MatchBuffer::new(64);
    whitespace_buffer.push_char(' ');
    whitespace_buffer.push_char(';');
    assert_eq!(
        whitespace_buffer.boundary_char_state(0),
        BoundaryState::Whitespace
    );

    let mut punctuation_buffer = MatchBuffer::new(64);
    punctuation_buffer.push_char('(');
    punctuation_buffer.push_char(';');
    assert_eq!(
        punctuation_buffer.boundary_char_state(0),
        BoundaryState::Punctuation
    );
}

#[test]
fn all_reset_events_clear_the_buffer() {
    let resets = [
        Reset::ArrowKey,
        Reset::Home,
        Reset::End,
        Reset::PageUp,
        Reset::PageDown,
        Reset::ImeCompositionStart,
        Reset::CapsLockToggled,
        Reset::FocusChanged,
    ];

    for reset in resets {
        let mut buffer = MatchBuffer::new(64);
        buffer.push_char(';');
        buffer.push_char('x');
        buffer.reset_with(reset);
        assert_eq!(buffer.as_str(), "", "reset {reset:?} should clear buffer");
    }
}

#[test]
#[tracing_test::traced_test]
fn longest_match_wins() {
    let mut matcher = Matcher::new();
    matcher
        .rebuild(&[snippet(";sig", "short"), snippet(";signature", "long")])
        .unwrap();
    let mut buffer = MatchBuffer::new(64);
    let mut hit = None;

    for ch in ";signature".chars() {
        hit = matcher.on_char(&mut buffer, ch);
    }

    let hit = hit.expect("expected longest match");
    assert_eq!(hit.snippet_id.as_ref(), "long");
    assert_eq!(hit.trigger_len_chars, 10);
}

#[test]
fn word_boundary_is_required() {
    let mut matcher = Matcher::new();
    matcher.rebuild(&[snippet(";sig", "sig")]).unwrap();
    let mut buffer = MatchBuffer::new(64);
    let mut hit = None;

    for ch in "aa;sig".chars() {
        hit = matcher.on_char(&mut buffer, ch);
    }

    assert!(hit.is_none());
}

#[test]
fn backspace_pop_reverts_a_near_miss() {
    let mut matcher = Matcher::new();
    matcher.rebuild(&[snippet(";sig", "sig")]).unwrap();
    let mut buffer = MatchBuffer::new(64);

    for ch in ";six".chars() {
        assert!(matcher.on_char(&mut buffer, ch).is_none());
    }

    buffer.pop_char();
    let hit = matcher
        .on_char(&mut buffer, 'g')
        .expect("expected recovered match");

    assert_eq!(hit.snippet_id.as_ref(), "sig");
    assert_eq!(hit.trigger_len_chars, 4);
}

#[test]
fn full_reset_clears_buffer_for_matching() {
    let mut matcher = Matcher::new();
    matcher.rebuild(&[snippet(";sig", "sig")]).unwrap();
    let mut buffer = MatchBuffer::new(64);

    for ch in ";si".chars() {
        assert!(matcher.on_char(&mut buffer, ch).is_none());
    }

    buffer.reset_with(Reset::FocusChanged);

    assert!(matcher.on_char(&mut buffer, 'g').is_none());
}

#[test]
fn multibyte_utf8_trigger_matches_by_char_count() {
    let mut matcher = Matcher::new();
    matcher.rebuild(&[snippet(";π", "pi")]).unwrap();
    let mut buffer = MatchBuffer::new(64);
    let mut hit = None;

    for ch in ";π".chars() {
        hit = matcher.on_char(&mut buffer, ch);
    }

    let hit = hit.expect("expected UTF-8 trigger match");
    assert_eq!(hit.snippet_id.as_ref(), "pi");
    assert_eq!(hit.trigger_len_chars, 2);
}
