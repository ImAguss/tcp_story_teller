use crate::escenarios::utiles::crear_stream;
use std::{
    io::{Read, Write},
    str::FromStr,
};

pub fn enviar_datos_http_simple() -> std::io::Result<()> {
    let mut buffer = [0u8; 2048];
    let pagina = String::from_str("httpbin.org:80").unwrap();

    let mut stream = match crear_stream(pagina) {
        Ok(stream) => stream,
        _ => panic!("Error al crear stream."),
    };
    let peticion_get_cerrar = "GET /get HTTP/1.1\r\nHost: httpbin.org\r\nUser-Agent: TcpStoryTeller/0.1\r\nConnection: close\r\n\r\n";

    stream.write_all(peticion_get_cerrar.as_bytes())?;

    while let Ok(bytes_leidos) = stream.read(&mut buffer) {
        if bytes_leidos == 0 {
            break;
        }
    }

    Ok(())
}

pub fn enviar_datos_http_pesado() -> std::io::Result<()> {
    let pagina = String::from_str("httpbin.org:80").unwrap();
    let mut stream =
        crear_stream(pagina).map_err(|_| std::io::Error::other("Fallo al crear stream"))?;

    let cuerpo = "Hola Mundo desde Tcp Story Teller!".repeat(20000);
    let peticion_post = format!(
        "POST /post HTTP/1.1\r\n\
        Host: httpbin.org\r\n\
        User-Agent: TcpStoryTeller/0.1\r\n\
        Connection: close\r\n\
        Content-Type: text/plain\r\n\
        Content-Length: {}\r\n\r\n\
        {}",
        cuerpo.len(),
        cuerpo,
    );
    stream.write_all(peticion_post.as_bytes())?;

    let mut buffer = [0u8; 4096];
    while let Ok(bytes_leidos) = stream.read(&mut buffer) {
        if bytes_leidos == 0 {
            break;
        }
    }

    Ok(())
}

pub fn enviar_datos_http_rafaga() -> std::io::Result<()> {
    let pagina = String::from_str("httpbin.org:80").unwrap();
    let mut stream =
        crear_stream(pagina).map_err(|_| std::io::Error::other("Fallo al crear stream"))?;

    let peticion_get = "GET /get HTTP/1.1\r\nHost: httpbin.org\r\nUser-Agent: TcpStoryTeller/0.1\r\nConnection: keep-alive\r\n\r\n";
    let cuerpo = "Hola Mundo desde Tcp Story Teller!";
    let peticion_post = format!(
        "POST /post HTTP/1.1\r\n\
        Host: httpbin.org\r\n\
        User-Agent: TcpStoryTeller/0.1\r\n\
        Connection: keep-alive\r\n\
        Content-Type: text/plain\r\n\
        Content-Length: {}\r\n\r\n\
        {}",
        cuerpo.len(),
        cuerpo,
    );
    let peticion_get_cerrar = "GET /get HTTP/1.1\r\nHost: httpbin.org\r\nUser-Agent: TcpStoryTeller/0.1\r\nConnection: close\r\n\r\n";
    stream.set_nodelay(true)?;
    stream.write_all(peticion_get.as_bytes())?;
    stream.write_all(peticion_post.as_bytes())?;
    stream.write_all(peticion_get_cerrar.as_bytes())?;

    let mut buffer = [0u8; 4096];
    while let Ok(bytes_leidos) = stream.read(&mut buffer) {
        if bytes_leidos == 0 {
            break;
        }
    }

    Ok(())
}
