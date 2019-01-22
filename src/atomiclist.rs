use crossbeam::atomic::AtomicCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
pub type AtomicLink<T> = Arc<AtomicNode<T>>;

pub struct AtomicNode<T> {
    request: AtomicBool,
    content: AtomicCell<Option<T>>,
    next: AtomicCell<Option<AtomicLink<T>>>,
}
impl<T> AtomicNode<T> {
    pub fn new(content: T) -> Self {
        AtomicNode {
            request: AtomicBool::new(false),
            content: AtomicCell::new(Some(content)),
            next: AtomicCell::new(None),
        }
    }

    pub fn take(&self) -> Option<T> {
        self.content.swap(None)
    }

    pub fn split(&self, content: T) -> Arc<Self> {
        let new_node = Arc::new(AtomicNode::new(content));
        let next_node = self.next.swap(None);
        new_node.next.swap(next_node);
        self.next.swap(Some(new_node.clone()));
        new_node
    }
}

pub struct AtomicList<T> {
    head: Option<AtomicLink<T>>,
}

impl<T> Iterator for AtomicList<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.head.is_some() {
            let mut head = self.head.take().unwrap();
            head.request.store(true, Ordering::SeqCst);
            let node = {
                loop {
                    match Arc::try_unwrap(head) {
                        Ok(real_node) => break real_node,
                        Err(failed) => head = failed,
                    }
                }
            };
            self.head = node.next.swap(None);
            node.take()
        } else {
            None
        }
    }
}

impl<T> AtomicList<T> {
    pub fn new() -> Self {
        AtomicList { head: None }
    }
    pub fn push_front(&mut self, content: T) {
        let new_node = Arc::new(AtomicNode::new(content));
        let next_node = self.head.take();
        new_node.next.swap(next_node);
        self.head = Some(new_node);
    }
}
