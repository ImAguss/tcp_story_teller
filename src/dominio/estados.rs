use std::net::IpAddr;

use etherparse::{NetSlice, TransportSlice};

#[allow(dead_code)]
use crate::dominio::datagrama_tcp::{Extremo, FlagsTCP};
use crate::{
    captura::errores::ErrorProcesamiento,
    dominio::visual::{Direccion, EtapaConexion, PasoCierre, PasoHandshake, PasoTCP},
};

pub struct InformeSesion {
    pub isn_cliente: Option<u32>,
    pub isn_servidor: Option<u32>,

    pub extremo_cliente: Option<Extremo>,
    pub extremo_receptor: Option<Extremo>,

    pub ultimo_ack_visto: Option<u32>,
    pub contador_ack_dupl: u8,
    pub ventana_contexto_actual: u16,

    pub tiempo_inicio: Option<std::time::Instant>,
    pub pasos: Vec<PasoTCP>,
}

impl InformeSesion {
    pub fn new() -> Self {
        return InformeSesion {
            isn_cliente: None,
            isn_servidor: None,
            extremo_cliente: None,
            extremo_receptor: None,
            ultimo_ack_visto: None,
            contador_ack_dupl: 0,
            ventana_contexto_actual: 0,
            tiempo_inicio: None,
            pasos: Vec::new(),
        };
    }

    pub fn procesar_paquete(&mut self, paquete: Vec<u8>) -> Result<(), ErrorProcesamiento> {
        let Ok(paquete) = etherparse::SlicedPacket::from_ethernet(&paquete) else {
            return Err(ErrorProcesamiento);
        };

        let (ip_origen, ip_destino) = match paquete.net {
            Some(NetSlice::Ipv4(ipv4)) => (
                IpAddr::V4(ipv4.header().source_addr()),
                IpAddr::V4(ipv4.header().destination_addr()),
            ),
            Some(NetSlice::Ipv6(ipv6)) => (
                IpAddr::V6(ipv6.header().source_addr()),
                IpAddr::V6(ipv6.header().destination_addr()),
            ),
            _ => return Err(ErrorProcesamiento),
        };

        let tcp = match paquete.transport {
            Some(TransportSlice::Tcp(tcp)) => tcp,
            _ => return Err(ErrorProcesamiento),
        };

        let (puerto_origen, puerto_destino) = (tcp.source_port(), tcp.destination_port());
        let origen = Extremo {
            ip: ip_origen,
            puerto: puerto_origen,
        };
        let destino = Extremo {
            ip: ip_destino,
            puerto: puerto_destino,
        };

        let flags = FlagsTCP {
            syn: tcp.syn(),
            ack: tcp.ack(),
            fin: tcp.fin(),
            rst: tcp.rst(),
            psh: tcp.psh(),
            urg: tcp.urg(),
        };

        let seq_abs = tcp.sequence_number();
        let ack_abs = if tcp.ack() {
            Some(tcp.acknowledgment_number())
        } else {
            None
        };
        let len_datos = tcp.payload().len();
        let ventana = tcp.window_size();
        let checksum = tcp.checksum();
        let puntero_urgente = tcp.urgent_pointer();
        let longitud_cabecera_bytes = tcp.header_len();

        if let Some(ultimo) = self.pasos.last() {
            if ultimo.cabecera.origen == origen
                && ultimo.cabecera.destino == destino
                && ultimo.cabecera.seq_abs == seq_abs
                && ultimo.cabecera.ack_abs == ack_abs
                && ultimo.cabecera.flags_activas == flags
                && ultimo.cabecera.longitud_datos == len_datos
            {
                return Ok(());
            }
        }

        if self.pasos.is_empty() {
            self.isn_cliente = Some(seq_abs);
            self.extremo_cliente = Some(origen.clone());
            self.extremo_receptor = Some(destino.clone());
            self.tiempo_inicio = Some(std::time::Instant::now());
        }

        let direccion = self.obtener_direccion(&origen);

        if self.isn_servidor.is_none() && direccion == Direccion::ServidorCliente && flags.syn {
            self.isn_servidor = Some(seq_abs);
        }

        let tiempo_relativo = self
            .tiempo_inicio
            .map(|t0| std::time::Instant::now().duration_since(t0))
            .unwrap_or(std::time::Duration::ZERO);

        let (seq_rel, ack_rel) = match direccion {
            Direccion::ClienteServidor => {
                let isn_emisor = self.isn_cliente.unwrap_or(seq_abs);
                calcular_relativos(seq_abs, ack_abs, isn_emisor, self.isn_servidor)
            }
            Direccion::ServidorCliente => {
                let isn_emisor = self.isn_servidor.unwrap_or(seq_abs);
                calcular_relativos(seq_abs, ack_abs, isn_emisor, self.isn_cliente)
            }
        };

        let etapa = if self.pasos.is_empty() {
            EtapaConexion::Handshake(PasoHandshake::Syn)
        } else if self.pasos.len() == 1 && flags.syn && flags.ack {
            EtapaConexion::Handshake(PasoHandshake::SynAck)
        } else if self.pasos.len() == 2 && flags.ack && len_datos == 0 {
            EtapaConexion::Handshake(PasoHandshake::Ack)
        } else {
            deducir_etapa(
                &flags,
                len_datos,
                ventana,
                Some(seq_rel),
                None,
                ack_rel,
                self.ultimo_ack_visto,
                self.contador_ack_dupl,
            )
            .unwrap_or(EtapaConexion::TransferenciaDatos {
                bytes: len_datos,
                es_push: flags.psh,
            })
        };

        if let Some(ack) = ack_rel {
            if self.ultimo_ack_visto == Some(ack) && len_datos == 0 {
                self.contador_ack_dupl += 1;
            } else if self.ultimo_ack_visto.map_or(true, |ul| ack > ul) {
                self.contador_ack_dupl = 0;
                self.ultimo_ack_visto = Some(ack);
            }
        }

        let payload = if tcp.payload().len() > 0 {
            Some(tcp.payload().to_vec())
        } else {
            None
        };
        let indice = self.pasos.len() + 1;

        let paso = PasoTCP::new(
            indice,
            tiempo_relativo,
            origen,
            destino,
            direccion,
            etapa,
            seq_abs,
            seq_rel,
            ack_abs,
            ack_rel,
            longitud_cabecera_bytes,
            flags,
            ventana,
            checksum,
            puntero_urgente,
            payload,
            None,
            len_datos,
        );
        self.pasos.push(paso);

        Ok(())
    }

    fn obtener_direccion(&self, origen: &Extremo) -> Direccion {
        let direccion = if self.extremo_cliente.as_ref() == Some(origen) {
            Direccion::ClienteServidor
        } else {
            Direccion::ServidorCliente
        };

        return direccion;
    }

    pub fn conexion_terminada(&self) -> bool {
        let Some(ultimo) = self.pasos.last() else {
            return false;
        };
        if ultimo.cabecera.flags_activas.rst {
            return true;
        }

        let _paquetes_fin: Vec<&PasoTCP> = self
            .pasos
            .iter()
            .filter(|x| x.cabecera.flags_activas.fin)
            .collect();

        let fin_cliente = self
            .pasos
            .iter()
            .any(|p| p.direccion == Direccion::ClienteServidor && p.cabecera.flags_activas.fin);

        let fin_servidor = self
            .pasos
            .iter()
            .any(|p| p.direccion == Direccion::ServidorCliente && p.cabecera.flags_activas.fin);

        if (fin_cliente && fin_servidor)
            && (ultimo.cabecera.flags_activas.ack && !ultimo.cabecera.flags_activas.fin)
        {
            return true;
        }

        return false;
    }
}

pub fn calcular_relativos(
    seq_abs: u32,
    ack_abs: Option<u32>,
    isn_emisor: u32,
    isn_receptor: Option<u32>,
) -> (u32, Option<u32>) {
    let seq_rel = seq_abs.wrapping_sub(isn_emisor);
    let ack_rel = ack_abs
        .zip(isn_receptor)
        .map(|(ack, isn)| ack.wrapping_sub(isn));
    (seq_rel, ack_rel)
}

pub fn deducir_etapa(
    flags_activas: &FlagsTCP,
    len_datos: usize,
    ventana: u16,
    seq: Option<u32>,
    seq_esperado: Option<u32>,
    ack_actual: Option<u32>,
    ultimo_ack: Option<u32>,
    contador_dups: u8,
) -> Option<EtapaConexion> {
    if detectar_rst(flags_activas) {
        return Some(EtapaConexion::Reset);
    }

    if detectar_cierre_tcp(flags_activas) {
        if flags_activas.ack {
            return Some(EtapaConexion::Cierre(PasoCierre::AckFin));
        } else {
            return Some(EtapaConexion::Cierre(PasoCierre::Fin));
        }
    }

    if detectar_ventana_cero(ventana) {
        return Some(EtapaConexion::VentanaCero);
    }

    if detectar_sonda_ventana(flags_activas, len_datos) {
        return Some(EtapaConexion::SondaVentana);
    }

    if flags_activas.syn && flags_activas.ack {
        return Some(EtapaConexion::Handshake(PasoHandshake::SynAck));
    }

    if flags_activas.syn {
        return Some(EtapaConexion::Handshake(PasoHandshake::Syn));
    }

    if detectar_retransmision_rapida(flags_activas, contador_dups, len_datos, seq, seq_esperado) {
        let seq = seq.unwrap_or(0);
        return Some(EtapaConexion::RetransmisionRapida {
            seq_retransmitido: seq,
        });
    }

    if detectar_transferencia_datos(flags_activas, len_datos) {
        return Some(EtapaConexion::TransferenciaDatos {
            bytes: len_datos,
            es_push: flags_activas.psh,
        });
    }

    if flags_activas.ack && len_datos == 0 && ultimo_ack.is_none() {
        return Some(EtapaConexion::Handshake(PasoHandshake::Ack));
    }

    if detectar_ack_duplicado(
        flags_activas,
        contador_dups,
        len_datos,
        ack_actual,
        ultimo_ack,
    ) {
        let esperando = seq_esperado.unwrap_or(0);
        return Some(EtapaConexion::AckDuplicado {
            contador: contador_dups,
            esperando_seq: esperando,
        });
    }

    if detectar_ack_acumulativo(flags_activas, len_datos, ultimo_ack, ack_actual) {
        return Some(EtapaConexion::AckAcumulativo);
    }

    if flags_activas.ack && len_datos == 0 {
        return Some(EtapaConexion::Cierre(PasoCierre::Ack));
    }

    None
}

fn detectar_transferencia_datos(flags: &FlagsTCP, len_datos: usize) -> bool {
    if flags.ack && len_datos > 0 {
        return true;
    }

    false
}

fn detectar_ack_acumulativo(
    flags: &FlagsTCP,
    len_datos: usize,
    ultimo_ack: Option<u32>,
    ack_actual: Option<u32>,
) -> bool {
    let ul_ack = match ultimo_ack {
        Some(x) => x,
        None => return false,
    };

    let act_ack = match ack_actual {
        Some(x) => x,
        None => return false,
    };

    if (flags.ack && len_datos == 0) && (act_ack > ul_ack) {
        return true;
    } else {
        return false;
    }
}

fn detectar_ack_duplicado(
    flags: &FlagsTCP,
    contador_ack_dup: u8,
    len_datos: usize,
    ack_actual: Option<u32>,
    ultimo_ack: Option<u32>,
) -> bool {
    let ack_act = match ack_actual {
        Some(x) => x,
        None => return false,
    };
    let ack_ul = match ultimo_ack {
        Some(x) => x,
        None => return false,
    };
    let es_duplicado = ack_act == ack_ul && contador_ack_dup >= 1;

    if flags.ack && es_duplicado && len_datos == 0 {
        return true;
    }
    false
}

fn detectar_retransmision_rapida(
    flags: &FlagsTCP,
    contador_dups: u8,
    len_datos: usize,
    seq: Option<u32>,
    seq_esperado: Option<u32>,
) -> bool {
    let seq_actual = match seq {
        Some(x) => x,
        None => return false,
    };

    let seq_esperado = match seq_esperado {
        Some(x) => x,
        None => return false,
    };

    if flags.ack && len_datos > 0 && (seq_actual == seq_esperado) && contador_dups == 3 {
        return true;
    }

    false
}

fn detectar_sonda_ventana(flags: &FlagsTCP, len_datos: usize) -> bool {
    return flags.ack && len_datos == 1;
}

fn detectar_ventana_cero(ventana: u16) -> bool {
    return ventana == 0;
}

fn detectar_cierre_tcp(flags: &FlagsTCP) -> bool {
    return flags.fin;
}

fn detectar_rst(flags: &FlagsTCP) -> bool {
    return flags.rst;
}
