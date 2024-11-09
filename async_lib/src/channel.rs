use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use crate::future::SimpleFuture;
use crate::task::{State, Context, Waker};

// Currently this is a single thread implementation so the channel is not really neccessary, it is only to implement the concept
pub struct Channel<T> {
    buffer: Arc<Mutex<VecDeque<T>>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Channel {
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            waker: Arc::new(Mutex::new(None)),
        }
    }

    pub fn sender(&self) -> Sender<T> {
        Sender {
            buffer: self.buffer.clone(),
            waker: self.waker.clone(),
        }
    }

    pub fn receiver(&self) -> Receiver<T> {
        Receiver {
            buffer: self.buffer.clone(),
            waker: self.waker.clone(),
        }
    }
}

pub struct Sender<T> {
    buffer: Arc<Mutex<VecDeque<T>>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) {
        let mut buffer = self.buffer.lock().unwrap(); // lock the buffer to push the value
        buffer.push_back(value);

        println!("Sender: Value pushed to buffer");

        let mut waker_slot = self.waker.lock().unwrap();
        if let Some(waker) = waker_slot.take() {
            println!("Sender: Waking up task ...");
            waker.wake();
        } else {
            println!("Sender: No waker to wake up");
        }
    }
}

pub struct Receiver<T> {
    buffer: Arc<Mutex<VecDeque<T>>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl<T> SimpleFuture for Receiver<T> {
    type Output = T;

    fn poll(&mut self, ctx: &Context) -> State<Self::Output> {
        println!("Receiver: Polling ...");
        let mut buffer = self.buffer.lock().unwrap(); // lock the buffer to pop the value
        if let Some(value) = buffer.pop_front() {
            println!("Receiver: value found in buffer");
            State::Ready(value)
        } else {
            println!("Receiver: No value found, setting up waker ...");
            let mut waker_slot = self.waker.lock().unwrap();
            if let Some(waker) = ctx.waker() {
                println!("Receiver: Setting waker ...");
                *waker_slot = Some(waker.clone());
            } else {
                println!("Receiver: No waker found in Context");
            }
            State::Pending
        }
    }
}