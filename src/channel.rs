use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::task::Waker;

/*
    The channel module provides a way to communicate between tasks.
    Currently this is a single-producer, single-consumer channel.
*/

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let queue = Arc::new(Mutex::new(VecDeque::new()));
    let waker = Arc::new(Mutex::new(None));
    let sender: Sender<T> = Sender {
        queue: Arc::clone(&queue),
        waker: Arc::clone(&waker),
    };
    // The queue is owned by the receiver, so when the receiver is dropped, the queue is dropped
    let receiver: Receiver<T> = Receiver { queue, waker };
    (sender, receiver)
}

pub struct Sender<T> {
    queue: Arc<Mutex<VecDeque<T>>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(value);

        if let Some(waker) = self.waker.lock().unwrap().take() {
            waker.wake();
        }
    }
}

pub struct Receiver<T> {
    queue: Arc<Mutex<VecDeque<T>>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl<T> Receiver<T> {
    pub fn try_recv(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();
        queue.pop_front()
    }

    pub fn register_waker(&self, waker: Waker) {
        let mut waker_slot = self.waker.lock().unwrap();
        *waker_slot = Some(waker);
    }
}