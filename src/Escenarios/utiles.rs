use crate::Dominio::datagrama_tcp::Extremo;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn leer_buffer(stream: &mut TcpStream) -> std::io::Result<()> {
    let mut buffer = [0u8; 512];
    let bytes_leidos = stream.read(&mut buffer)?;

    if bytes_leidos > 0 {
        stream.write_all(&buffer[..bytes_leidos])?;
        stream.flush()?;
    }

    Ok(())
}

pub fn abrir_socket(socket: Extremo, limite_conexiones: usize) -> std::io::Result<()> {
    let socket = format!("{}:{}", socket.ip, socket.puerto);
    let listener = TcpListener::bind(socket)?;

    println!("Escuchando en {}", listener.local_addr()?);

    for conexion in listener.incoming().take(limite_conexiones) {
        match conexion {
            Ok(mut stream) => {
                std::thread::spawn(move || {
                    if let Err(e) = leer_buffer(&mut stream) {
                        eprintln!("Error en cliente {}", e);
                    }
                });
            }
            Err(e) => eprintln!("Error al recibir la conexion {}", e),
        };
    }

    Ok(())
}
