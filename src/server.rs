use crate::epoll::*;

use std::collections::HashMap;
use std::io::prelude::*;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;

#[test]
fn server() {
    let mut eloop = EventLoop::new();
    let mut listener = MyTCPListener::bind("127.0.0.1:7878");
    let server = listener.0.as_raw_fd();
    let mut connections = HashMap::new();
    eloop.register(listener.0.as_raw_fd(), ffi::EPOLLIN);

    loop {
        let events = eloop.wait();
        for event in events {
            if event.is_read_ready() && event.fd == server {
                let (stream, _) = listener.0.accept().unwrap();
                stream.set_nonblocking(true).unwrap();
                eloop.register(stream.as_raw_fd(), ffi::EPOLLIN);
                connections.insert(stream.as_raw_fd(), stream);
            } else if event.is_read_ready() {
                handle_read_event(&eloop, event.fd, &connections);
            } else {
                handle_write_event(&eloop, event.fd, &connections);
            }
        }
    }

    //
    //match listener.accept() {
    //    Ok((_socket, addr)) => println!("new client: {:?}", addr),
    //    Err(e) => println!("couldn't get client: {:?}", e),
    //}
}

fn handle_read_event(eloop: &EventLoop, fd: i32, connections: &HashMap<i32, TcpStream>) {
    let mut stream = connections.get(&fd).unwrap();
    let mut buff: [u8; 1024] = [0; 1024];
    stream.read(&mut buff).unwrap();
    println!("{}", std::str::from_utf8(&buff).unwrap());
    eloop.reregister(fd, ffi::EPOLLOOUT)
}

fn handle_write_event(eloop: &EventLoop, fd: i32, connections: &HashMap<i32, TcpStream>) {
    let mut stream = connections.get(&fd).unwrap();
    let mut message = b"boran";
    stream.write(&message[..]).unwrap();
    eloop.reregister(fd, ffi::EPOLLIN)
}
