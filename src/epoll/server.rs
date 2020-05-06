use crate::epoll::*;
use crate::runtime::*;

use std::io::prelude::*;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;

#[test]
fn server() {
    let mut registery = Registery::create();
    let mut listener = MyTCPListener::bind("127.0.0.1:7878");

    registery.add_read_listener(listener.0.as_raw_fd(), move |reg, fd: i32| {
        let (stream, addr) = listener.0.accept().unwrap();
        stream.set_nonblocking(true).unwrap();

        reg.add_read_listener(stream.as_raw_fd(), |reg, fd: i32| {
            let mut stream = unsafe { TcpStream::from_raw_fd(fd) };

            let mut buff: [u8; 1024] = [0; 1024];
            stream.read(&mut buff).unwrap();
            println!("{}", std::str::from_utf8(&buff).unwrap());
        });
    });

    registery.run();

    //
    //match listener.accept() {
    //    Ok((_socket, addr)) => println!("new client: {:?}", addr),
    //    Err(e) => println!("couldn't get client: {:?}", e),
    //}
}
