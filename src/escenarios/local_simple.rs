use crate::{
    dominio::{contratos::Peticion, datagrama_tcp::Extremo},
    escenarios::utiles::{abrir_socket, enviar_datos},
};
use std::{net::TcpStream, time::Duration};

fn cliente(server: Extremo, peticion: Peticion) -> std::io::Result<()> {
    let socket_server = format!("{}:{}", server.ip, server.puerto);
    let mut stream = TcpStream::connect(socket_server)?;

    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    stream.set_nodelay(true)?;

    enviar_datos(&mut stream, peticion)?;

    Ok(())
}

pub fn ejecutar(server: Extremo) -> std::io::Result<()> {
    let socket_server = server.clone();
    let servidor = std::thread::spawn(move || {
        if let Err(e) = abrir_socket(socket_server, 1) {
            eprintln!("Error al iniciar servidor {}", e);
        }
    });

    std::thread::sleep(Duration::from_millis(120));

    let peticion = Peticion {
        id_peticion: 32,
        accion: crate::dominio::contratos::TipoPeticion::ObtenerUsuario(3),
    };
    cliente(server, peticion)?;

    servidor.join().unwrap();

    Ok(())
}
