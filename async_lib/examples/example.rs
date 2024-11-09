use async_lib::executor::Executor;
use async_lib::future::SimpleFuture;
use async_lib::task::{Context, State};
use async_lib::channel::{Channel, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct TestFuture {
    id: usize,
    shared_state: Arc<Mutex<i32>>,
}

impl SimpleFuture for TestFuture {
    type Output = ();

    fn poll(&mut self, _ctx: &Context) -> State<Self::Output> {
        let mut state = self.shared_state.lock().unwrap();
        if *state < 5 {
            *state += 1;
            println!("Future ID: {}, Shared State: {}", self.id, *state);
            State::Pending
        } else {
            println!("Future ID: {}, Completed with Shared State: {}", self.id, *state);
            State::Ready(())
        }
    }
}

struct ProducerFuture {
    sender: Sender<i32>,
    count: i32,
    max_count: i32,
}

impl SimpleFuture for ProducerFuture {
    type Output = ();

    fn poll(&mut self, _ctx: &Context) -> State<Self::Output> {
        if self.count < self.max_count {
            println!("Producer: About to send value: {}", self.count);
            self.sender.send(self.count);
            println!("Producer: Sending value: {}", self.count);
            self.count += 1;
            thread::sleep(Duration::from_millis(100));
            State::Pending
        } else {
            println!("Producer: Completed sending values ...");
            State::Ready(())
        }
    }
}

struct ConsumerFuture {
    receiver: Receiver<i32>,
    received_values: Vec<i32>,
}

impl SimpleFuture for ConsumerFuture {
    type Output = ();

    fn poll(&mut self, ctx: &Context) -> State<Self::Output> {
        match self.receiver.poll(ctx) {
            State::Ready(value) => {
                self.received_values.push(value);
                println!("Consumer: Received value: {}", value);
                thread::sleep(Duration::from_millis(100));
                State::Pending
            }
            State::Pending => {
                if self.received_values.len() >= 5 {
                    println!("Consumer: Completed receiving values");
                    State::Ready(())
                } else {
                    State::Pending
                }
            }
        }
    }
}


fn main() {
    println!("Running first test on executor ...");
    let shared_state = Arc::new(Mutex::new(0));
    let mut executor = Executor::new();
    
    for id in 1..=3 {
        executor.spawn(Box::new(TestFuture {
            id,
            shared_state: shared_state.clone(),
        }));
    }

    executor.run();
    println!("Completed first test. Final shared state: {}", *shared_state.lock().unwrap());

    println!("Running second test on executor, resetting ...");
    let mut executor = Executor::new();

    let channel = Channel::new();
    let sender = channel.sender();
    let receiver = channel.receiver();

    executor.spawn(Box::new(ProducerFuture {
        sender,
        count: 0,
        max_count: 5,
    }));

    executor.spawn(Box::new(ConsumerFuture {
        receiver,
        received_values: Vec::new(),
    }));

    executor.run();

    println!("Completed second test.");

}