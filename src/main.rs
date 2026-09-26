mod Captura;
mod Dominio;
mod Escenarios;
mod Presentacion;
use std::{net::Ipv4Addr, sync::mpsc, time::Duration};

use crate::Escenarios::local_simple::ejecutar;
use crate::Presentacion::presentacion::ejecutar_presentacion;
use crate::{Captura::captura_paquetes::capturar_paquetes, Dominio::datagrama_tcp::Extremo};

fn main() {
    let (tx, rx) = mpsc::channel();
    let _handle_captura = std::thread::spawn(move || {
        let _informe = match capturar_paquetes(Some("lo"), 8080) {
            Ok(i) => tx.send(i),
            _ => panic!("Error al iniciar captura."),
        };
    });

    if let Err(_) = ejecutar(Extremo {
        ip: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        puerto: 8080,
    }) {
        return;
    }

    match rx.recv() {
        Ok(informe) => {
            if let Err(e) = ejecutar_presentacion(informe.pasos) {
                eprintln!("Error al ejecutar presentacion: {}", e);
            }
        }
        Err(_) => eprintln!("Error en Hilo"),
    }
}
