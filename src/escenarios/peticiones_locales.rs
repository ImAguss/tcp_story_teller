use crate::{
    dominio::{
        contratos::{Peticion, TipoPeticion},
        datagrama_tcp::Extremo,
    },
    escenarios::utiles::{abrir_socket, crear_stream, enviar_datos},
};
use std::time::Duration;

pub fn ejecutar_localmente(server: Extremo, local: bool) -> std::io::Result<()> {
    let servidor = if local {
        let socket_server = server.clone();
        let handle = std::thread::spawn(move || {
            if let Err(e) = abrir_socket(socket_server) {
                eprintln!("Error al iniciar servidor {}", e);
            }
        });
        std::thread::sleep(Duration::from_millis(120));
        Some(handle)
    } else {
        None
    };

    let socket = format!("{}:{}", server.ip, server.puerto);
    let mut stream = crear_stream(socket)?;
    let peticion = Peticion {
        id_peticion: 45,
        accion: TipoPeticion::ObtenerTodosLosUsuarios,
    };
    enviar_datos(&mut stream, peticion)?;

    if let Some(handle) = servidor {
        handle.join().unwrap();
    }

    Ok(())
}
