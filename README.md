# async_lib
An over-simplified async runtime implementation in Rust for learning purpose.

This crate is a manual exploration of asynchronous programming concepts in Rust. It avoids using standard features like the 'Future' trait and 'mpsc' channels to provide a deeper understanding of how async runtimes function under the hood. 

This is purely for my own educational and experimental purposes.

## What It Does
'async_lib' provides a minimal implementation of core async programming concepts, including:

- **SimpleFuture trait**: a simplified version of the 'Rust' future trait.
- **Executor**: a basic task scheduler for executing asynchronous tasks.
- **Channels**: a custom implementation for task communication.
- **Waker**: manages notifications for when tasks are ready to progress.

## Limitations
Even for a simple manual async implementation, this crate is still lacking many basic features, and can only perform the very basic async tasks.

Below is a list of thoughts afterward:

* 'Waker' should be invoked by event-driven listeners responding to I/O events, here wakers are manually triggered by the sender, which 
   restricts functionality.. (this contributes to many of the below limitations)
*  Using 'Condvar' as a way to wake up executor isnt ideal as it blocks the thread when the Executor awaits.
*  Due to using 'Condvar', 'Executor' currently doesnt provide a true runtime environment, tasks are spawned first and then call run().
   If Executor is ran on a separate thread, this might be overcame but still very very far from ideal
*  Cannot use 'Rust' 'async/await' syntax sugar as this crate uses its own 'SimpleFuture'

## Code Structure

- 'src/future.rs': Defines the Future trait and related types.
- 'src/executor.rs': Contains the task execution logic and the executor implementation.
- 'src/channel.rs': Implements asynchronous communication channels.
- 'src/task.rs': Manages task states and the wake-up mechanism.
- 'examples/': Provides example implementation demonstrating the library's capabilities.

## Usage
### Example
This example demonstrates how non-blocking I/O can be executed asynchronously using this crate.
```rust,no_run
    use std::thread;
    use std::time::Duration;

    use async_lib::{
        executor::Executor, 
        task::{Context, SimpleFuture, State},
        channel::*,
    };
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
```

### Run the Example:
To try out an example demonstrating basic async behavior:
```bash
# Run the example
cargo run --example io
```