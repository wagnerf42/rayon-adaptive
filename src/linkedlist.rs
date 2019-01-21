use crate::prelude::*;
//use std::sync::mpsc::{channel, Receiver, Sender};
//use crate::smallchannel::SmallChannel;
use crossbeam::atomic::AtomicCell;
//use rayon_core::current_thread_index;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
type ThreadId = usize;
pub type Link<O2, I> = Arc<Node<O2, I>>;

pub struct Node<O2, I> {
    id: ThreadId,
    output: Arc<AtomicCell<Option<O2>>>,
    input: Arc<AtomicCell<Option<I>>>,
    next: AtomicCell<Option<Link<O2, I>>>,
}
impl<O2, I> Node<O2, I>
where
    I: Divisible,
{
    pub fn store_input(&self, input: I) {
        debug_assert!(input.base_length() > 0);
        self.input.swap(Some(input));
    }
    pub fn take_input(&self) -> Option<I> {
        self.input.swap(None)
    }
    pub fn store_output(&self, output: O2) {
        self.output.swap(Some(output));
    }
    pub fn take_output(&self) -> Option<O2> {
        self.output.swap(None)
    }
    pub fn new(id: ThreadId) -> Self {
        Node {
            id,
            output: Arc::new(AtomicCell::new(None)),
            input: Arc::new(AtomicCell::new(None)),
            next: AtomicCell::new(None),
        }
    }
    pub fn split(&self, id: ThreadId, slave_input: I) -> Arc<Self> {
        debug_assert!(slave_input.base_length() > 0);
        let new_node = Arc::new(Node::new(id));
        new_node.store_input(slave_input);
        let new_node_clone = new_node.clone();
        let split_me_successor = self.next.swap(None);
        new_node.next.swap(split_me_successor);
        self.next.swap(Some(new_node));
        new_node_clone
    }
}

pub struct LinkedList<O2, I, RET> {
    node: AtomicCell<Option<Link<O2, I>>>,
    retrieve_closure: RET,
    input: Option<I>,
}
impl<O2, I, RET> LinkedList<O2, I, RET>
where
    I: Divisible,
    O2: Send + Sync + Sized,
{
    // Consume the list only as long as you have only O2s in the nodes. As soon as you encounter O2,
    // I(non-empty), return a head node along with the O1 that you (may) have generated in this function.
    // The head NEVER contains an input!
    pub fn new(input: I, retrieve_closure: RET) -> Self {
        debug_assert!(input.base_length() > 0);
        LinkedList {
            node: AtomicCell::new(None),
            retrieve_closure,
            input: Some(input),
        }
    }
    pub fn remaining_input_length(&self) -> usize {
        if self.input.is_none() {
            0
        } else {
            self.input.as_ref().unwrap().base_length()
        }
    }
    pub fn take_input(&mut self) -> Option<I> {
        self.input.take()
    }
    pub fn store_input(&mut self, input: I) {
        debug_assert!(input.base_length() > 0);
        self.input.replace(input);
    }
    pub fn push_node(&self, slave_input: I, slave_id: ThreadId) -> Link<O2, I> {
        let slave_node = Arc::new(Node::new(slave_id));
        slave_node.store_input(slave_input);
        let current_node = self.node.swap(None);
        slave_node.next.swap(current_node);
        let slave_clone = slave_node.clone();
        self.node.swap(Some(slave_node));
        slave_clone
    }
    pub fn start_retrieve<O1>(
        mut self,
        processed_output: O1,
        vector: Arc<Vec<AtomicBool>>,
    ) -> (O1, Self)
    where
        RET: Fn(O1, O2) -> O1 + Sync + Copy,
        O1: Send + Sync,
    {
        let mut partial_output = processed_output;
        let ret_fn = self.retrieve_closure;
        loop {
            let iter_node = self.node.swap(None);
            if iter_node.is_none() {
                break;
            }
            let iter_node = iter_node.unwrap();
            //Pessimistically signal it and spinlock on the option.
            vector[iter_node.id].store(true, Ordering::SeqCst);
            let unwrapped_node;
            match Arc::try_unwrap(iter_node) {
                Ok(task_node) => {
                    unwrapped_node = task_node;
                }
                Err(mut arc_node) => loop {
                    let temp = Arc::try_unwrap(arc_node);
                    if let Ok(inner_stuff) = temp {
                        unwrapped_node = inner_stuff;
                        break;
                    } else {
                        arc_node = temp.err().unwrap();
                    }
                },
            }
            let his_output = unwrapped_node.output.swap(None);
            debug_assert!(his_output.is_some());
            partial_output = ret_fn(partial_output, his_output.unwrap());
            let maybe_next_node = unwrapped_node.next.swap(None);
            self.node.swap(maybe_next_node);
            let mut his_input = unwrapped_node.input.swap(None);
            if his_input.is_some() {
                debug_assert!(his_input.as_ref().unwrap().base_length() > 0);
                self.input = his_input.take();
                break;
            }
            //iter_link = task_node.next.swap(None);
        }
        (partial_output, self)
    }
}
