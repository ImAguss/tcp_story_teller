use crate::{Dominio::datagrama_tcp::Extremo, Escenarios::utiles::abrir_socket};
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

fn cliente(server: Extremo) -> std::io::Result<()> {
    let socket_server = format!("{}:{}", server.ip, server.puerto);
    let mut stream = TcpStream::connect(socket_server)?;

    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    stream.set_nodelay(true)?;

    for _ in 0..10 {
        stream.write_all(b"HOLAAAAA")?;
    }

    let mut respuesta = [0u8; 126];
    stream.read(&mut respuesta)?;

    Ok(())
}

pub fn ejecutar(server: Extremo) -> std::io::Result<()> {
    let socket_server = server.clone();
    let servidor = std::thread::spawn(move || {
        if let Err(e) = abrir_socket(socket_server, 1) {
            eprintln!("Error al iniciar servidor {}", e);
        }
    });

    std::thread::sleep(Duration::from_millis(20));

    cliente(server.clone())?;

    servidor.join().unwrap();

    Ok(())
}
