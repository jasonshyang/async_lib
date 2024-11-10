use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use crate::future::SimpleFuture;
use crate::task::{State, Context, Waker};

pub struct Channel<T: Send> { // TODO: to be improved with more features, how to handle multiple receivers, etc.
    buffer: Arc<Mutex<VecDeque<T>>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl<T: Send> Channel<T> {
    // Create a new instance of the channel with an empty buffer and a None waker
    // The buffer is used to store the values sent by the sender
    // The waker is used to wake up the receiver when a value is sent
    // The waker is stored in the channel so that the sender can wake up the receiver when a value is sent
    // The waker and the buffer are stored in an Arc+Mutex to allow for shared ownership and thread safety
    pub fn new() -> Self {
        Channel {
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            waker: Arc::new(Mutex::new(None)),
        }
    }

    // Create a new sender for the channel
    // Clone the pointer to the buffer and the waker to allow for shared ownership
    pub fn sender(&self) -> Sender<T> {
        Sender {
            buffer: self.buffer.clone(),
            waker: self.waker.clone(),
        }
    }

    // Create a new receiver for the channel
    // Clone the pointer to the buffer and the waker to allow for shared ownership
    pub fn receiver(&self) -> Receiver<T> {
        Receiver {
            buffer: self.buffer.clone(),
            waker: self.waker.clone(),
        }
    }
}

// The Sender struct is used to send values via the channel
pub struct Sender<T: Send> {
    buffer: Arc<Mutex<VecDeque<T>>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl<T: Send + std::fmt::Debug> Sender<T> {
    // Send a value to the channel by pushing it to the buffer
    // Use the waker to wake up the receiver when a value is sent
    pub fn send(&self, value: T) {
        let mut buffer = self.buffer.lock().unwrap(); // lock the buffer to push the value
        println!("[Async_lib][Sender][send] Value pushed to buffer: {:?}", value);
        buffer.push_back(value);
        // Check if there is a waker to wake up, if so wake up the receiver
        let mut waker_slot = self.waker.lock().unwrap();
        if let Some(waker) = waker_slot.take() {
            println!("[Async_lib][Sender][send] Waking up receiver ...");
            waker.wake();
        } else {
            println!("[Async_lib][Sender][send] No waker to wake up");
        }
    }
}

// The Receiver struct is used to receive values via the channel
pub struct Receiver<T: Send> {
    buffer: Arc<Mutex<VecDeque<T>>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl<T: Send> SimpleFuture for Receiver<T> {
    type Output = T;
    // Poll the receiver to check if there is a value in the buffer
    // If there is no value in the buffer, set up the waker to wake up the receiver when a value is sent
    // This is effectively awaiting, as the receiver will wait until a value is sent
    fn poll(&mut self, ctx: &mut Context) -> State<Self::Output> {
        println!("[Async_lib][Receiver][poll] Polling ...");
        let mut buffer = self.buffer.lock().unwrap();
        if let Some(value) = buffer.pop_front() {
            println!("[Async_lib][Receiver][poll] Value found in buffer");
            State::Ready(value)
        } else {
            println!("[Async_lib][Receiver][poll] No value in buffer, awaiting ...");
            let mut waker_slot = self.waker.lock().unwrap();
            if let Some(waker) = ctx.waker() {
                *waker_slot = Some(waker.clone());
                println!("[Async_lib][Receiver][poll] Waker set");
            }
            State::Pending
        }
    }
}