use std::sync::Arc;
pub enum State<T> {
    Pending,
    Ready(T), // Ready might not be the best name here: it means the task is already done and the output is ready, not the task is ready
}


// The Waker struct is used to wake up the task when it is ready
#[derive(Clone)]
pub struct Waker {
    wake_fn: Arc<dyn Fn() + Send + Sync>, // Pointer to the function to be called when the task is woken up
}

impl Waker {
    // Create a new instance of the Waker with the function to be called when the task is woken up
    pub fn new(wake_fn: Arc<dyn Fn() + Send + Sync>) -> Self {
        Waker { wake_fn }
    }

    // Call the function to wake up the task
    pub fn wake(&self) {
        println!("[Async_lib][Waker][wake] Waking up task ...");
        (self.wake_fn)();
    }
}

// The Context struct is used to store the waker for the task, it is used by the task to get the waker
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