use crate::Dominio::datagrama_tcp::{Extremo, FlagsTCP};
use crate::Dominio::visual::{EtapaConexion, PasoTCP};

pub struct InformeSesion {
    pub isn_cliente: Option<u32>,
    pub isn_servidor: Option<u32>,

    pub extremo_cliente: Option<Extremo>,
    pub extremo_receptor: Option<Extremo>,

    pub ultimo_ack_visto: Option<u32>,
    pub contador_ack_dupl: u8,
    pub ventana_contexto_actual: u16,

    pub tiempo_inicio: Option<std::time::Instant>,
    pub passo: Vec<PasoTCP>,
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
    ultimo_ack: Option<u32>,
    contador_dups: u8,
    es_primer_paquete: bool,
) -> EtapaConexion {
}
