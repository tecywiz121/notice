extern crate notice_test;
extern crate notice_eventfd;

use notice_test::{Pair, all_tests};

struct PipePair;

impl Pair for PipePair {
    type Notify = notice_eventfd::Notifier;
    type Wait = notice_eventfd::Waiter;

    fn pair(&self) -> (Self::Notify, Self::Wait) {
        notice_eventfd::pair().unwrap()
    }
}

#[test]
fn test() {
    all_tests::<PipePair>(PipePair);
}
