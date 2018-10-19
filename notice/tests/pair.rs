extern crate notice_test;
extern crate notice;

use notice_test::{Pair, all_tests};

struct PipePair;

impl Pair for PipePair {
    type Notify = notice::Notifier;
    type Wait = notice::Waiter;

    fn pair(&self) -> (Self::Notify, Self::Wait) {
        notice::pair().unwrap()
    }
}

#[test]
fn test() {
    all_tests::<PipePair>(PipePair);
}
