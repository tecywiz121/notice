#![deny(unused)]

extern crate notice;

use notice::{Notify, Wait};

use std::thread;

pub trait Pair {
    type Notify: 'static + Notify + Send;
    type Wait: 'static + Wait + Send;

    fn pair(&self) -> (Self::Notify, Self::Wait);
}

pub fn all_tests<P: Pair>(p: P) {
    test_notify_wait_one(&p);
    test_notify_wait_two(&p);
    test_wait_notify(&p);
}

fn test_notify_wait_one<P: Pair>(p: &P) {
    let (notify, wait) = p.pair();

    notify.notify().unwrap();

    let actual = wait.wait().unwrap();

    assert_eq!(1, actual);
}

fn test_notify_wait_two<P: Pair>(p: &P) {
    let (notify, wait) = p.pair();

    notify.notify().unwrap();
    notify.notify().unwrap();

    let actual = wait.wait().unwrap();

    assert_eq!(2, actual);
}

fn test_wait_notify<P: Pair>(p: &P) {
    let (notify, wait) = p.pair();

    let handle = thread::spawn(move || {
        wait.wait().unwrap()
    });

    notify.notify().unwrap();

    assert_eq!(1, handle.join().unwrap());
}
