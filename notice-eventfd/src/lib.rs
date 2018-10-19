extern crate nix;
extern crate notice_core;

pub mod prelude {
    pub use super::notice_core::{Notify, Wait};
}

use prelude::*;

use nix::Error as NixError;
use nix::unistd;
use nix::sys::eventfd::{eventfd, EfdFlags};
use nix::fcntl::{fcntl, FcntlArg};

use notice_core::Unicast;

use std::os::unix::io::{AsRawFd, FromRawFd, RawFd, IntoRawFd};
use std::io;
use std::mem;

struct EventFd(Option<RawFd>);

impl EventFd {
    pub fn read(&self) -> io::Result<usize> {
        let mut buf = [0u8; 8];
        let count = unistd::read(self.as_raw_fd(), &mut buf).map_err(nix2io)?;

        if 8 != count {
            return Err(io::Error::from(io::ErrorKind::Other));
        }

        let num: u64 = unsafe { mem::transmute(buf) };

        Ok(num as usize)
    }

    pub fn write(&self) -> io::Result<()> {
        let buf: [u8; 8] = unsafe { mem::transmute(1u64) };
        unistd::write(self.as_raw_fd(), &buf).map_err(nix2io)?;
        Ok(())
    }
}

impl AsRawFd for EventFd {
    fn as_raw_fd(&self) -> RawFd {
        match self.0 {
            Some(x) => x,
            None => unreachable!(),
        }
    }
}

impl FromRawFd for EventFd {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        EventFd(Some(fd))
    }
}

impl IntoRawFd for EventFd {
    fn into_raw_fd(mut self) -> RawFd {
        match self.0.take() {
            Some(x) => x,
            None => unreachable!(),
        }
    }
}

impl Drop for EventFd {
    fn drop(&mut self) {
        if let Some(x) = self.0.take() {
            unistd::close(x).ok();
        }
    }
}

pub struct Notifier(EventFd);

impl AsRawFd for Notifier {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl FromRawFd for Notifier {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Notifier(EventFd::from_raw_fd(fd))
    }
}

impl IntoRawFd for Notifier {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl Unicast for Notifier {}

impl Notify for Notifier {
    type Error = io::Error;

    fn notify(&self) -> Result<(), io::Error> {
        self.0.write()
    }
}

pub struct Waiter(EventFd);

impl AsRawFd for Waiter {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl FromRawFd for Waiter {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Waiter(EventFd::from_raw_fd(fd))
    }
}

impl IntoRawFd for Waiter {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl Unicast for Waiter {}

impl Wait for Waiter {
    type Error = io::Error;

    fn wait(&self) -> Result<usize, io::Error> {
        self.0.read()
    }
}

fn nix2io(e: NixError) -> io::Error {
    match e {
        NixError::Sys(e) => io::Error::from_raw_os_error(e as i32),
        _ => io::Error::new(io::ErrorKind::Other, e),
    }
}

pub fn pair() -> Result<(Notifier, Waiter), io::Error> {
    let w_fd = eventfd(0, EfdFlags::EFD_CLOEXEC).map_err(nix2io)?;
    let wr = Notifier(EventFd(Some(w_fd)));

    let r_fd = fcntl(wr.as_raw_fd(), FcntlArg::F_DUPFD_CLOEXEC(w_fd)).map_err(nix2io)?;
    let rd = Waiter(EventFd(Some(r_fd)));

    Ok((wr, rd))
}
