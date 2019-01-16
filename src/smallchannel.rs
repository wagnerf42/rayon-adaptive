use crossbeam::atomic::AtomicCell;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct SmallChannel<T> {
    data: Arc<AtomicCell<Option<T>>>,
}

impl<T> SmallChannel<T> {
    fn new() -> Self {
        SmallChannel {
            data: Arc::new(AtomicCell::new(None)),
        }
    }
    fn recv(self) -> T {
        let mut data = self.data;
        loop {
            let r = Arc::try_unwrap(data);
            match r {
                Ok(cell) => return cell.into_inner().unwrap(),
                Err(still_data) => data = still_data,
            }
        }
    }
    fn send(&self, t: T) {
        self.data.store(Some(t));
    }
}
