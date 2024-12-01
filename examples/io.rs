use std::thread;
use std::time::Duration;

use async_lib::{
    executor::Executor, 
    task::{Context, SimpleFuture, State},
    channel::*,
};

/*
    This example simulates non-blocking I/O operations and demonstrates this crate creats 
    a simple executor that can run asynchronous tasks. For simplicity, we use a separate 
    thread to simulate the non-blocking I/O operations.

    The `start_task` function creates a new task that will run for a specified duration.
    The `process_task` function simulates the I/O operation by sleeping for the specified
    duration and then sending a message to the task that the operation is complete.

    The `IoTask` struct represents the task that will be executed
    The `SimpleFuture` trait is implemented for the `IoTask` struct
    The `poll` method is implemented to check if the task is ready or pending

    The `BoxedSimpleFuture` struct is used to store the `IoTask` struct in a Box

    The `main` function creates a new executor and spawns four tasks with different durations
    The executor runs the tasks and waits for them to complete

    The output will show the tasks starting and completing in the order they were spawned

    Expected output:
    ```
    Starting I/O task ... 1
    Starting I/O task ... 2
    Starting I/O task ... 3
    Starting I/O task ... 4
    Received data: Task id: 1 completed, process time: 1 seconds
    Received data: Task id: 3 completed, process time: 2 seconds
    Received data: Task id: 2 completed, process time: 3 seconds
    Received data: Task id: 4 completed, process time: 3 seconds
    ```

*/

fn main() {
    let mut executor = Executor::new();
    executor.spawn( start_task(1, 1) );
    executor.spawn( start_task(2, 3) );
    executor.spawn( start_task(3, 2) );
    executor.spawn( start_task(4, 3) );
    executor.run();
}

fn start_task(id: usize, duration: u64) -> BoxedSimpleFuture {
    println!("Starting I/O task ... {}", id);
    let (sender, receiver) = channel::<String>();
    let task = IoTask { receiver };

    thread::spawn(move || {
        process_task(id, Duration::from_secs(duration), sender);
    });

    BoxedSimpleFuture(Box::new(task))
}

fn process_task(id: usize, duration: Duration, sender: Sender<String>) {
    thread::sleep(duration);
    let data = "Task id: ".to_string() + &id.to_string() + " completed, process time: " + &duration.as_secs().to_string() + " seconds";
    sender.send(data);
}

struct IoTask {
    receiver: Receiver<String>,
}

impl SimpleFuture for IoTask {
    type Output = ();

    fn poll(&mut self, ctx: &mut Context) -> State<Self::Output> {
        match self.receiver.try_recv() {
            Some(data) => {
                println!("Received data: {}", data);
                State::Ready(())
            }
            None => {
                self.receiver.register_waker(ctx.waker().clone());
                State::Pending
            }
        }
    }
}

struct BoxedSimpleFuture(Box<dyn SimpleFuture<Output = ()>>);

impl SimpleFuture for BoxedSimpleFuture {
    type Output = ();

    fn poll(&mut self, ctx: &mut Context) -> State<Self::Output> {
        self.0.as_mut().poll(ctx)
    }
}