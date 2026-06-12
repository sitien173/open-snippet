use openmacro_lib::matcher::{MatchBuffer, Matcher};
use openmacro_lib::store::Snippet;
use std::path::PathBuf;

#[test]
fn test_matcher_enter_burst() {
    let mut matcher = Matcher::new();
    let snippet = Snippet {
        id: "test_id".to_string(),
        trigger: "/test".to_string(),
        raw_trigger: "/test".to_string(),
        trigger_literal: false,
        replace: "replace".to_string(),
        vars: vec![],
        source_file: PathBuf::from("test.yaml"),
    };
    matcher.rebuild(&[snippet]).unwrap();
    let mut buffer = MatchBuffer::new(64);

    for ch in "some words".chars() {
        matcher.on_char(&mut buffer, ch);
    }

    // Simulate 10 Enters
    for _ in 0..10 {
        matcher.on_char(&mut buffer, '\r');
    }

    // Type the trigger
    let mut hit = None;
    for ch in "/test".chars() {
        hit = matcher.on_char(&mut buffer, ch);
    }

    assert!(hit.is_some(), "Trigger should match after burst of enters");
    assert_eq!(hit.unwrap().snippet_id.as_ref(), "test_id");
}
