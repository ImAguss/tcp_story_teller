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

    pub longitud_cabecera_bytes: usize,
    pub flags_activas: FlagsTCP,
    pub ventana_recepcion: u16,
    pub checksum: u16,
    pub puntero_urgente: u16,

    pub payload: Option<Vec<u8>>,
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

        writeln!(
            f,
            "┌────────────────────────────────────────────────────────────────────────┐"
        )?;
        writeln!(f, "│ Flujo: {} ──> {}", self.origen, self.destino)?;
        writeln!(
            f,
            "├────────────────────────────────────────────────────────────────────────┤"
        )?;
        writeln!(
            f,
            "│ Secuencia (Seq):      {} (Abs: {})",
            self.seq_rel, self.seq_abs
        )?;
        writeln!(f, "│ Reconocimiento (Ack): {}", ack_str)?;
        writeln!(f, "│ Flags de control:     [ {} ]", self.flags_activas)?;
        writeln!(
            f,
            "│ Ventana (rwnd):       {} bytes",
            self.ventana_recepcion
        )?;
        writeln!(
            f,
            "│ Longitud payload:     {} bytes (Cabecera: {} bytes)",
            self.longitud_datos, self.longitud_cabecera_bytes
        )?;
        writeln!(f, "│ Opciones TCP:         {}", opciones_str)?;

        if let Some(ref bytes) = self.payload {
            if !bytes.is_empty() {
                writeln!(
                    f,
                    "├────────────────────────────────────────────────────────────────────────┤"
                )?;
                if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(bytes) {
                    writeln!(f, "│ Carga útil (JSON - {} bytes):", bytes.len())?;
                    if let Ok(pretty) = serde_json::to_string_pretty(&json_val) {
                        let lineas: Vec<&str> = pretty.lines().collect();
                        let max_lineas = 8;
                        for linea in lineas.iter().take(max_lineas) {
                            let linea_formateada = if linea.chars().count() > 64 {
                                format!("{}...", linea.chars().take(61).collect::<String>())
                            } else {
                                linea.to_string()
                            };
                            writeln!(f, "│   {}", linea_formateada)?;
                        }
                        if lineas.len() > max_lineas {
                            writeln!(
                                f,
                                "│   [... {} líneas más | {} bytes totales ...]",
                                lineas.len() - max_lineas,
                                bytes.len()
                            )?;
                        }
                    }
                } else if let Ok(texto) = std::str::from_utf8(bytes) {
                    let texto_limpio = texto.trim();
                    if texto_limpio.chars().count() <= 64 && !texto_limpio.contains('\n') {
                        writeln!(f, "│ Carga útil (Texto):   {}", texto_limpio)?;
                    } else {
                        writeln!(f, "│ Carga útil (Texto - {} bytes):", bytes.len())?;
                        let lineas: Vec<&str> = texto_limpio.lines().collect();
                        let max_lineas = 6;
                        for linea in lineas.iter().take(max_lineas) {
                            let linea_formateada = if linea.chars().count() > 64 {
                                format!("{}...", linea.chars().take(61).collect::<String>())
                            } else {
                                linea.to_string()
                            };
                            writeln!(f, "│   {}", linea_formateada)?;
                        }
                        if lineas.len() > max_lineas {
                            writeln!(
                                f,
                                "│   [... {} líneas más | {} bytes totales ...]",
                                lineas.len() - max_lineas,
                                bytes.len()
                            )?;
                        }
                    }
                } else {
                    let hex_preview = bytes
                        .iter()
                        .take(16)
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    writeln!(f, "│ Carga útil (Hex - {} bytes):", bytes.len())?;
                    writeln!(f, "│   {} ...", hex_preview)?;
                }
            }
        }

        write!(
            f,
            "└────────────────────────────────────────────────────────────────────────┘"
        )
    }
}
