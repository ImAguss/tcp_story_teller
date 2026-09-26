use pcapture::{Capture, Device};

use crate::Captura::errores::ErrorCaptura;
use crate::Dominio::estados::InformeSesion;

fn obtener_interfaz(interfaz: Option<&str>) -> Option<Device> {
    let Ok(interfaces) = Device::list() else {
        return None;
    };

    if let Some(i) = interfaz {
        return interfaces.into_iter().find(|x| x.0.name == i);
    }

    for i in interfaces {
        if !i.0.is_loopback() && i.0.is_running() && !i.0.ips.is_empty() {
            return Some(i);
        }
    }

    None
}

pub fn capturar_paquetes(
    interfaz: Option<&str>,
    puerto: u16,
) -> Result<InformeSesion, ErrorCaptura> {
    let interfaz = obtener_interfaz(interfaz).expect("Error al obtener interfaz activa.");
    let mut capturador = Capture::new(&interfaz.0.name).unwrap();

    let filtro = format!("tcp port {}", puerto);
    capturador.set_filter(filtro.as_str()).unwrap();
    capturador.set_timeout(50f32);

    let mut informe = InformeSesion::new();
    let mut indice: usize = 0;

    let mut tiempo_ultimo_paquete = std::time::Instant::now();
    let tiempo_inactividad = std::time::Duration::from_secs(6);

    loop {
        if let Ok(paquete) = capturador.next_as_vec() {
            if let Ok(()) = informe.procesar_paquete(paquete) {
                if informe.conexion_terminada() {
                    return Ok(informe);
                }
                tiempo_ultimo_paquete = std::time::Instant::now();
            }
        }
        if !informe.pasos.is_empty() && tiempo_ultimo_paquete.elapsed() >= tiempo_inactividad {
            return Ok(informe);
        }

        if informe.pasos.is_empty() && tiempo_ultimo_paquete.elapsed() >= tiempo_inactividad {
            return Err(ErrorCaptura::SalidaInesperada);
        }
    }
}
