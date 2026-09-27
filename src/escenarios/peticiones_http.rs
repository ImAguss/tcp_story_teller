use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

const GET: &str = "GET /get HTTP/1.1\r\nHost: httpbin.org\r\nUser-Agent: TcpStoryTeller/0.1\r\nConnection: close\r\n\r\n";
const POST: &str = "POST /post HTTP/1.1\r\nHost: httpbin.org\r\nUser-Agent: TcpStoryTeller/0.1\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: 45\r\n\r\n{\"ping\": \"pong\", \"cliente\": \"TcpStoryTeller\"}";

pub fn enviar_datos_http() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("httpbin.org:80")?;
    let mut buffer = [0u8; 2048];

    stream.set_read_timeout(Some(Duration::from_secs(20)))?;
    stream.set_write_timeout(Some(Duration::from_secs(20)))?;
    stream.set_nodelay(true)?;

    stream.write_all(GET.as_bytes())?;
    stream.flush()?;

    let _ = stream.read(&mut buffer)?;

    Ok(())
}
