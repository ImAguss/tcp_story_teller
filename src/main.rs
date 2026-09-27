#![allow(unused)]

mod captura;
mod dominio;
mod escenarios;
mod presentacion;
use std::{net::Ipv4Addr, sync::mpsc, time::Duration};

use crate::escenarios::peticiones_http::enviar_datos_http;
use crate::escenarios::peticiones_locales::ejecutar;
use crate::presentacion::presentacion::ejecutar_presentacion;
use crate::{captura::captura_paquetes::capturar_paquetes, dominio::datagrama_tcp::Extremo};

fn main() {
    let (tx, rx) = mpsc::channel();
    let _handle_captura = std::thread::spawn(move || {
        let _informe = match capturar_paquetes(Some("enp4s0"), 80) {
            Ok(i) => tx.send(i),
            _ => panic!("Error al iniciar captura."),
        };
    });

    if let Err(e) = enviar_datos_http() {
        eprintln!("Error aca: {}", e);
    };

    match rx.recv() {
        Ok(informe) => {
            if let Err(e) = ejecutar_presentacion(informe.pasos) {
                eprintln!("Error al ejecutar presentacion: {}", e);
            }
        }
        Err(_) => eprintln!("Error en Hilo"),
    }
}
