use std::net::IpAddr;

// Modelo de datos

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extremo {
    pub ip: IpAddr,
    pub puerto: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FlagsTCP {
    pub syn: bool,
    pub ack: bool,
    pub fin: bool,
    pub rst: bool,
    pub psh: bool,
    pub urg: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpcionesTCP {
    Mss(u16),
    TamañoVentana(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CabeceraTCP {
    pub origen: Extremo,
    pub destino: Extremo,

    pub seq_abs: u32,
    pub seq_rel: u32,

    pub ack_abs: Option<u32>,
    pub ack_rel: Option<u32>,

    pub longitud_cabecera_bytes: u8,
    pub flags_activas: FlagsTCP,
    pub ventana_recepcion: u16,
    pub checksum: u16,
    pub puntero_urgente: Option<u16>,

    pub opciones: Option<Vec<OpcionesTCP>>,
    pub longitud_datos: usize,
}

// Implementaciones

impl Extremo {
    pub fn new(ip: IpAddr, puerto: u16) -> Self {
        return Extremo { ip, puerto };
    }
}

impl std::fmt::Display for Extremo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Ip: {} Puerto:{}", self.ip, self.puerto);
    }
}

impl std::fmt::Display for FlagsTCP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.syn, self.ack, self.fin, self.psh, self.rst, self.urg) {
            (true, true, false, false, false, false) => return write!(f, "SYN-ACK"),
            (false, true, true, false, false, false) => return write!(f, "ACK-FIN"),
            (false, true, false, true, false, false) => return write!(f, "ACK-PSH"),
            (false, true, false, false, true, false) => return write!(f, "ACK-RST"),
            (true, false, false, false, false, false) => return write!(f, "SYN"),
            (false, true, false, false, false, false) => return write!(f, "ACK"),
            (false, false, true, false, false, false) => return write!(f, "FIN"),
            (false, false, false, true, false, false) => return write!(f, "PSH"),
            (false, false, false, false, true, false) => return write!(f, "RST"),
            (false, false, false, false, false, true) => return write!(f, "URG"),
            _ => return write!(f, "NO SE DETECTARON FLAGS VALIDAS!!"),
        }
    }
}

impl std::fmt::Display for CabeceraTCP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ack_str = match (self.ack_rel, self.ack_abs) {
            (Some(rel), Some(abs)) => format!("{} (Abs: {})", rel, abs),
            _ => "-".to_string(),
        };

        let opciones_str = match &self.opciones {
            Some(ops) if !ops.is_empty() => ops
                .iter()
                .map(|op| match op {
                    OpcionesTCP::Mss(m) => format!("MSS: {}", m),
                    OpcionesTCP::TamañoVentana(w) => format!("WScale: {}", w),
                })
                .collect::<Vec<_>>()
                .join(", "),
            _ => "Ninguna".to_string(),
        };

        writeln!(f, "┌────────────────────────────────────────────────────────────────────────┐")?;
        writeln!(f, "│ Flujo: {} ──> {}", self.origen, self.destino)?;
        writeln!(f, "├────────────────────────────────────────────────────────────────────────┤")?;
        writeln!(f, "│ Secuencia (Seq):      {} (Abs: {})", self.seq_rel, self.seq_abs)?;
        writeln!(f, "│ Reconocimiento (Ack): {}", ack_str)?;
        writeln!(f, "│ Flags de control:     [ {} ]", self.flags_activas)?;
        writeln!(f, "│ Ventana (rwnd):       {} bytes", self.ventana_recepcion)?;
        writeln!(f, "│ Longitud payload:     {} bytes (Cabecera: {} bytes)", self.longitud_datos, self.longitud_cabecera_bytes)?;
        writeln!(f, "│ Opciones TCP:         {}", opciones_str)?;
        write!(f,   "└────────────────────────────────────────────────────────────────────────┘")
    }
}
