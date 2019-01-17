use crossbeam::atomic::AtomicCell;
use std::sync::Arc;

pub struct SmallChannel<T> {
    data: Arc<AtomicCell<Option<T>>>,
}

impl<T> SmallChannel<T> {
    pub fn new() -> (Self, Self) {
        let data = Arc::new(AtomicCell::new(None));
        let data_clone = data.clone();
        (SmallChannel { data }, SmallChannel { data: data_clone })
    }
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
    pub fn send(self, t: T) {
        self.data.store(Some(t));
    }
    pub fn close_channel(self) {
        self.data.store(None);
    }
}
