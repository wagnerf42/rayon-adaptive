use crossbeam::atomic::AtomicCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct SmallChannel<T> {
    request: AtomicBool,
    data: AtomicCell<Option<T>>,
}

pub struct SmallSender<T> {
    channel: Arc<SmallChannel<T>>,
}

pub struct SmallReceiver<T> {
    channel: Arc<SmallChannel<T>>,
}

impl<T> SmallChannel<T> {
    fn new() -> Self {
        SmallChannel {
            request: AtomicBool::new(false),
            data: AtomicCell::new(None),
        }
    }
}

/// Communicate between threads like a channel but only once.
pub fn small_channel<T: Send>() -> (SmallSender<T>, SmallReceiver<T>) {
    let channel = Arc::new(SmallChannel::new());
    (
        SmallSender {
            channel: channel.clone(),
        },
        SmallReceiver { channel },
    )
}

impl<T> SmallReceiver<T> {
    pub fn recv(self) -> Option<T> {
        self.channel.request.store(true, Ordering::Relaxed);
        let mut channel = self.channel;
        loop {
            let r = Arc::try_unwrap(channel);
            match r {
                Ok(c) => return c.data.into_inner(),
                Err(ac) => channel = ac,
            }
        }
    }
}

impl<T> SmallSender<T> {
    /// Return whether receiver is blocking, waiting for something.
    pub fn receiver_is_waiting(&self) -> bool {
        self.channel.request.load(Ordering::Relaxed)
    }
    pub fn send(self, t: T) {
        self.channel.data.store(Some(t));
    }
}
