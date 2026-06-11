//! Lock-free SPSC transport for hook events.

use std::{
    sync::mpsc::{self, Receiver, SyncSender},
    time::Duration,
};

use rtrb::{Consumer, Producer, RingBuffer};

use super::{ConfirmKey, HookEvent};

pub const RING_CAPACITY: usize = 1024;

pub struct HookProducer {
    inner: Producer<HookEvent>,
    wake_tx: SyncSender<()>,
}

pub struct HookConsumer {
    inner: Consumer<HookEvent>,
    wake_rx: Receiver<()>,
}

pub fn channel(capacity: usize) -> (HookProducer, HookConsumer) {
    let (producer, consumer) = RingBuffer::new(capacity);
    let (wake_tx, wake_rx) = mpsc::sync_channel(capacity);
    (
        HookProducer {
            inner: producer,
            wake_tx,
        },
        HookConsumer {
            inner: consumer,
            wake_rx,
        },
    )
}

impl HookProducer {
    pub fn push(&mut self, event: HookEvent) -> bool {
        if tracing::enabled!(tracing::Level::DEBUG) {
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
                HookEvent::Confirm(key) => tracing::debug!(
                    kind = "confirm",
                    key = match key {
                        ConfirmKey::Tab => "tab",
                        ConfirmKey::Enter => "enter",
                    },
                    "queued hook event"
                ),
                HookEvent::Reset(cause) => {
                    tracing::debug!(kind = "reset", cause = ?cause, "queued hook event")
                }
            }
        }
        let queued = self.inner.push(event).is_ok();
        if queued {
            let _ = self.wake_tx.try_send(());
        }
        queued
    }
}

impl HookConsumer {
    pub fn pop(&mut self) -> Option<HookEvent> {
        self.inner.pop().ok()
    }

    pub fn wait_timeout(&mut self, timeout: Duration) -> Option<HookEvent> {
        if let Some(event) = self.pop() {
            return Some(event);
        }

        while self.wake_rx.try_recv().is_ok() {}
        if let Some(event) = self.pop() {
            return Some(event);
        }

        self.wake_rx.recv_timeout(timeout).ok()?;
        self.pop()
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

    #[test]
    fn wait_timeout_wakes_for_queued_event() {
        let (mut producer, mut consumer) = channel(2);

        assert!(producer.push(HookEvent::Char('a')));

        assert_eq!(
            consumer.wait_timeout(std::time::Duration::from_millis(1)),
            Some(HookEvent::Char('a'))
        );
    }
}
