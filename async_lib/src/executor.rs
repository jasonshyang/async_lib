use crate::task::{State, Context, Waker};
use crate::future::SimpleFuture;
use crate::channel::Channel;
use std::sync::Arc;
use std::collections::VecDeque;
use std::thread;

pub struct Executor {
    tasks: VecDeque<Box<dyn SimpleFuture<Output = ()>>>, // Tasks queue
    channel: Channel<()>, // Channel to wake up the tasks
}

impl Executor {
    // Create a new instance of the executor, containing empty list of tasks and a channel
    pub fn new() -> Self {
        Executor {
            tasks: VecDeque::new(),
            channel: Channel::new(),
        }
    }

    // Spawn a new task to the executor
    pub fn spawn(&mut self, task: Box<dyn SimpleFuture<Output = ()>>) {
        self.tasks.push_back(task);
    }

    // Run the executor
    // Initial polling: poll all the tasks in the list, if pending, stores the waker in the context
    // If all tasks are pending, wait for the wake up signal from the receiver
    // If a wake up signal is received, continue polling the tasks
    pub fn run(&mut self) {
        let mut receiver = self.channel.receiver();
        // Main loop to run the executor, runs until all tasks are completed
        'main: while !self.tasks.is_empty() {
            println!("[Async_lib][Executor][run] Running executor ...");
            let mut pending_tasks = VecDeque::new();
            let mut all_tasks_pending = true;
            // Poll all the tasks in the list
            while let Some(mut task) = self.tasks.pop_front() {
                println!("[Async_lib][Executor][run] Polling task ...");
                let waker = Waker::new(Arc::new({
                    let thread = thread::current();
                    move || {
                        println!("!!! Executor Waking up !!!");
                        thread.unpark();
                    }
                }));
                // Poll the task and check the state, context is passed to the task with the waker
                let mut ctx = Context::new(waker);
                match task.poll(&mut ctx) {
                    State::Ready(_) => {
                        println!("[Async_lib][Executor][run] Task completed");
                        all_tasks_pending = false;
                    }
                    State::Pending => {
                        println!("[Async_lib][Executor][run] Task pending");
                        pending_tasks.push_back(task); // Push the task back to the list if it is not ready
                    }
                }
            }

            self.tasks = pending_tasks;

            if !self.tasks.is_empty() && all_tasks_pending {
                println!("[Async_lib][Executor][run] All tasks are pending, waiting for wake up signal ...");
                println!("ZZZzzz... Executor Sleeping ZZZzzz...");
                let waker = Waker::noop();
                let mut ctx = Context::new(waker);
                
                match receiver.poll(&mut ctx) {
                    State::Ready(_) => continue 'main,  // Resume polling tasks with the main loop
                    State::Pending => thread::park(),
                }
            }

            println!("[Async_lib][Executor][run] Tasks left: {}", self.tasks.len());
        }
    }
}