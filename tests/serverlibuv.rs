use rustio::epoll::*;
use rustio::libuv::*;

use std::collections::HashMap;
use std::io::prelude::*;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

#[test]
pub fn server() {
    let mut runtime = Runtime::new();
    let mut listener = MyTCPListener::bind("127.0.0.1:7879");
    let mut buf = [0_u8; 1024];

    runtime.add(
        listener.as_raw_fd(),
        Interest::Read,
        move |runtime, token| {
            let mut stream = listener.accept();
            runtime.add(stream.as_raw_fd(), Interest::Read, move |runtime, token| {
                stream.read(&mut buf).unwrap();
                println!("{}", std::str::from_utf8(&buf).unwrap());
            });
        },
    );

    runtime.run();
}
