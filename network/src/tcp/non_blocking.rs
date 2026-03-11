use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::SocketAddr;

const SERVER: Token = Token(0);

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;

    let mut listener = TcpListener::bind(addr)?;
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(128);

    // listener를 poll에 등록
    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)?;

    println!("[SERVER] epoll-style echo server listening on 127.0.0.1:8080");

    let mut unique_token_id = 1usize;
    let mut clients: HashMap<Token, TcpStream> = HashMap::new();

    loop {
        // 이벤트가 생길 때까지 대기
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    loop {
                        match listener.accept() {
                            Ok((mut stream, addr)) => {
                                let token = Token(unique_token_id);
                                unique_token_id += 1;

                                println!("[SERVER] Client connected: {} -> token {:?}", addr, token);

                                poll.registry().register(
                                    &mut stream,
                                    token,
                                    Interest::READABLE,
                                )?;

                                clients.insert(token, stream);
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                break;
                            }
                            Err(e) => {
                                eprintln!("[SERVER] Accept error: {}", e);
                                break;
                            }
                        }
                    }
                }

                token => {
                    let mut disconnected = false;

                    if let Some(stream) = clients.get_mut(&token) {
                        let mut buffer = [0u8; 1024];

                        match stream.read(&mut buffer) {
                            Ok(0) => {
                                println!("[SERVER] Client disconnected: {:?}", token);
                                disconnected = true;
                            }
                            Ok(n) => {
                                let received = &buffer[..n];
                                println!(
                                    "[SERVER] Received from {:?}: {}",
                                    token,
                                    String::from_utf8_lossy(received)
                                );

                                match stream.write_all(received) {
                                    Ok(_) => {}
                                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                        println!("[SERVER] Write would block for {:?}", token);
                                    }
                                    Err(e) => {
                                        eprintln!("[SERVER] Write error for {:?}: {}", token, e);
                                        disconnected = true;
                                    }
                                }
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                // 지금 읽을 데이터 없음
                            }
                            Err(e) => {
                                eprintln!("[SERVER] Read error for {:?}: {}", token, e);
                                disconnected = true;
                            }
                        }
                    }

                    if disconnected {
                        if let Some(mut stream) = clients.remove(&token) {
                            let _ = poll.registry().deregister(&mut stream);
                        }
                    }
                }
            }
        }
    }
}