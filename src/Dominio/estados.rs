#[allow(dead_code)]
use crate::Dominio::datagrama_tcp::{Extremo, FlagsTCP};
use crate::Dominio::visual::{EtapaConexion, PasoCierre, PasoHandshake, PasoTCP};

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
