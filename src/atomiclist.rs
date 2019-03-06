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

    pub fn requested(&self) -> bool {
        self.request.load(Ordering::Relaxed)
    }

    pub fn take(&self) -> Option<T> {
        self.content.swap(None)
    }

    pub fn replace(&self, new_content: T) {
        self.content.store(Some(new_content))
    }

    pub fn split(&self, content: T) -> Arc<Self> {
        let new_node = Arc::new(AtomicNode::new(content));
        let next_node = self.next.swap(None);
        new_node.next.swap(next_node);
        self.next.swap(Some(new_node.clone()));
        new_node
    }
}

#[derive(Default)]
pub struct AtomicList<T> {
    head: AtomicCell<Option<AtomicLink<T>>>,
}

impl<'a, T> Iterator for AtomicListIterator<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let possible_head = self.list.head.swap(None);
        if possible_head.is_some() {
            let mut head = possible_head.unwrap();
            head.request.store(true, Ordering::SeqCst);
            let node: AtomicNode<T>;
            #[cfg(feature = "logs")]
            {
                node = rayon_logs::subgraph("wait retrieving", 1, || loop {
                    match Arc::try_unwrap(head) {
                        Ok(real_node) => break real_node,
                        Err(failed) => head = failed,
                    }
                })
            }
            #[cfg(not(feature = "logs"))]
            {
                node = loop {
                    match Arc::try_unwrap(head) {
                        Ok(real_node) => break real_node,
                        Err(failed) => head = failed,
                    }
                }
            }
            self.list.head.swap(node.next.swap(None));
            node.take()
        } else {
            None
        }
    }
}

pub struct AtomicListIterator<'a, T> {
    list: &'a AtomicList<T>,
}

impl<T> AtomicList<T> {
    pub fn new() -> Self {
        AtomicList {
            head: AtomicCell::new(None),
        }
    }
    pub fn iter(&self) -> AtomicListIterator<T> {
        AtomicListIterator { list: self }
    }
    pub fn push_front(&self, content: T) -> AtomicLink<T> {
        let new_node = Arc::new(AtomicNode::new(content));
        let next_node = self.head.swap(None);
        new_node.next.swap(next_node);
        self.head.swap(Some(new_node.clone()));
        new_node
    }
}
