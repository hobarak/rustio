use futures::{AsyncRead, AsyncWrite};
use rustio::futur::*;
use std::sync::{Arc, Mutex};

async fn start(spawner: Spawner) {
    let mut listener = TcpListener::bind("127.0.0.1:7879");
    loop {
        let stream = listener.accept().await;
        println!("2222");
        spawner.spawn(talk(stream));
    }
}

async fn talk(mut stream: TcpStream) {
    let mut buf = vec![0; 10];
    println!("22221");

    let _ = stream.read(&mut buf).await;
    println!("22222");

    println!("{}", String::from_utf8_lossy(&buf));
}

#[test]
fn server() {
    let (e, s) =  new_executor_and_spawner();
    s.spawn(start(s.clone()));
    e.run();
}
