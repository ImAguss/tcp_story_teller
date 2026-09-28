use crate::escenarios::utiles::crear_stream;
use std::{
    io::{Read, Write},
    str::FromStr,
};

const GET: &str = "GET /get HTTP/1.1\r\nHost: httpbin.org\r\nUser-Agent: TcpStoryTeller/0.1\r\nConnection: close\r\n\r\n";

pub fn enviar_datos_http_simple() -> std::io::Result<()> {
    let mut buffer = [0u8; 2048];
    let pagina = String::from_str("httpbin.org:80").unwrap();

    let mut stream = match crear_stream(pagina) {
        Ok(stream) => stream,
        _ => panic!("Error al crear stream."),
    };

    stream.write_all(GET.as_bytes())?;

    let _ = stream.read(&mut buffer)?;

    Ok(())
}

pub fn enviar_datos_http_rafaga() -> std::io::Result<()> {
    let pagina = String::from_str("httpbin.org:80").unwrap();
    let mut stream = match crear_stream(pagina) {
        Ok(stream) => stream,
        _ => panic!("Error al crear stream."),
    };

    let cuerpo = "HOLA".repeat(20000);
    let peticion = format!(
        "POST /post HTTP/1.1\r\n\
         Host: httpbin.org\r\n\
         User-Agent: TcpStoryTeller/0.1\r\n\
         Connection: close\r\n\
         Content-Type: text/plain\r\n\
         Content-Length: {}\r\n\r\n\
         {}",
        cuerpo.len(),
        cuerpo
    );

    stream.write_all(peticion.as_bytes())?;

    let mut buffer = [0u8; 4096];
    while let Ok(bytes_leidos) = stream.read(&mut buffer) {
        if bytes_leidos == 0 {
            break;
        }
    }

    Ok(())
}
