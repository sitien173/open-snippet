use serde_json::json;
use std::sync::Arc;
use tracing::Level;
use tracing_subscriber::prelude::*;

use openmacro_lib::log_init::ring::{LogEntry, RingBuffer, RingLayer};

fn entry(message: String) -> LogEntry {
    LogEntry {
        seq: 0,
        ts_unix_ms: 0,
        level: Level::INFO,
        target: "test".to_string(),
        message,
        fields: json!({}),
        span_path: Vec::new(),
    }
}

#[test]
fn ring_buffer_caps_at_capacity_and_slices_by_sequence() {
    let ring = RingBuffer::new(2000);

    for index in 0..2100 {
        ring.push(entry(format!("event-{index}")));
    }

    let entries = ring.slice_since(0);
    assert_eq!(entries.len(), 2000);
    assert_eq!(entries[0].message, "event-100");
    assert_eq!(entries[1999].message, "event-2099");

    for pair in entries.windows(2) {
        assert!(pair[0].seq < pair[1].seq);
    }

    let seq_at_index_500 = entries[500].seq;
    let after = ring.slice_since(seq_at_index_500);
    assert_eq!(after.len(), 1499);
    assert_eq!(after[0].message, "event-601");
    assert_eq!(after[1498].message, "event-2099");
}

#[test]
fn ring_layer_captures_event_metadata_fields_and_spans() {
    let ring = Arc::new(RingBuffer::new(2000));
    let subscriber = tracing_subscriber::registry().with(RingLayer::new(Arc::clone(&ring)));

    tracing::subscriber::with_default(subscriber, || {
        let span = tracing::info_span!("outer");
        let _entered = span.enter();
        tracing::warn!(answer = 42_u64, "captured message");
    });

    let entries = ring.slice_since(0);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].level, Level::WARN);
    assert_eq!(entries[0].target, "log_init_ring");
    assert_eq!(entries[0].message, "captured message");
    assert_eq!(entries[0].fields["answer"], 42);
    assert_eq!(entries[0].span_path, vec!["outer".to_string()]);
}
