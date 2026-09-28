mod captura;
mod dominio;
mod escenarios;
mod presentacion;

use std::net::Ipv4Addr;
use std::sync::mpsc;
use std::time::Duration;

use crate::captura::captura_paquetes::capturar_paquetes;
use crate::captura::errores::ErrorCaptura;
pub use crate::dominio::datagrama_tcp::Extremo;
use crate::dominio::estados::InformeSesion;
pub use crate::dominio::visual::PasoTCP;
use crate::escenarios::peticiones_http::{enviar_datos_http_rafaga, enviar_datos_http_simple};
use crate::escenarios::peticiones_locales::ejecutar_localmente;
use crate::escenarios::utiles::abrir_socket;
use crate::presentacion::presentacion::ejecutar_presentacion;

pub fn escucha_pasiva(
    puerto: u16,
    interfaz: Option<String>,
) -> Result<InformeSesion, ErrorCaptura> {
    println!("Escuchando en el puerto {}...", puerto);
    let informe = match capturar_paquetes(interfaz, puerto) {
        Ok(i) => i,
        Err(_) => return Err(ErrorCaptura::ErrorCapturador),
    };

    Ok(informe)
}

pub fn ejecutar_escenario_web_simple(
    interfaz: Option<String>,
) -> Result<InformeSesion, ErrorCaptura> {
    let (tx, rx) = mpsc::channel();

    let _handlesniffer = std::thread::spawn(move || {
        let _informe = match capturar_paquetes(interfaz, 80) {
            Ok(i) => tx.send(i),
            Err(_) => panic!("Error al capturar paquetes."),
        };
    });

    std::thread::sleep(Duration::from_millis(500));
    if let Err(e) = enviar_datos_http_simple() {
        eprintln!("Error al enviar peticion HTTP: {}", e);
    }

    let informe = match rx.recv() {
        Ok(i) => Ok(i),
        Err(_) => Err(ErrorCaptura::ErrorCapturador),
    };

    _handlesniffer.join();
    return informe;
}

pub fn ejecutar_escenario_web_rafaga(
    interfaz: Option<String>,
) -> Result<InformeSesion, ErrorCaptura> {
    let (tx, rx) = mpsc::channel();

    let _handlesniffer = std::thread::spawn(move || {
        let _informe = match capturar_paquetes(interfaz, 80) {
            Ok(i) => tx.send(i),
            Err(_) => panic!("Error al capturar paquetes."),
        };
    });

    std::thread::sleep(Duration::from_millis(500));
    if let Err(e) = enviar_datos_http_rafaga() {
        eprintln!("Error al enviar peticion HTTP: {}", e);
    }

    let informe = match rx.recv() {
        Ok(i) => Ok(i),
        Err(_) => Err(ErrorCaptura::ErrorCapturador),
    };

    _handlesniffer.join();
    return informe;
}

pub fn enviar_datos(
    otra_pc: Extremo,
    interfaz: Option<String>,
) -> Result<InformeSesion, ErrorCaptura> {
    let (tx, rx) = mpsc::channel();
    let _handlesniffer = std::thread::spawn(move || {
        let _informe = match capturar_paquetes(interfaz, 80) {
            Ok(i) => tx.send(i),
            Err(_) => panic!("Error al capturar paquetes."),
        };
    });

    std::thread::sleep(Duration::from_millis(500));
    if let Err(_) = ejecutar_localmente(otra_pc, false) {
        return Err(ErrorCaptura::Error);
    }

    match rx.recv() {
        Ok(i) => return Ok(i),
        Err(_) => return Err(ErrorCaptura::ErrorCapturador),
    };
    _handlesniffer.join();
}

pub fn escenario_local() -> Result<InformeSesion, ErrorCaptura> {
    let (tx, rx) = mpsc::channel();
    let _handlesniffer = std::thread::spawn(move || {
        let _informe = match capturar_paquetes(Some("lo".to_string()), 80) {
            Ok(i) => tx.send(i),
            Err(_) => panic!("Error al capturar paquetes."),
        };
    });

    std::thread::sleep(Duration::from_millis(500));
    let loopback = Extremo {
        ip: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        puerto: 9400,
    };
    if let Err(_) = ejecutar_localmente(loopback, true) {
        return Err(ErrorCaptura::Error);
    }

    match rx.recv() {
        Ok(i) => return Ok(i),
        Err(_) => return Err(ErrorCaptura::ErrorCapturador),
    };
    _handlesniffer.join();
}

pub fn renderizar_diapositivas(diapositivas: Vec<PasoTCP>) -> std::io::Result<()> {
    ejecutar_presentacion(diapositivas)?;
    Ok(())
}
