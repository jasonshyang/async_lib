use async_lib::{
    task::{State, Context},
    channel::{Channel, Sender, Receiver},
    executor::Executor,
    future::SimpleFuture,
};
use std::thread;
use std::time::Duration;

// This example demonstrates how the lib can handle multiple network requests asynchronously
// Channel is used to wake up the tasks when the API calls are completed
// When the API calls are completed, the sender sends a signal to the receiver to wake up the task
// The tasks are not blocked by the API calls and can continue polling
pub struct NetworkRequest {
    id: usize,
    receiver: Receiver<String>,
    url: String,
}

impl NetworkRequest {
    pub fn new(id: usize, receiver: Receiver<String>, url: String) -> Self {
        NetworkRequest { id, receiver, url }
    }

    fn make_api_call(url: String, sender: Sender<String>) {
        thread::spawn(move || {
            println!("[NetworkRequest][API Call] Making request to {}", url);
            thread::sleep(Duration::from_millis(100));
            let response = format!("status: success, url: {}, data: ok", url);
            println!("[NetworkRequest][API Call] Got response from {}", url);
            sender.send(response);
        });
    }
}

impl SimpleFuture for NetworkRequest {
    type Output = ();

    fn poll(&mut self, ctx: &mut Context) -> State<Self::Output> {
        println!("[NetworkRequest {}][poll] Polling request to {} ...", self.id, self.url);

        match self.receiver.poll(ctx) {
            State::Ready(_) => {
                println!("NetworkRequest {}][poll] Request to {} completed", self.id, self.url);
                State::Ready(())
            }
            State::Pending => {
                println!("[NetworkRequest {}][poll] Request to {} still pending...", self.id, self.url);
                State::Pending
            }
        }
    }
}

fn main() {
    let mut executor = Executor::new();

    let channel1: Channel<String> = Channel::new();
    let channel2: Channel<String> = Channel::new();
    let channel3: Channel<String> = Channel::new();

    let request1 = NetworkRequest::new(1, channel1.receiver(), "http://api1.example.com".to_string());
    let request2 = NetworkRequest::new(2, channel2.receiver(), "http://api2.example.com".to_string());
    let request3 = NetworkRequest::new(3, channel3.receiver(), "http://api3.example.com".to_string());

    executor.spawn(Box::new(request1));
    executor.spawn(Box::new(request2));
    executor.spawn(Box::new(request3));

    thread::spawn(move || {
        executor.run();
    });

    // Simulate API calls
    NetworkRequest::make_api_call("http://api1.example.com".to_string(), channel1.sender());
    NetworkRequest::make_api_call("http://api2.example.com".to_string(), channel2.sender());
    NetworkRequest::make_api_call("http://api3.example.com".to_string(), channel3.sender());

    // Wait for all requests to complete
    thread::sleep(Duration::from_secs(2));
}