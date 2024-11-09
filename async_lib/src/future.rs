use crate::task::{State, Context};

// Simpflied version of the Future trait, used to define the interface for the future types
pub trait SimpleFuture {
    type Output; // used to define the output type of the future

    fn poll(&mut self, ctx: &Context) -> State<Self::Output>;
}