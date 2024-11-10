use crate::task::{State, Context};

// Simpflied version of the Future trait, used to define the interface for the future types used in the lib
pub trait SimpleFuture: Send {
    type Output;
    
    // The poll function is used to poll the future and check if it is ready
    // The poll function returns the state of the future, either Pending or Ready
    // When pending, the waker is set to wake up the future when it is ready
    fn poll(&mut self, ctx: &mut Context) -> State<Self::Output>; 
}