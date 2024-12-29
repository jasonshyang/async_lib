use crate::task::{Context, SimpleFuture, State, Task, Waker};
use std::sync::{Arc, Condvar, Mutex};
use std::collections::VecDeque;

/*
    The Executor struct is responsible for managing the tasks and running them to completion.
    It contains a queue of tasks and a notifier to wake up the executor when a task is ready.
    The spawn method is used to add a new task to the queue.
    The wait method is used to wait for a notification to wake up the executor.
    The run method is used to run the tasks in the queue to completion.
    If a task is not ready, it is added back to the queue and the executor waits for a notification to wake up.
    If the queue is empty, the executor stops running.
*/

pub struct Executor {
    queue: VecDeque<Task>,
    notifier: Arc<(Mutex<bool>, Condvar)>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            queue: VecDeque::new(),
            notifier: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }
    
    // Takes a closure that implements the SimpleFuture trait, wraps it in a Task struct and adds it to the queue
    pub fn spawn<F>(&mut self, f: F)
    where
        F: SimpleFuture<Output = ()> + 'static,
    {
        self.queue.push_back(
            Task::new(Box::new(f))
        );
    }

    // Blocks the executor until a notification is received via the Condvar
    // The Waker function wakes up the executor using the Condvar
    pub fn wait(&self) -> Result<(), ()> {
        let (lock, cvar) = &*self.notifier;
        let mut notified = lock.lock().unwrap();
        while !*notified {
            notified = cvar.wait(notified).unwrap();
        }
        *notified = false;
        Ok(())
    }

    // Runs the tasks in the queue to completion
    pub fn run(&mut self) {
        while !self.queue.is_empty() {
            let mut pending_tasks = VecDeque::new();

            // Run each task in the queue
            while let Some(mut task) = self.queue.pop_front() {
                let waker = Waker::new(Arc::new({
                    let notifier = self.notifier.clone();
                    move || {
                        let (lock, cvar) = &*notifier;
                        let mut notified = lock.lock().unwrap();
                        *notified = true;
                        cvar.notify_one();
                    }
                }));
                let mut ctx = Context::new(waker);
                match task.future.poll(&mut ctx) {
                    State::Ready(_) => {}
                    State::Pending => {
                        pending_tasks.push_back(task);
                    }
                }
            }

            if pending_tasks.is_empty() {
                break;
            }
            // Wait for notification to wake up
            self.wait().unwrap();

            // Put the pending tasks back into the queue
            self.queue = pending_tasks;
        }
    }
}
