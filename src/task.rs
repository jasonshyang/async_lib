use std::sync::Arc;

/*
    The Waker is responsible for waking up the executor when the task is ready.
    The Waker is provided in the Context when polling the task by the executor.
    The Waker is directly stored in Channel for simplicity, this is a over-simplified 
    implementation of waker communication to demonstrate the concept.
*/

pub trait SimpleFuture {
    type Output;
    fn poll(&mut self, ctx: &mut Context) -> State<Self::Output>;
}

pub enum State<T> {
    Pending,
    Ready(T),
}

pub struct Task {
    pub future: Box<dyn SimpleFuture<Output = ()>>,
}

impl Task {
    pub fn new(future: Box<dyn SimpleFuture<Output = ()>>) -> Self {
        Task { future }
    }
}

pub struct Context {
    pub waker: Waker,
}

impl Context {
    pub fn new(waker: Waker) -> Self {
        Context { waker }
    }

    pub fn waker(&self) -> &Waker {
        &self.waker
    }
}

pub struct Waker {
    wake_fn: Arc<dyn Fn() + Send + Sync>,
}

impl Waker {
    pub fn new(wake_fn: Arc<dyn Fn() + Send + Sync>) -> Self {
        Waker { wake_fn }
    }

    pub fn wake(&self) {
        (self.wake_fn)();
    }

    pub fn noop() -> Self {
        Waker {
            wake_fn: Arc::new(|| {}),
        }
    }
}

impl Clone for Waker {
    fn clone(&self) -> Self {
        Waker {
            wake_fn: self.wake_fn.clone(),
        }
    }
}