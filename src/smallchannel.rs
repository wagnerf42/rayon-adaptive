use crossbeam::atomic::AtomicCell;
use std::sync::Arc;

pub struct SmallSender<T> {
    data: Arc<AtomicCell<Option<T>>>,
}

pub struct SmallReceiver<T> {
    data: Arc<AtomicCell<Option<T>>>,
}

/// Communicate between threads like a channel but only once.
pub fn small_channel<T: Send>() -> (SmallSender<T>, SmallReceiver<T>) {
    let data = Arc::new(AtomicCell::new(None));
    (SmallSender { data: data.clone() }, SmallReceiver { data })
}

impl<T> SmallReceiver<T> {
    pub fn recv(self) -> Option<T> {
        let mut data = self.data;
        loop {
            let r = Arc::try_unwrap(data);
            match r {
                Ok(cell) => return cell.into_inner(),
                Err(still_data) => data = still_data,
            }
        }
    }
}

impl<T> SmallSender<T> {
    pub fn send(self, t: T) {
        self.data.store(Some(t));
    }
}
