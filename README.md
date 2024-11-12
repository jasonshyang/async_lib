# async_lib
A simple async runtime implementation in Rust for learning purposes.

## What it does
This library implements basic async programming concepts:
- Future polling mechanism
- Task executor
- Async channels
- Task wake-up system

## Code Structure
- `src/future.rs` - Future trait definition
- `src/executor.rs` - Task execution logic  
- `src/channel.rs` - Async communication
- `src/task.rs` - Task state and waking

## Example
This example simulate making multiple network requests concurrently. The executor schedules three network requests, each represented by a NetworkRequest task. The tasks will poll for data from a channel, and when the data (simulating an API response) is ready, the task is woken up.

```bash
# Run the example
cargo run --example network_request
```

## Limitations
- Cannot add tasks to the executor during runtime, so this does not really provide a runtime env for a typical server use case.
- Might be able to improve this if the executor can use the channel to spawn new tasks, and enhance the channel to be able to handle cross thread task communication.
- Also the executor doesn't support multithreaded execution, all the tasks are put in one queue which is not efficient, a thread pool can potentially improve this.

## Next Iteration
Improve the lib so it can actually handle multithread, and use for a simple echo server/client example where server can spawn multiple connections with multiple clients and use the lib to process client requests.

The basic idea for next iteraction design:
- The executor provides the runtime env, where it will first poll all tasks registered in queue
- When a task is polled, it checks if the condition is ready, if yes completes the task and pass back data and the Ready state, if no pass back the Pending state. The task should also register the Waker with the Executor so it can wake up the Executor when receiving a signal
- If all the tasks are pending, the Executor goes to sleep to avoid busy polling
- Meanwhile new tasks can continue to be spawned, which adds to the Executor's queue
- In multithread scenario, need to use a Channel to pass the tasks across to the Executor running on the Executor thread
- When new task is added, Executor main loop wakes up again and go through the task queue, and go back to sleep
- When there is signal for the task to be ready (e.g. network msg received), the signal is communicated via the Sender where it invokes the Waker, again need to consider multithread scenario where a Channel is required to pass the signal back to wake up the Executor
- Now the Executor is waken up, it goes through the queue and poll them

Futher improvement:
- Provide a way to terminate the runtime
- Thread Pool so the Executor can handle things more efficiently 