#![allow(unused)]

mod captura;
mod dominio;
mod escenarios;
mod presentacion;
use std::{net::Ipv4Addr, sync::mpsc, time::Duration};

use crate::escenarios::local_simple::ejecutar;
use crate::presentacion::presentacion::ejecutar_presentacion;
use crate::{captura::captura_paquetes::capturar_paquetes, dominio::datagrama_tcp::Extremo};

fn main() {
    let (tx, rx) = mpsc::channel();
    let _handle_captura = std::thread::spawn(move || {
        let _informe = match capturar_paquetes(Some("lo"), 9421) {
            Ok(i) => tx.send(i),
            _ => panic!("Error al iniciar captura."),
        };
    });

    if let Err(_) = ejecutar(Extremo {
        ip: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        puerto: 9421,
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
