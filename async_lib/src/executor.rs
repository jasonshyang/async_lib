use crate::task::{State, Context, Waker};
use crate::future::SimpleFuture;
use crate::channel::Channel;
use std::sync::Arc;

pub struct Executor {
    tasks: Vec<Box<dyn SimpleFuture<Output = ()>>>, // Vector of tasks to be executed
    channel: Channel<()>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: Vec::new(),
            channel: Channel::new(),
        }
    }

    pub fn spawn(&mut self, task: Box<dyn SimpleFuture<Output = ()>>) {
        self.tasks.push(task);
    }

    pub fn run(&mut self) {
        let mut receiver = self.channel.receiver();
        while !self.tasks.is_empty() {
            let mut i = 0;
            while i < self.tasks.len() { // Iterate over the tasks and poll them
                let task = &mut self.tasks[i];
                // Create a new waker for the task and notify the executor when the task is ready
                let waker = Waker::new(Arc::new({
                    let sender = self.channel.sender();
                    move || {
                        sender.send(());
                        println!("Executor: Task woke up ...");
                    }
                }));
                let mut ctx = Context::new(waker);
                match task.poll(&mut ctx) {
                    State::Ready(_) => {
                        println!("Executor: Task is ready, removing from list");
                        self.tasks.remove(i);
                    }
                    State::Pending => {
                        println!("Executor: Task is pending, moving to next");
                        i += 1;
                    }
                }
                while let State::Ready(_) = receiver.poll(&Context::new(Waker::new(Arc::new(|| {})))) {
                    println!("Executor: Successfully polled the receiver ...");
                }
            }
        }
    }
}