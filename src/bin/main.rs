//===----------------------------------------------------------------------===//
// main.rs
// 
// This source file is part of the hello-tcp project
//
// Copyright (c) 2020 Philippe Nadon
// Licensed under Apache License v2.0
//===----------------------------------------------------------------------===//
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use hello_tcp::ThreadPool;
use std::fs;

static ADDRESS: &str = "127.0.0.1:7878";

fn main() {
    let listener = TcpListener::bind(ADDRESS).unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

/// Handles incoming connections
/// 
/// stream: An incoming TCP stream
/// 
/// TODO: Create html "file server"
/// TODO: Handle "directory sanitization (sandbox)"
fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };
    let contents = fs::read_to_string(filename).unwrap();

    let response = format!("{}{}", status_line, contents);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}