use crate::epoll::*;

use std::collections::HashMap;
use std::io::prelude::*;

struct State {
    message: Vec<u8>,
    stream: MyTcpStream,
}

#[test]
pub fn server() {
    let mut eloop = EventLoop::new();
    let mut listener = MyTCPListener::bind("127.0.0.1:7878");
    let mut connections = HashMap::new();

    let mut counter = 0;
    let mut next = || {
        counter += 1;
        counter
    };
    let server = next();
    eloop.add(&listener, server, ffi::EPOLLIN);

    loop {
        let events = eloop.wait();
        for event in events {
            if event.is_read_ready() && event.token == server {
                let stream = listener.accept();
                let id = next();
                eloop.add(&stream, id, ffi::EPOLLIN);
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
        eloop.modify(&state.stream, event.token, ffi::EPOLLOOUT)
    } else {
        let mut state = connections.get_mut(&event.token).unwrap();
        state.stream.write(&state.message).unwrap();
        eloop.modify(&state.stream, event.token, ffi::EPOLLIN)
    }
}
