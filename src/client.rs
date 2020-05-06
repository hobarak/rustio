use crate::epoll::*;

use std::io::prelude::*;
use std::io::{self, Write};
use std::net::TcpStream;

use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
//
//#[test]
//fn client() {
//    let mut event_counter = 10;
//    let queue = EventLoop::create();
//
//    let addr = "slowwly.robertomurray.co.uk:80";
//    let mut stream = TcpStream::connect(addr).unwrap();
//    stream.set_nonblocking(true).unwrap();
//
//    queue.add_write_listener(stream.as_raw_fd());
//    event_counter += 1;
//    //let mut streams = vec![];
//    //streams.push(stream);
//
//    queue.start(|q, events| {
//        println!("asd: {:? }  ", events);
//
//        for event in events {
//            if event.is_read_ready() {
//                println!("RECEIVED: { } { }", event.epoll_data, event.events);
//                //let mut t = unsafe { TcpStream::from_raw_fd(event.epoll_data) };
//                let mut buff: [u8; 1024] = [0; 1024];
//                stream.read(&mut buff).unwrap();
//                println!("{}", std::str::from_utf8(&buff).unwrap());
//                event_counter -= 1;
//                queue.add_write_listener(event.epoll_data);
//            } else if event.is_write_ready() {
//                let delay = 1000;
//                let request = format!(
//                    "GET /delay/{}/url/http://www.google.com HTTP/1.1\r\n\
//             Host: slowwly.robertomurray.co.uk\r\n\
//             Connection: close\r\n\
//             \r\n",
//                    delay
//                );
//                //let mut t = unsafe { TcpStream::from_raw_fd(event.epoll_data) };
//                stream.write(request.as_bytes()).unwrap();
//
//                q.asd(event.epoll_data, (0x02) as u32);
//                q.add_read_listener(event.epoll_data);
//                println!("asd")
//            }
//        }
//        if event_counter == 0 {
//            return Status::Terminate;
//        } else {
//            Status::Continue
//        }
//    });
//
//    queue.close();
//
//    //println!("{}", streams.len());
//}
//
