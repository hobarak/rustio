use rustio::epoll::*;

use std::collections::HashMap;
use std::io::prelude::*;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
struct State {
    message: Vec<u8>,
    stream: MyTcpStream,
}

//#[test]
pub fn server() {
    let mut eloop = EventLoop::new().unwrap();
    let mut listener = MyTCPListener::bind("127.0.0.1:7878");
    let mut connections = HashMap::new();

    let mut counter = 0;
    let mut next = || {
        counter += 1;
        counter
    };
    let server = next();
    eloop
        .add(listener.as_raw_fd(), server, ffi::EPOLLIN)
        .unwrap();

    loop {
        let events = eloop.wait().unwrap();
        for event in events {
            if event.is_read_ready() && event.token == server {
                let stream = listener.accept().unwrap();
                let id = next();
                eloop.add(stream.as_raw_fd(), id, ffi::EPOLLIN);
                connections.insert(
                    id,
                    State {
                        message: vec![0; 1024],
                        stream: stream,
                    },
                );
            } else {
                handle_socket_event(&eloop, event, &mut connections);
            }
        }
    }
}

fn handle_socket_event(
    eloop: &EventLoop,
    event: ffi::Event,
    connections: &mut HashMap<i32, State>,
) {
    if event.is_read_ready() {
        println!("boran");
        let state = connections.get_mut(&event.token).unwrap();
        state.stream.read(&mut state.message[0..1023]).unwrap();
        println!("{}", std::str::from_utf8(&state.message).unwrap());
        eloop
            .modify(&state.stream, event.token, ffi::EPOLLOOUT)
            .unwrap()
    } else {
        let mut state = connections.get_mut(&event.token).unwrap();
        state.stream.write(&state.message).unwrap();
        eloop
            .modify(&state.stream, event.token, ffi::EPOLLIN)
            .unwrap()
    }
}

//#[test]
fn client() -> Result<(), std::io::Error> {
    let mut stream = MyTcpStream::connect("slowwly.robertomurray.co.uk:80");
    let mut eloop = EventLoop::new()?;
    eloop.add(stream.as_raw_fd(), 1, ffi::EPOLLOOUT)?;
    let mut buff: [u8; 1024] = [0; 1024];

    loop {
        let events = eloop.wait()?;

        println!("{:?}", events.len());

        for event in events {
            println!("{:?}", event);
            if event.is_read_ready() {
                stream.read(&mut buff)?;
                println!("{}", String::from_utf8_lossy(&buff));
                eloop.remove(&stream)?;
            } else if event.is_write_ready() {
                let request = "GET /\r\n\
             Host: www.google.com\r\n\
             Connection: close\r\n\
             \r\n";

                stream.write(request.as_bytes()).unwrap();

                eloop.modify(&stream, event.token, ffi::EPOLLIN).unwrap();
            }
        }
    }
    Ok(())
}
