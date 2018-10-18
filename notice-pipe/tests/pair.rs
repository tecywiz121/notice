extern crate notice_test;
extern crate notice_pipe;

use notice_test::{Pair, all_tests};

struct PipePair;

impl Pair for PipePair {
    type Notify = notice_pipe::Notifier;
    type Wait = notice_pipe::Waiter;

    fn pair(&self) -> (Self::Notify, Self::Wait) {
        notice_pipe::pair().unwrap()
    }
}

#[test]
fn test() {
    all_tests::<PipePair>(PipePair);
}
