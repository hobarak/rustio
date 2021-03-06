use io::{Read, Write};
use std::io;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::os::unix::io::AsRawFd;

pub mod ffi {
    pub const EPOLL_CTL_ADD: i32 = 1;
    pub const EPOLL_CTL_DEL: i32 = 2;
    pub const EPOLL_CTL_MOD: i32 = 3;

    pub const EPOLLIN: i32 = 0x1;
    pub const EPOLLOOUT: i32 = 0x4;

    #[link(name = "c")]
    extern "C" {
        pub fn epoll_create(size: i32) -> i32;
        pub fn close(fd: i32) -> i32;
        pub fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: *mut Event) -> i32;
        pub fn epoll_wait(epfd: i32, events: *mut Event, maxevents: i32, timeout: i32) -> i32;
    }
    #[derive(Debug)]
    #[repr(C)]
    pub struct Event {
        pub flags: i32,
        pub token: i32,
    }

    impl Event {
        pub fn is_read_ready(&self) -> bool {
            self.flags & EPOLLIN == EPOLLIN
        }
        pub fn is_write_ready(&self) -> bool {
            self.flags & EPOLLOOUT == EPOLLOOUT
        }
    }
}

pub struct EventLoop {
    fd: i32,
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        let res = unsafe { ffi::close(self.fd) };
        if res < 0 {
            panic!(io::Error::last_os_error());
        }
    }
}

type Result<T> = std::result::Result<T, std::io::Error>;

impl EventLoop {
    pub fn new() -> Result<EventLoop> {
        let fd = unsafe { ffi::epoll_create(1) };
        if fd < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(EventLoop { fd })
        }
    }
    pub fn wait(&self) -> Result<Vec<ffi::Event>> {
        let mut events = Vec::with_capacity(10);
        let res = unsafe { ffi::epoll_wait(self.fd, events.as_mut_ptr(), 10, 3000) };
        if res < 0 {
            Err(io::Error::last_os_error())
        } else {
            unsafe { events.set_len(res as usize) };
            Ok(events)
        }
    }

    pub fn add(&self, fd: i32, token: i32, flags: i32) -> Result<()> {
        let mut event = ffi::Event {
            flags: flags,
            token: token,
        };
        println!("{} {} {}", fd, token, flags);
        let res = unsafe { ffi::epoll_ctl(self.fd, ffi::EPOLL_CTL_ADD, fd, &mut event) };
        if res < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn modify<F: AsRawFd>(&self, fd: &F, token: i32, flags: i32) -> Result<()> {
        let mut event = ffi::Event {
            flags: flags,
            token: token,
        };

        let res =
            unsafe { ffi::epoll_ctl(self.fd, ffi::EPOLL_CTL_MOD, fd.as_raw_fd(), &mut event) };
        if res < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn remove<F: AsRawFd>(&self, fd: &F) -> Result<()> {
        let res = unsafe {
            ffi::epoll_ctl(
                self.fd,
                ffi::EPOLL_CTL_DEL,
                fd.as_raw_fd(),
                std::ptr::null_mut(),
            )
        };
        if res < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

pub struct MyTCPListener(pub TcpListener);

impl MyTCPListener {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> MyTCPListener {
        let mut listener = TcpListener::bind(addr).unwrap();
        listener
            .set_nonblocking(true)
            .expect("Cannot set non-blocking");
        MyTCPListener(listener)
    }
    pub fn accept(&self) -> Result<MyTcpStream> {
        let (stream, _) = self.0.accept()?;
        stream.set_nonblocking(true)?;
        Ok(MyTcpStream(stream))
    }
}

pub struct MyTcpStream(pub TcpStream);

impl MyTcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> MyTcpStream {
        let stream = TcpStream::connect(addr).unwrap();
        stream.set_nonblocking(true).unwrap();
        MyTcpStream(stream)
    }
}

impl Write for MyTcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}
impl Read for MyTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl AsRawFd for MyTcpStream {
    fn as_raw_fd(&self) -> i32 {
        self.0.as_raw_fd()
    }
}

impl AsRawFd for MyTCPListener {
    fn as_raw_fd(&self) -> i32 {
        self.0.as_raw_fd()
    }
}
