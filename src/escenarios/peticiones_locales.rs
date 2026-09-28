use crate::{
    dominio::{
        contratos::{Peticion, TipoPeticion},
        datagrama_tcp::Extremo,
    },
    escenarios::utiles::{abrir_socket, crear_stream, enviar_datos},
};
use std::{
    io::{Read, Write},
    net::TcpStream,
    thread,
    time::Duration,
};

fn cliente(server: Extremo, peticiones: Vec<Peticion>) -> std::io::Result<()> {
    let socket_server = format!("{}:{}", server.ip, server.puerto);
    let mut buffer = [0u8; 2048];
    let mut stream = crear_stream(socket_server)?;

    for peticion in peticiones {
        let peticion_parseada = serde_json::to_vec(&peticion);
        if let Ok(bytes) = peticion_parseada {
            stream.write_all(&bytes)?;
            stream.flush()?;
        }
    }
    let _ = stream.read(&mut buffer)?;
    Ok(())
}

fn ejecutar_cliente(server: Extremo) -> std::io::Result<()> {
    std::thread::sleep(Duration::from_millis(120));

    let peticion1 = Peticion {
        id_peticion: 32,
        accion: TipoPeticion::ObtenerUsuario(3),
    };
    let peticion2 = Peticion {
        id_peticion: 32,
        accion: TipoPeticion::ObtenerTodosLosUsuarios,
    };
    let peticion3 = Peticion {
        id_peticion: 32,
        accion: TipoPeticion::CargarPaginaWeb("/index".to_string()),
    };
    let peticiones = vec![peticion1, peticion2, peticion3];

    cliente(server, peticiones)?;
    Ok(())
}

pub fn ejecutar_localmente(server: Extremo, local: bool) -> std::io::Result<()> {
    if local {
        let socket_server = server.clone();
        let servidor = std::thread::spawn(move || {
            if let Err(e) = abrir_socket(socket_server) {
                eprintln!("Error al iniciar servidor {}", e);
            }
        });
        ejecutar_cliente(server);
        servidor.join().unwrap();
    } else {
        let socket = format!("{}:{}", server.ip, server.puerto);
        let mut stream = crear_stream(socket).unwrap();
        let peticion = Peticion {
            id_peticion: 45,
            accion: TipoPeticion::ObtenerTodosLosUsuarios,
        };
        enviar_datos(&mut stream, peticion)?;
    }

    Ok(())
}
