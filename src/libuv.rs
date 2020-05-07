use crate::epoll::*;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::sync::{mpsc, mpsc::channel, Arc, Mutex};

pub enum Interest {
    Read,
    Write,
}
use std::{
    cell::{Cell, RefCell, UnsafeCell},
    collections::HashMap,
};
type Token = i32;

pub struct Runtime {
    currentToken: Cell<Token>,
    eventLoop: EventLoop,
    callbacks: UnsafeCell<HashMap<Token, Box<dyn FnMut(&Runtime, Token)>>>,
}

impl Runtime {
    fn next(&self) -> Token {
        let r = self.currentToken.get();

        self.currentToken.set(r + 1);

        self.currentToken.get()
    }

    pub fn new() -> Runtime {
        let mut eloop = EventLoop::new();

        println!("boran1");

        let r = Runtime {
            currentToken: Cell::new(0),
            eventLoop: eloop,
            callbacks: UnsafeCell::new(HashMap::new()),
        };
        r
    }

    pub fn add<F>(&self, a: i32, i: Interest, f: F)
    where
        F: FnMut(&Runtime, Token) + 'static,
    {
        println!("boran3");

        let token = self.next();
        unsafe {
            (&mut *self.callbacks.get()).insert(token, Box::new(f));
        }
        println!("boran4");

        self.eventLoop.add(a, token, ffi::EPOLLIN);
    }

    pub fn run(&mut self) {
        loop {
            let events = self.eventLoop.wait();
            for event in events {
                if let Some(f) = unsafe { (&mut *self.callbacks.get()).get_mut(&event.token) } {
                    f(self, event.token)
                }
            }
        }
    }
}
