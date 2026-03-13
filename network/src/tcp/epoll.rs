use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::fd::{AsRawFd, RawFd};

use nix::errno::Errno;
use nix::sys::epoll::{
    EpollCreateFlags, EpollEvent, EpollFlags, EpollOp, epoll_create1, epoll_ctl, epoll_wait,
};
use nix::unistd::close;

const MAX_EVENTS: usize = 1024;
const LISTENER_TOKEN: u64 = 1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "0.0.0.0:9000".parse()?;
    let listener = TcpListener::bind(addr)?;
    listener.set_nonblocking(true)?;

    println!("[INFO] listening on {}", addr);

    let listener_fd = listener.as_raw_fd();
    let epoll_fd = epoll_create1(EpollCreateFlags::EPOLL_CLOEXEC)?;

    let mut listener_event =
        EpollEvent::new(EpollFlags::EPOLLIN, LISTENER_TOKEN);
    epoll_ctl(epoll_fd, EpollOp::EpollCtlAdd, listener_fd, &mut listener_event)?;

    let mut next_token: u64 = LISTENER_TOKEN + 1;
    let mut clients: HashMap<RawFd, Client> = HashMap::new();
    let mut token_to_fd: HashMap<u64, RawFd> = HashMap::new();

    let mut events = vec![EpollEvent::empty(); MAX_EVENTS];

    loop {
        let nfds = epoll_wait(epoll_fd, &mut events, -1)?;

        for event in events.iter().take(nfds) {
            let token = event.data();
            let flags = event.events();

            if token == LISTENER_TOKEN {
                loop {
                    match listener.accept() {
                        Ok((stream, peer_addr)) => {
                            if let Err(e) =
                                handle_new_client(stream, peer_addr, epoll_fd, &mut next_token, &mut clients, &mut token_to_fd)
                            {
                                eprintln!("[ERROR] accept handling failed: {}", e);
                            }
                        }
                        Err(e) if e.kind() == ErrorKind::WouldBlock => {
                            break;
                        }
                        Err(e) => {
                            eprintln!("[ERROR] accept failed: {}", e);
                            break;
                        }
                    }
                }
                continue;
            }

            let Some(&fd) = token_to_fd.get(&token) else {
                continue;
            };

            let mut should_close = false;

            if flags.contains(EpollFlags::EPOLLERR)
                || flags.contains(EpollFlags::EPOLLHUP)
                || flags.contains(EpollFlags::EPOLLRDHUP)
            {
                should_close = true;
            } else {
                if let Some(client) = clients.get_mut(&fd) {
                    if flags.contains(EpollFlags::EPOLLIN) {
                        match read_from_client(client) {
                            Ok(ReadResult::Data(data)) => {
                                println!(
                                    "[RECV] fd={} peer={} bytes={} msg={}",
                                    client.fd,
                                    client.peer_addr,
                                    data.len(),
                                    String::from_utf8_lossy(&data).trim_end()
                                );
                                client.write_buf.extend_from_slice(&data);
                            }
                            Ok(ReadResult::Closed) => {
                                should_close = true;
                            }
                            Ok(ReadResult::WouldBlock) => {}
                            Err(e) => {
                                eprintln!("[ERROR] read failed fd={}: {}", client.fd, e);
                                should_close = true;
                            }
                        }
                    }

                    if !client.write_buf.is_empty() {
                        if let Err(e) = modify_interest(
                            epoll_fd,
                            client.fd,
                            client.token,
                            EpollFlags::EPOLLIN
                                | EpollFlags::EPOLLOUT
                                | EpollFlags::EPOLLRDHUP,
                        ) {
                            eprintln!("[ERROR] modify interest failed fd={}: {}", client.fd, e);
                            should_close = true;
                        }
                    }

                    if !should_close && flags.contains(EpollFlags::EPOLLOUT) {
                        match write_to_client(client) {
                            Ok(()) => {
                                let interest = if client.write_buf.is_empty() {
                                    EpollFlags::EPOLLIN | EpollFlags::EPOLLRDHUP
                                } else {
                                    EpollFlags::EPOLLIN
                                        | EpollFlags::EPOLLOUT
                                        | EpollFlags::EPOLLRDHUP
                                };

                                if let Err(e) =
                                    modify_interest(epoll_fd, client.fd, client.token, interest)
                                {
                                    eprintln!(
                                        "[ERROR] modify interest failed fd={}: {}",
                                        client.fd, e
                                    );
                                    should_close = true;
                                }
                            }
                            Err(e) => {
                                eprintln!("[ERROR] write failed fd={}: {}", client.fd, e);
                                should_close = true;
                            }
                        }
                    }
                }
            }

            if should_close {
                if let Some(client) = clients.remove(&fd) {
                    println!("[INFO] disconnect fd={} peer={}", client.fd, client.peer_addr);
                    token_to_fd.remove(&client.token);
                    let _ = epoll_ctl(epoll_fd, EpollOp::EpollCtlDel, client.fd, None);
                    let _ = close(client.fd);
                }
            }
        }
    }
}

struct Client {
    fd: RawFd,
    token: u64,
    peer_addr: SocketAddr,
    stream: TcpStream,
    write_buf: Vec<u8>,
}

enum ReadResult {
    Data(Vec<u8>),
    Closed,
    WouldBlock,
}

fn handle_new_client(
    stream: TcpStream,
    peer_addr: SocketAddr,
    epoll_fd: RawFd,
    next_token: &mut u64,
    clients: &mut HashMap<RawFd, Client>,
    token_to_fd: &mut HashMap<u64, RawFd>,
) -> Result<(), Box<dyn std::error::Error>> {
    stream.set_nonblocking(true)?;
    let fd = stream.as_raw_fd();
    let token = *next_token;
    *next_token += 1;

    let mut event = EpollEvent::new(
        EpollFlags::EPOLLIN | EpollFlags::EPOLLRDHUP,
        token,
    );
    epoll_ctl(epoll_fd, EpollOp::EpollCtlAdd, fd, &mut event)?;

    println!("[INFO] new client fd={} peer={}", fd, peer_addr);

    clients.insert(
        fd,
        Client {
            fd,
            token,
            peer_addr,
            stream,
            write_buf: Vec::new(),
        },
    );
    token_to_fd.insert(token, fd);

    Ok(())
}

fn read_from_client(client: &mut Client) -> Result<ReadResult, Box<dyn std::error::Error>> {
    let mut buf = [0u8; 4096];
    let mut out = Vec::new();

    loop {
        match client.stream.read(&mut buf) {
            Ok(0) => {
                if out.is_empty() {
                    return Ok(ReadResult::Closed);
                } else {
                    return Ok(ReadResult::Data(out));
                }
            }
            Ok(n) => {
                out.extend_from_slice(&buf[..n]);
                if n < buf.len() {
                    return Ok(ReadResult::Data(out));
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                if out.is_empty() {
                    return Ok(ReadResult::WouldBlock);
                } else {
                    return Ok(ReadResult::Data(out));
                }
            }
            Err(e) => return Err(Box::new(e)),
        }
    }
}

fn write_to_client(client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
    while !client.write_buf.is_empty() {
        match client.stream.write(&client.write_buf) {
            Ok(0) => {
                return Err("write returned 0".into());
            }
            Ok(n) => {
                client.write_buf.drain(..n);
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                break;
            }
            Err(e) => return Err(Box::new(e)),
        }
    }
    Ok(())
}

fn modify_interest(
    epoll_fd: RawFd,
    fd: RawFd,
    token: u64,
    flags: EpollFlags,
) -> nix::Result<()> {
    let mut event = EpollEvent::new(flags, token);
    epoll_ctl(epoll_fd, EpollOp::EpollCtlMod, fd, &mut event)
}