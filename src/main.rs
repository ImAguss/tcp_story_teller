mod Captura;
mod Dominio;
mod Escenarios;
mod Presentacion;
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::mpsc,
    time::Duration,
};

use crate::{
    Captura::{
        captura_paquetes::{self, capturar_paquetes},
        errores::ErrorCaptura::ErrorCapturador,
    },
    Dominio::datagrama_tcp::Extremo,
    Escenarios::local_simple::ejecutar,
};

fn main() {
    let (tx, rx) = mpsc::channel();
    let handle_captura = std::thread::spawn(move || {
        let informe = match capturar_paquetes(Some("lo"), 8080) {
            Ok(i) => tx.send(i),
            _ => return,
        };
    });

    if let Err(_) = ejecutar(Extremo {
        ip: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        puerto: 8080,
    }) {
        return;
    }

    let informe = rx.recv_timeout(Duration::from_secs(10));

    if let Ok(informe) = informe {
        for i in &informe.pasos {
            println!("{}", i.bloque_completo());
        }
    }
}
