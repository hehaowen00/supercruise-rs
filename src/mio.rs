use crate::codec::http::Http;
use crate::codec::{Decoder, Encoder};
use crate::context::Body;
use bytes::BytesMut;
use mio::net::TcpListener;
use mio::{event::Event, Events, Interest, Poll, Registry, Token};
use slab::Slab;
use std::io::{Read, Write};
use std::net::SocketAddr;

const SERVER: Token = Token(0);
static BODY: &str = include_str!("../examples/chat.html");

pub fn spawn() {
    let addr = "0.0.0.0:8080";

    let addr = match addr.parse() {
        Ok(addr) => addr,
        Err(_) => {
            log::error!("failed to parse host address");
            return;
        }
    };

    log::info!("starting server on {}", addr);

    let handles: Vec<_> = (0..num_cpus::get()).map(|_| spawn_worker(addr)).collect();
    handles.into_iter().for_each(|h| h.join().unwrap());
}

fn spawn_worker(addr: SocketAddr) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || loop {
        let res = mio_worker(addr);
        log::error!("error runtime {:?}", res);
    })
}

pub fn mio_worker(addr: SocketAddr) {
    let socket = socket2::Socket::new(
        match addr {
            SocketAddr::V4(_) => socket2::Domain::IPV4,
            SocketAddr::V6(_) => socket2::Domain::IPV6,
        },
        socket2::Type::STREAM,
        None,
    )
    .unwrap();

    socket.set_reuse_address(true).unwrap();
    socket.set_reuse_port(true).unwrap();
    socket.set_nonblocking(true).unwrap();
    socket.bind(&addr.into()).unwrap();
    socket.listen(8192).unwrap();

    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1);

    let mut listener = TcpListener::from_std(socket.into());

    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)
        .unwrap();

    let mut connections = Slab::new();
    let mut sessions = Slab::new();
    let mut buffers = Slab::new();

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                SERVER => match listener.accept() {
                    Ok((socket, _addr)) => {
                        socket.set_nodelay(true).unwrap();
                        let idx = connections.insert(socket);
                        sessions.insert(false);
                        buffers.insert(([0; 8192], 0));

                        let _ = poll.registry().register(
                            &mut connections[idx],
                            Token(token_id(idx)),
                            Interest::READABLE.add(Interest::WRITABLE),
                        );
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => {
                        log::error!("{}", e);
                        return;
                    }
                },
                Token(id) => {
                    let mut conn = &mut connections[conn_id(id)];
                    let mut session = &mut sessions[conn_id(id)];
                    let mut buffer = &mut buffers[conn_id(id)];

                    match handle_conn(&mut conn, &mut session, &mut buffer, poll.registry(), event)
                    {
                        Ok(true) => {
                            let mut conn = connections.remove(conn_id(id));
                            sessions.remove(conn_id(id));
                            buffers.remove(conn_id(id));
                            poll.registry().deregister(&mut conn).unwrap();
                        }
                        Ok(false) => {
                            // poll.registry()
                            //     .reregister(
                            //         conn,
                            //         event.token(),
                            //         Interest::READABLE.add(Interest::WRITABLE),
                            //     )
                            //     .unwrap();
                        }
                        Err(_) => {
                            let mut conn = connections.remove(conn_id(id));
                            sessions.remove(conn_id(id));
                            buffers.remove(conn_id(id));
                            poll.registry().deregister(&mut conn).unwrap();
                        }
                    }
                }
            }
        }

        poll.registry()
            .reregister(&mut listener, SERVER, Interest::READABLE)
            .unwrap();
    }
}

const fn token_id(id: usize) -> usize {
    id + 1
}

const fn conn_id(id: usize) -> usize {
    id - 1
}

fn handle_conn(
    conn: &mut mio::net::TcpStream,
    session: &mut bool,
    buffer: &mut ([u8; 8192], usize),
    registry: &Registry,
    event: &Event,
) -> std::io::Result<bool> {
    if event.is_readable() {
        let read = conn.read(&mut buffer.0[buffer.1..])?;
        buffer.1 += read;

        if read == 0 {
            return Ok(true);
        }

        let mut http = Http::<Body>::new();
        let mut bytes = BytesMut::from(&buffer.0[..buffer.1]);

        match http.decode(&mut bytes) {
            Ok(Some(req)) => {
                if req.uri().path() != "/" {
                    return Ok(false);
                }
                *session = true;
            }
            Ok(None) => return Ok(false),
            Err(_) => {
                log::error!("failed to parse request");
                return Ok(true);
            }
        }
    }

    if *session && event.is_writable() {
        let resp: http::Response<Body> = http::Response::builder()
            .status(http::StatusCode::OK)
            .header(http::header::CONNECTION, "keep-alive")
            .header(http::header::CONTENT_TYPE, "text/html")
            .body(BODY.into())
            .unwrap();

        let mut http = Http::<Body>::new();
        let mut output = BytesMut::new();
        http.encode(resp, &mut output).unwrap();

        match conn.write(&output) {
            Ok(n) if n < output.len() => {
                return Err(std::io::ErrorKind::WriteZero.into());
            }
            Ok(_) => {
                registry.reregister(
                    conn,
                    event.token(),
                    Interest::READABLE.add(Interest::WRITABLE),
                )?;
            }
            Err(ref err) if would_block(err) => {}
            Err(ref err) if interrupted(err) => {}
            Err(err) => return Err(err),
        }
    }

    Ok(false)
}

fn would_block(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::WouldBlock
}

fn interrupted(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::Interrupted
}
