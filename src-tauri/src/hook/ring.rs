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
        assert_eq!(consumer.pop(), Some(HookEvent::Reset(ResetCause::CapsToggle)));
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
