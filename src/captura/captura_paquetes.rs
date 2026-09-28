use pcapture::{Capture, Device};

use crate::captura::errores::ErrorCaptura;
use crate::dominio::estados::InformeSesion;

fn obtener_interfaz(interfaz: Option<String>) -> Option<Device> {
    let Ok(interfaces) = Device::list() else {
        return None;
    };

    if let Some(i) = interfaz {
        return interfaces.into_iter().find(|x| x.0.name == i.to_string());
    }

    for i in interfaces {
        if !i.0.is_loopback() && i.0.is_running() && !i.0.ips.is_empty() {
            return Some(i);
        }
    }

    None
}

pub fn capturar_paquetes(
    interfaz: Option<String>,
    puerto: u16,
) -> Result<InformeSesion, ErrorCaptura> {
    let interfaz = obtener_interfaz(interfaz).ok_or(ErrorCaptura::ErrorInterfaz)?;
    let mut capturador =
        Capture::new(&interfaz.0.name).map_err(|_| ErrorCaptura::ErrorCapturador)?;

    let filtro = format!("tcp port {}", puerto);
    capturador
        .set_filter(filtro.as_str())
        .map_err(|_| ErrorCaptura::ErrorCapturador)?;
    capturador.set_timeout(50f32);

    let mut informe = InformeSesion::new();

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
