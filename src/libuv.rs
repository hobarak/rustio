use crate::epoll::*;

pub enum Interest {
    Read,
    Write,
}
use std::{
    cell::{Cell, UnsafeCell},
    collections::HashMap,
};
type Token = i32;

pub struct Runtime {
    pub currentToken: Cell<Token>,
    pub eventLoop: EventLoop,
    pub callbacks: UnsafeCell<HashMap<Token, Box<dyn FnMut(&Runtime, Token)>>>,
}

impl Runtime {
    fn next(&self) -> Token {
        let r = self.currentToken.get();

        self.currentToken.set(r + 1);

        self.currentToken.get()
    }

    pub fn new() -> Runtime {
        let mut eloop = EventLoop::new().unwrap();
        let r = Runtime {
            currentToken: Cell::new(0),
            eventLoop: eloop,
            callbacks: UnsafeCell::new(HashMap::new()),
        };
        r
    }

    pub fn add<F>(&self, a: i32, flags: i32, f: F)
    where
        F: FnMut(&Runtime, Token) + 'static,
    {
        let token = self.next();
        unsafe {
            (&mut *self.callbacks.get()).insert(token, Box::new(f));
        }

        self.eventLoop.add(a, token, flags).unwrap();
    }

    pub fn run(&self) {
        loop {
            let events = self.eventLoop.wait().unwrap();
            for event in events {
                if let Some(f) = unsafe { (&mut *self.callbacks.get()).get_mut(&event.token) } {
                    f(self, event.token)
                }
            }
        }
    }
}
