use std::{io::{Read, Write}, net::*};

use num_traits::ops::bytes;

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 100];
    let n = stream.read(&mut buffer[..])?;
    println!("The bytes: {:?}", &buffer[..n]);
    
    Ok(())
}

fn listen() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:80")?;

    for stream in listener.incoming() {
        handle_client(stream?);
    }

    Ok(())
}

fn connect() -> std::io::Result<()> {
    let mut client = TcpStream::connect("127.0.0.1:80")?;

    let message = "Hello world!".as_bytes();
    client.write(message)?;
    
    Ok(())
}

fn main() {
    listen();
    connect();
}