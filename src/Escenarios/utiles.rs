use crate::Dominio::contratos::{
    DatosRespuesta, PaginaWeb, Peticion, Respuesta, TipoPeticion, Usuario,
};
use crate::Dominio::datagrama_tcp::Extremo;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

/// Procesa la petición recibida y devuelve una Respuesta tipada según la acción solicitada.
fn leer_buffer(stream: &mut TcpStream) -> std::io::Result<()> {
    let mut buffer = [0u8; 512];
    let bytes_leidos = stream.read(&mut buffer)?;

    if bytes_leidos > 0 {
        stream.write_all(&buffer[..bytes_leidos])?;
        stream.flush()?;
    }

    Ok(())
}

/// Envía una petición serializada y lee la respuesta del servidor.
pub fn enviar_datos(stream: &mut TcpStream, peticion: Peticion) -> std::io::Result<()> {
    let bytes = serde_json::to_vec(&peticion)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    stream.write_all(&bytes)?;
    stream.flush()?;

    let mut buffer = [0u8; 2048];
    let _ = stream.read(&mut buffer)?;

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
