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
    pub opciones: Vec<OpcionesTCP>,
    pub longitud_datos: usize,
}

// Implementaciones

impl std::fmt::Display for Extremo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.ip, self.puerto)
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
            (Some(rel), Some(abs)) => format!("{}          (Absoluto: {})", rel, abs),
            _ => "-".to_string(),
        };

        let opciones_str = if !self.opciones.is_empty() {
            self.opciones
                .iter()
                .map(|op| match op {
                    OpcionesTCP::Mss(m) => format!("MSS: {}", m),
                    OpcionesTCP::TamañoVentana(w) => format!("WScale: {}", w),
                })
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            "Ninguna".to_string()
        };

        let pad_line = |content: &str| -> String {
            let char_count = content.chars().count();
            if char_count > 74 {
                let truncada: String = content.chars().take(71).collect();
                format!("│  {}...  │", truncada)
            } else {
                let pad = 74 - char_count;
                format!("│  {}{:pad$}  │", content, "", pad = pad)
            }
        };

        let empty_line = format!("│{:78}│", "");
        let divider = format!("├{}┤", "─".repeat(78));

        writeln!(f, "┌── [ INSPECCIÓN DE CABECERA TCP ] {}┐", "─".repeat(44))?;
        writeln!(f, "{}", empty_line)?;
        writeln!(f, "{}", pad_line(&format!("Flujo:       {}  ────────────►  {}", self.origen, self.destino)))?;
        writeln!(f, "{}", empty_line)?;
        writeln!(f, "{}", divider)?;
        writeln!(f, "{}", pad_line(&format!("Secuencia (Seq):         {}          (Absoluto: {})", self.seq_rel, self.seq_abs)))?;
        writeln!(f, "{}", pad_line(&format!("Reconocimiento (Ack):    {}", ack_str)))?;
        writeln!(f, "{}", pad_line(&format!("Flags de Control:        [ {} ]", self.flags_activas)))?;
        writeln!(f, "{}", divider)?;
        writeln!(f, "{}", pad_line(&format!("Ventana de Recepción:    {} bytes", self.ventana_recepcion)))?;
        let opciones_bytes = self.longitud_cabecera_bytes.saturating_sub(20);
        writeln!(f, "{}", pad_line(&format!("Tamaño Cabecera:         {} bytes (Base: 20B + Opciones: {}B)", self.longitud_cabecera_bytes, opciones_bytes)))?;
        writeln!(f, "{}", pad_line(&format!("Opciones TCP:            {}", opciones_str)))?;

        if let Some(ref bytes) = self.payload {
            if !bytes.is_empty() {
                writeln!(f, "{}", empty_line)?;
                writeln!(f, "{}", divider)?;
                if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(bytes) {
                    writeln!(f, "{}", pad_line(&format!("Carga Útil (JSON - {} bytes):", bytes.len())))?;
                    if let Ok(pretty) = serde_json::to_string_pretty(&json_val) {
                        let lineas: Vec<&str> = pretty.lines().collect();
                        let max_lineas = 8;
                        for linea in lineas.iter().take(max_lineas) {
                            let linea_formateada = if linea.chars().count() > 68 {
                                format!("{}...", linea.chars().take(65).collect::<String>())
                            } else {
                                linea.to_string()
                            };
                            writeln!(f, "{}", pad_line(&format!("  {}", linea_formateada)))?;
                        }
                        if lineas.len() > max_lineas {
                            writeln!(
                                f,
                                "{}",
                                pad_line(&format!(
                                    "  [... {} líneas más | {} bytes totales ...]",
                                    lineas.len() - max_lineas,
                                    bytes.len()
                                ))
                            )?;
                        }
                    }
                } else if let Ok(texto) = std::str::from_utf8(bytes) {
                    let texto_limpio = texto.trim();
                    if texto_limpio.chars().count() <= 68 && !texto_limpio.contains('\n') {
                        writeln!(f, "{}", pad_line(&format!("Carga Útil (Texto):      {}", texto_limpio)))?;
                    } else {
                        writeln!(f, "{}", pad_line(&format!("Carga Útil (Texto - {} bytes):", bytes.len())))?;
                        let lineas: Vec<&str> = texto_limpio.lines().collect();
                        let max_lineas = 6;
                        for linea in lineas.iter().take(max_lineas) {
                            let linea_formateada = if linea.chars().count() > 68 {
                                format!("{}...", linea.chars().take(65).collect::<String>())
                            } else {
                                linea.to_string()
                            };
                            writeln!(f, "{}", pad_line(&format!("  {}", linea_formateada)))?;
                        }
                        if lineas.len() > max_lineas {
                            writeln!(
                                f,
                                "{}",
                                pad_line(&format!(
                                    "  [... {} líneas más | {} bytes totales ...]",
                                    lineas.len() - max_lineas,
                                    bytes.len()
                                ))
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
                    writeln!(f, "{}", pad_line(&format!("Carga Útil (Hex - {} bytes):", bytes.len())))?;
                    writeln!(f, "{}", pad_line(&format!("  {} ...", hex_preview)))?;
                }
            } else {
                writeln!(f, "{}", empty_line)?;
                writeln!(f, "{}", pad_line("Carga Útil (Payload):    0 bytes (Tráfico de control puro)"))?;
            }
        } else {
            writeln!(f, "{}", empty_line)?;
            writeln!(f, "{}", pad_line("Carga Útil (Payload):    0 bytes (Tráfico de control puro)"))?;
        }

        writeln!(f, "{}", empty_line)?;
        write!(f, "└{}┘", "─".repeat(78))
    }
}
