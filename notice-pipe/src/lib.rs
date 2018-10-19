extern crate nix;
extern crate notice_core;

pub mod prelude {
    pub use super::notice_core::{Notify, Wait};
}

use prelude::*;

use nix::Error as NixError;
use nix::errno::Errno;
use nix::unistd;
use nix::fcntl::OFlag;

use notice_core::Unicast;

use std::os::unix::io::{AsRawFd, FromRawFd, RawFd, IntoRawFd};
use std::io;

struct Pipe(Option<RawFd>);

impl Pipe {
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        unistd::read(self.as_raw_fd(), buf).map_err(nix2io)
    }

    pub fn write_all(&self, buf: &[u8]) -> io::Result<()> {
        let mut written = 0;
        loop {
            match unistd::write(self.as_raw_fd(), &buf[written..buf.len()]) {
                Ok(x) => written += x,
                Err(NixError::Sys(Errno::EINTR)) => (),
                Err(e) => return Err(nix2io(e)),
            }

            if written == buf.len() {
                break;
            }
        }

        Ok(())
    }
}

impl AsRawFd for Pipe {
    fn as_raw_fd(&self) -> RawFd {
        match self.0 {
            Some(x) => x,
            None => unreachable!(),
        }
    }
}

impl FromRawFd for Pipe {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Pipe(Some(fd))
    }
}

impl IntoRawFd for Pipe {
    fn into_raw_fd(mut self) -> RawFd {
        match self.0.take() {
            Some(x) => x,
            None => unreachable!(),
        }
    }
}

impl Drop for Pipe {
    fn drop(&mut self) {
        if let Some(x) = self.0.take() {
            unistd::close(x).ok();
        }
    }
}

pub struct Notifier(Pipe);

impl AsRawFd for Notifier {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl FromRawFd for Notifier {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Notifier(Pipe::from_raw_fd(fd))
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
        self.0.write_all(&[1])
    }
}

pub struct Waiter {
    pipe: Pipe,
    buf_sz: usize,
}

impl Waiter {
    const DEFAULT_BUFFER_SIZE: usize = 64;

    pub fn set_buffer_len(&mut self, len: usize) {
        self.buf_sz = len;
    }

    pub fn buffer_len(&self) -> usize {
        self.buf_sz
    }
}

impl AsRawFd for Waiter {
    fn as_raw_fd(&self) -> RawFd {
        self.pipe.as_raw_fd()
    }
}

impl FromRawFd for Waiter {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Waiter {
            pipe: Pipe::from_raw_fd(fd),
            buf_sz: Self::DEFAULT_BUFFER_SIZE,
        }
    }
}

impl IntoRawFd for Waiter {
    fn into_raw_fd(self) -> RawFd {
        self.pipe.into_raw_fd()
    }
}

impl Unicast for Waiter {}

impl Wait for Waiter {
    type Error = io::Error;

    fn wait(&self) -> Result<usize, io::Error> {
        let mut buf = vec![0; self.buf_sz];
        self.pipe.read(&mut buf)
    }
}

fn nix2io(e: NixError) -> io::Error {
    match e {
        NixError::Sys(e) => io::Error::from_raw_os_error(e as i32),
        _ => io::Error::new(io::ErrorKind::Other, e),
    }
}

pub fn pair() -> Result<(Notifier, Waiter), io::Error> {
    let (r_fd, w_fd) = unistd::pipe2(OFlag::O_CLOEXEC).map_err(nix2io)?;

    let wr = Notifier(Pipe(Some(w_fd)));
    let rd = Waiter {
        pipe: Pipe(Some(r_fd)),
        buf_sz: Waiter::DEFAULT_BUFFER_SIZE,
    };

    Ok((wr, rd))
}
