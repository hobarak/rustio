use crate::epoll::*;

use std::io::prelude::*;

#[test]
fn client() {
    let mut stream = MyTcpStream::connect("slowwly.robertomurray.co.uk:80");
    let mut eloop = EventLoop::new();
    eloop.add(&stream, 1, ffi::EPOLLOOUT);
    let mut buff: [u8; 1024] = [0; 1024];

    loop {
        let events = eloop.wait();

        println!("{:?}", events.len());

        for event in events {
            println!("{:?}", event);
            if event.is_read_ready() {
                stream.read(&mut buff).unwrap();
                println!("{}", std::str::from_utf8(&buff).unwrap());
                eloop.remove(&stream);
            } else if event.is_write_ready() {
                let request = "GET /\r\n\
             Host: www.google.com\r\n\
             Connection: close\r\n\
             \r\n";

                stream.write(request.as_bytes()).unwrap();

                eloop.modify(&stream, event.token, ffi::EPOLLIN);
            }
        }
    }
}
