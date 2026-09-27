use crate::dominio::contratos::{
    DatosRespuesta, PaginaWeb, Peticion, Respuesta, TipoPeticion, Usuario,
};
use crate::dominio::datagrama_tcp::Extremo;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

/// Procesa la petición recibida y devuelve una Respuesta tipada según la acción solicitada.
fn leer_buffer(stream: &mut TcpStream) -> std::io::Result<()> {
    let mut buffer = [0u8; 1024];
    let bytes_leidos = stream.read(&mut buffer)?;

    if bytes_leidos == 0 {
        return Ok(());
    }

    let Ok(peticion) = serde_json::from_slice::<Peticion>(&buffer[..bytes_leidos]) else {
        return Ok(());
    };

    let datos = match peticion.accion {
        TipoPeticion::ObtenerUsuario(id) => DatosRespuesta::Usuario(Some(Usuario {
            id,
            nombre: format!("Usuario_{}", id),
            correo: format!("usuario{}@tcpstoryteller.com", id),
        })),
        TipoPeticion::ObtenerTodosLosUsuarios => DatosRespuesta::Usuarios(vec![
            Usuario {
                id: 1,
                nombre: "Grace Hopper".to_string(),
                correo: "grace@hopper.org".to_string(),
            },
            Usuario {
                id: 2,
                nombre: "Alan Turing".to_string(),
                correo: "alan@turing.org".to_string(),
            },
            Usuario {
                id: 3,
                nombre: "Linus Torvalds".to_string(),
                correo: "linus@torvalds.com".to_string(),
            },
            Usuario {
                id: 4,
                nombre: "Fabrice Bellard".to_string(),
                correo: "fabrice@bellard.org".to_string(),
            },
        ]),
        TipoPeticion::CargarPaginaWeb(ruta) => DatosRespuesta::Web(PaginaWeb {
            ruta: ruta.clone(),
            titulo: "TCP Storyteller - Pagina Web".to_string(),
            contenido_html: format!(
                "<!DOCTYPE html><html><head><title>TCP</title></head><body><h1>Bienvenido a {}</h1><p>Conexión TCP activa y datos transmitidos en texto plano.</p></body></html>",
                ruta
            ),
        }),
    };

    let respuesta = Respuesta {
        id_secuencia: peticion.id_peticion,
        datos,
    };

    if let Ok(bytes_respuesta) = serde_json::to_vec(&respuesta) {
        stream.write_all(&bytes_respuesta)?;
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
