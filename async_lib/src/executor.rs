use crate::task::{State, Context, Waker};
use crate::future::SimpleFuture;
use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;

// TODO: Implement thread pool to distribute the tasks to multiple threads
pub struct Executor {
    tasks: Arc<Mutex<VecDeque<Box<dyn SimpleFuture<Output = ()>>>>>, // List of tasks to run
    notifier: Arc<(Mutex<bool>, Condvar)>, // Notifier to wake up the executor
}

impl Executor {
    // Create a new instance of the executor, containing empty list of tasks and a channel
    pub fn new() -> Self {
        Executor {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
            notifier: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    // Spawn a new task to the executor
    // Push the task to the list and notify the executor to wake up via the notifier
    pub fn spawn(&mut self, task: Box<dyn SimpleFuture<Output = ()>>) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push_back(task);
        let (lock, cvar) = &*self.notifier;
        let mut notified = lock.lock().unwrap();
        *notified = true;
        cvar.notify_one();
    }

    pub fn run(&mut self) {
        let (lock, cvar) = &*self.notifier;
        // Main loop to run the executor, runs until all tasks are completed
        'main: loop {
            println!("[Async_lib][Executor][run] Running executor main loop ...");
            let mut tasks = self.tasks.lock().unwrap();

            if tasks.is_empty() {
                let mut notified = lock.lock().unwrap();
                while !*notified {
                    println!("[Async_lib][Executor][run] No tasks to run, sleeping ...");
                    notified = cvar.wait(notified).unwrap();
                }
                *notified = false;

                continue 'main;
            }

            let mut pending_tasks = VecDeque::new();
            let mut all_tasks_pending = true;
            // Poll all the tasks in the list
            while let Some(mut task) = tasks.pop_front() {
                println!("[Async_lib][Executor][run] Polling task from the list ...");
                let waker = Waker::new(Arc::new({
                    let notifier = self.notifier.clone();
                    move || {
                        let (lock, cvar) = &*notifier;
                        let mut notified = lock.lock().unwrap();
                        *notified = true;
                        cvar.notify_one();
                        println!("!!! Executor Waking up !!!");
                    }
                }));
                // Poll the task and check the state, context is passed to the task with the waker
                let mut ctx = Context::new(waker);
                match task.poll(&mut ctx) {
                    State::Ready(_) => {
                        println!("[Async_lib][Executor][run] Task completed, Tasks left: {}", tasks.len());
                        all_tasks_pending = false;
                    }
                    State::Pending => {
                        println!("[Async_lib][Executor][run] Task pending");
                        pending_tasks.push_back(task); // Push the task back to the list if it is not ready
                    }
                }
            }

            *tasks = pending_tasks; // Update the list with the pending tasks

            if !tasks.is_empty() && all_tasks_pending {
                println!("[Async_lib][Executor][run] All tasks are pending, waiting for wake up signal ...");
                println!("ZZZzzz... Executor Sleeping ZZZzzz...");
                drop(tasks);

                let mut notified = lock.lock().unwrap();
                while !*notified {
                    notified = cvar.wait(notified).unwrap();
                }
                *notified = false;
            }
            println!("[Async_lib][Executor][run] Executor main loop completed");
        }
    }
}