use std::error::Error;

pub trait Broadcast {
}

pub trait Unicast {
}

pub trait Wait {
    type Error: Error;

    fn wait(&self) -> Result<usize, Self::Error>;
}

pub trait Notify {
    type Error: Error;

    fn notify(&self) -> Result<(), Self::Error>;
}
