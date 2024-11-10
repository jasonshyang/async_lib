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

The Channel serves to wake up tasks when the simulated network request has been completed. Each request runs on its own thread and is not blocked by others. The tasks continue polling the channel until a response is available, at which point the task transitions from a "pending" state to a "ready" state.

```bash
# Run the example
cargo run --example network_request