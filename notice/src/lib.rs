#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        extern crate notice_eventfd as notice_os;
    } else if #[cfg(unix)] {
        extern crate notice_pipe as notice_os;
    }
}

pub use notice_os::prelude;

pub use notice_os::*;
