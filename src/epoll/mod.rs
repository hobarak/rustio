use std::io;
use std::{
    future::Future,
    net::{TcpListener, ToSocketAddrs},
    task::Context,
};

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
    #[repr(C)]
    pub struct Event {
        pub flags: i32,
        pub fd: i32,
    }

    impl Event {
        pub fn is_read_ready(&self) -> bool {
            self.flags & EPOLLIN == EPOLLIN
        }
        pub fn is_write_ready(&self) -> bool {
            self.flags & EPOLLIN == EPOLLOOUT
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

impl EventLoop {
    pub fn new() -> EventLoop {
        let fd = unsafe { ffi::epoll_create(1) };
        if fd < 0 {
            panic!(io::Error::last_os_error());
        }
        EventLoop { fd }
    }
    pub fn wait(&self) -> Vec<ffi::Event> {
        let mut events = Vec::with_capacity(10);
        let res = unsafe { ffi::epoll_wait(self.fd, events.as_mut_ptr(), 10, -1) };
        if res < 0 {
            panic!(io::Error::last_os_error());
        };
        unsafe { events.set_len(res as usize) };
        events
    }

    pub fn register(&self, fd: i32, flags: i32) {
        let mut event = ffi::Event {
            flags: flags,
            fd: fd,
        };

        let res = unsafe { ffi::epoll_ctl(self.fd, ffi::EPOLL_CTL_ADD, fd, &mut event) };
        if res < 0 {
            panic!(io::Error::last_os_error());
        }
    }

    pub fn reregister(&self, fd: i32, flags: i32) {
        let mut event = ffi::Event {
            flags: flags,
            fd: fd,
        };

        let res = unsafe { ffi::epoll_ctl(self.fd, ffi::EPOLL_CTL_MOD, fd, &mut event) };
        if res < 0 {
            panic!(io::Error::last_os_error());
        }
    }

    pub fn deregister(&self, fd: i32) {
        let res = unsafe { ffi::epoll_ctl(self.fd, ffi::EPOLL_CTL_DEL, fd, std::ptr::null_mut()) };
        if res < 0 {
            panic!(io::Error::last_os_error());
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
    pub fn accept() {}
}
