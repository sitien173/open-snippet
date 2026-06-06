//! Lock-free SPSC transport for hook events.

use rtrb::{Consumer, Producer, RingBuffer};

use super::HookEvent;

pub const RING_CAPACITY: usize = 1024;

pub struct HookProducer {
    inner: Producer<HookEvent>,
}

pub struct HookConsumer {
    inner: Consumer<HookEvent>,
}

pub fn channel(capacity: usize) -> (HookProducer, HookConsumer) {
    let (producer, consumer) = RingBuffer::new(capacity);
    (
        HookProducer { inner: producer },
        HookConsumer { inner: consumer },
    )
}

impl HookProducer {
    pub fn push(&mut self, event: HookEvent) -> bool {
        // SECURITY: hook character content is typed user input; redact unless verbose logging is explicitly enabled.
        match event {
            HookEvent::Char(ch) => tracing::debug!(
                kind = "char",
                ch = %crate::log_init::redact::redact_str(
                    &ch.to_string(),
                    crate::log_init::redact::FieldKind::FormValue
                ),
                "queued hook event"
            ),
            HookEvent::Backspace => tracing::debug!(kind = "backspace", "queued hook event"),
            HookEvent::Reset(cause) => {
                tracing::debug!(kind = "reset", cause = ?cause, "queued hook event")
            }
        }
        self.inner.push(event).is_ok()
    }
}

impl HookConsumer {
    pub fn pop(&mut self) -> Option<HookEvent> {
        self.inner.pop().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::channel;
    use crate::hook::{HookEvent, ResetCause};

    #[test]
    fn spsc_preserves_event_order() {
        let (mut producer, mut consumer) = channel(8);

        assert!(producer.push(HookEvent::Char('a')));
        assert!(producer.push(HookEvent::Backspace));
        assert!(producer.push(HookEvent::Reset(ResetCause::CapsToggle)));

        assert_eq!(consumer.pop(), Some(HookEvent::Char('a')));
        assert_eq!(consumer.pop(), Some(HookEvent::Backspace));
        assert_eq!(
            consumer.pop(),
            Some(HookEvent::Reset(ResetCause::CapsToggle))
        );
        assert_eq!(consumer.pop(), None);
    }

    #[test]
    fn overflow_drops_newest_event() {
        let (mut producer, mut consumer) = channel(2);

        assert!(producer.push(HookEvent::Char('a')));
        assert!(producer.push(HookEvent::Char('b')));
        assert!(!producer.push(HookEvent::Char('c')));

        assert_eq!(consumer.pop(), Some(HookEvent::Char('a')));
        assert_eq!(consumer.pop(), Some(HookEvent::Char('b')));
        assert_eq!(consumer.pop(), None);
    }
}
