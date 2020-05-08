use crate::epoll::*;
use crate::libuv::*;
use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};
use futures::{AsyncRead, AsyncWrite};
use std::pin::Pin;

use io::Read;
use std::io;
use std::net;
use std::os::unix::io::AsRawFd;

use std::{
    future::Future,
    net::ToSocketAddrs,
    sync::{Arc, Mutex},
    task::Poll,
};

use std::{
    cell::RefCell,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    task::Context,
};

thread_local! {
  static Runtime: Runtime = Runtime::new();
}
pub struct TcpListener(net::TcpListener);
impl TcpListener {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> TcpListener {
        let listener = net::TcpListener::bind(addr).unwrap();
        listener
            .set_nonblocking(true)
            .expect("Cannot set non-blocking");
        TcpListener(listener)
    }
    pub fn accept(&self) -> TcpStreamFuture {
        TcpStreamFuture(&self)
    }
    pub fn accept_helper(&self) -> Result<TcpStream, std::io::Error> {
        let (stream, _) = self.0.accept()?;
        stream.set_nonblocking(true)?;
        Ok(TcpStream(stream))
    }
}

pub struct TcpStreamFuture<'a>(&'a TcpListener);

impl<'a> Future for TcpStreamFuture<'a> {
    type Output = TcpStream;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let waker = cx.waker().clone();
        println!("new f");
        match self.0.accept_helper() {
            Ok(s) => Poll::Ready(s),
            Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                Runtime.with(|r| {
                    let p = self.0;
                    r.add(p.0.as_raw_fd(), ffi::EPOLLIN, move |runtime, token| {
                        println!("s");
                        waker.wake_by_ref();
                    })
                });
                Poll::Pending
            }
            Err(err) => panic!("error {:?}", err),
        }
    }
}

pub struct TcpStream(pub std::net::TcpStream);

impl TcpStream {
    pub fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> ReadHelper<'a, Self>
    where
        Self: Unpin,
    {
        ReadHelper::new(self, buf)
    }
}

pub struct ReadHelper<'a, R: ?Sized> {
    reader: &'a mut R,
    buf: &'a mut [u8],
}
impl<'a, R: AsyncRead + ?Sized + Unpin> ReadHelper<'a, R> {
    pub(super) fn new(reader: &'a mut R, buf: &'a mut [u8]) -> Self {
        ReadHelper { reader, buf }
    }
}

impl<R: AsyncRead + ?Sized + Unpin> Future for ReadHelper<'_, R> {
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        Pin::new(&mut this.reader).poll_read(cx, this.buf)
    }
}

impl AsyncRead for TcpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let waker = cx.waker().clone();
        println!("www");

        match self.0.read(buf) {
            Ok(s) => {
                println!("ready");
                Poll::Ready(Ok(s))
            }
            Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                Runtime.with(|r| {
                    r.add(self.0.as_raw_fd(), ffi::EPOLLIN, move |runtime, token| {
                        println!("scheduled");
                        waker.wake_by_ref()
                    })
                });
                Poll::Pending
            }
            Err(err) => panic!("error {:?}", err),
        }
    }
}

impl Executor {
    pub fn run(&self) {
        //Runtime.with(|r| r.run());
        Runtime.with(|r| loop {
            while let Ok(task) = self.0.try_recv() {
                println!("task is ready");
                let mut future_slot = task.future.lock().unwrap();
                if let Some(mut future) = future_slot.take() {
                    let waker = waker_ref(&task);
                    let context = &mut Context::from_waker(&*waker);
                    if let Poll::Pending = future.as_mut().poll(context) {
                        *future_slot = Some(future);
                    }
                }
            }
            let events = r.eventLoop.wait().unwrap();
            for event in events {
                if let Some(f) = unsafe { (&mut *r.callbacks.get()).get_mut(&event.token) } {
                    f(&r, event.token)
                }
            }
        });
    }
}

pub struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    task_sender: SyncSender<Arc<Task>>,
}

pub struct Executor(Receiver<Arc<Task>>);

#[derive(Clone)]
pub struct Spawner(SyncSender<Arc<Task>>);

impl Spawner {
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        println!("spawn");
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.0.clone(),
        });
        println!("spawn2");

        self.0.send(task).unwrap();
        println!("spawn3")
    }
}

pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    let (ss, r) = sync_channel(10_000);
    (Executor(r), Spawner(ss))
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        println!("asdasd");
        arc_self.task_sender.send(cloned);
        println!("asdasd2");
    }
}
