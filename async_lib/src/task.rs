use std::sync::Arc;
pub enum State<T> {
    Pending,
    Ready(T),
}

#[derive(Clone)]
pub struct Waker {
    wake_fn: Arc<dyn Fn() + Send + Sync>, // Pointer to the function to be called when the task is woken up
}

impl Waker {
    pub fn new(wake_fn: Arc<dyn Fn() + Send + Sync>) -> Self {
        Waker { wake_fn }
    }

    pub fn wake(&self) {
        (self.wake_fn)();
    }
}

pub struct Context {
    waker: Option<Waker>,
}

impl Context {
    pub fn new(waker: Waker) -> Self {
        Context {
            waker: Some(waker),
        }
    }

    pub fn waker(&self) -> Option<&Waker> {
        self.waker.as_ref()
    }
}