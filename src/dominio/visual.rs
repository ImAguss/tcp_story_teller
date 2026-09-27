use crate::dominio::datagrama_tcp::{CabeceraTCP, Extremo, FlagsTCP, OpcionesTCP};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasoHandshake {
    Syn,
    SynAck,
    Ack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasoCierre {
    Ack,
    Fin,
    AckFin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EtapaConexion {
    Handshake(PasoHandshake),
    TransferenciaDatos { bytes: usize, es_push: bool },
    AckAcumulativo,
    AckDuplicado { contador: u8, esperando_seq: u32 },
    RetransmisionRapida { seq_retransmitido: u32 },
    VentanaCero,
    SondaVentana,
    Cierre(PasoCierre),
    Reset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasoTCP {
    pub indice: usize,
    pub tiempo_relativo: std::time::Duration,
    pub cabecera: CabeceraTCP,
    pub direccion: Direccion,
    pub etapa: EtapaConexion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direccion {
    ClienteServidor,
    ServidorCliente,
}

impl EtapaConexion {
    pub fn titulo(&self) -> &'static str {
        match self {
            EtapaConexion::Handshake(PasoHandshake::Syn) => {
                "Establecimiento (1/3): Solicitud de Inicio de Conexión [SYN]"
            }
            EtapaConexion::Handshake(PasoHandshake::SynAck) => {
                "Establecimiento (2/3): Acuse de Recibo y Sincronización Servidor [SYN-ACK]"
            }
            EtapaConexion::Handshake(PasoHandshake::Ack) => {
                "Establecimiento (3/3): Confirmación Final y Conexión Establecida [ACK]"
            }
            EtapaConexion::TransferenciaDatos { .. } => {
                "Transferencia de Datos: Transmisión de Datos de Aplicación"
            }
            EtapaConexion::AckAcumulativo => {
                "Control de Flujo: Acuse de Recibo Acumulativo de Datos [ACK]"
            }
            EtapaConexion::AckDuplicado { .. } => {
                "Alerta de Pérdida o Desorden: Acuse de Recibo Duplicado [ACK Duplicado]"
            }
            EtapaConexion::RetransmisionRapida { .. } => {
                "Mecanismo de Recuperación: Retransmisión Rápida"
            }
            EtapaConexion::VentanaCero => {
                "Control de Flujo Crítico: Buffer Receptor Lleno [Ventana Cero]"
            }
            EtapaConexion::SondaVentana => "Prevención de Bloqueo: Segmento Sonda de Ventana",
            EtapaConexion::Cierre(PasoCierre::Fin) => {
                "Cierre Ordenado: Petición de Fin de Transmisión [FIN]"
            }
            EtapaConexion::Cierre(PasoCierre::Ack) => {
                "Cierre Ordenado: Confirmación de Cierre Unidireccional [ACK]"
            }
            EtapaConexion::Cierre(PasoCierre::AckFin) => {
                "Cierre Ordenado: Confirmación y Solicitud Simultánea [FIN-ACK]"
            }
            EtapaConexion::Reset => "Aborto de Conexión: Rechazo o Reinicio Inmediato [RST]",
        }
    }

    pub fn caso_especial(&self) -> bool {
        match self {
            EtapaConexion::RetransmisionRapida { .. } => true,
            EtapaConexion::AckDuplicado { .. } => true,
            EtapaConexion::VentanaCero => true,
            _ => false,
        }
    }
}

impl PasoTCP {
    fn explicacion(&self) -> String {
        match &self.etapa {
            EtapaConexion::Handshake(PasoHandshake::Syn) => {
                format!(
                    "El cliente inicia la conversación eligiendo un número de secuencia inicial aleatorio (ISN: {}, relativo: {}). \
                    Solicita sincronización para abrir el canal y negociar parámetros. Aún no se transmiten datos de aplicación.",
                    self.cabecera.seq_abs, self.cabecera.seq_rel
                )
            }
            EtapaConexion::Handshake(PasoHandshake::SynAck) => {
                let ack_txt = self
                    .cabecera
                    .ack_rel
                    .map_or("-".to_string(), |a| a.to_string());
                format!(
                    "El servidor acepta la conexión y reserva buffers en memoria. Confirma el SYN del cliente (Ack: {}) \
                    y propone su propio número de secuencia inicial (ISN: {}, relativo: {}).",
                    ack_txt, self.cabecera.seq_abs, self.cabecera.seq_rel
                )
            }
            EtapaConexion::Handshake(PasoHandshake::Ack) => {
                let ack_txt = self
                    .cabecera
                    .ack_rel
                    .map_or("-".to_string(), |a| a.to_string());
                format!(
                    "El cliente confirma el SYN-ACK del servidor (Ack: {}). Concluye el saludo de tres vías: \
                    la conexión pasa a estado ESTABLECIDA y ambos extremos quedan habilitados para transmitir datos.",
                    ack_txt
                )
            }
            EtapaConexion::TransferenciaDatos { bytes, es_push } => {
                let push_txt = if *es_push {
                    " El bit PSH está encendido, indicando al receptor que entregue los datos a la aplicación de inmediato sin esperar a llenar el buffer."
                } else {
                    ""
                };
                let hasta_byte = self.cabecera.seq_rel + (*bytes as u32);
                format!(
                    "Se transmiten {} bytes de datos de aplicación (Secuencia relativa: {} a {}).{}",
                    bytes,
                    self.cabecera.seq_rel,
                    hasta_byte.saturating_sub(1),
                    push_txt
                )
            }
            EtapaConexion::AckAcumulativo => {
                let ack_txt = self
                    .cabecera
                    .ack_rel
                    .map_or("-".to_string(), |a| a.to_string());
                format!(
                    "El receptor confirma la llegada correcta y ordenada de todos los bytes anteriores sin pérdidas ni huecos. \
                    Notifica que espera el byte consecutivo (Ack: {}) y anuncia una ventana libre de {} bytes.",
                    ack_txt, self.cabecera.ventana_recepcion
                )
            }
            EtapaConexion::AckDuplicado {
                contador,
                esperando_seq,
            } => {
                format!(
                    "¡Alerta de posible pérdida o desorden en tránsito! El receptor recibió un segmento con secuencia superior a la esperada. \
                    Reitera su acuse anterior (duplicado #{}) reclamando la llegada del byte faltante (Seq: {}).",
                    contador, esperando_seq
                )
            }
            EtapaConexion::RetransmisionRapida { seq_retransmitido } => {
                format!(
                    "Mecanismo de Retransmisión Rápida: tras recibir 3 ACKs duplicados consecutivos, el emisor deduce que el segmento \
                    con Seq: {} se perdió en la red y lo retransmite de inmediato sin aguardar al vencimiento del temporizador.",
                    seq_retransmitido
                )
            }
            EtapaConexion::VentanaCero => {
                format!(
                    "¡Control de flujo crítico! El receptor anuncia una ventana de recepción rwnd = 0 (buffer de memoria colmado). \
                    Ordena al emisor congelar los envíos para evitar descartar paquetes por saturación."
                )
            }
            EtapaConexion::SondaVentana => {
                format!(
                    "Segmento Sonda de Ventana: ante una ventana en cero, el emisor transmite 1 byte mínimo de datos para forzar \
                    una respuesta del receptor y averiguar si ya liberó espacio en memoria, previniendo un interbloqueo mutuo."
                )
            }
            EtapaConexion::Cierre(PasoCierre::Fin) => {
                format!(
                    "El emisor solicita un cierre ordenado de su sentido de la transmisión [FIN]. Comunica que no tiene más datos \
                    por transmitir y aguarda la confirmación del otro extremo."
                )
            }
            EtapaConexion::Cierre(PasoCierre::Ack) => {
                let ack_txt = self
                    .cabecera
                    .ack_rel
                    .map_or("-".to_string(), |a| a.to_string());
                format!(
                    "Se confirma el acuse de recibo del cierre [ACK: {}]. El canal unidireccional correspondiente queda concluido.",
                    ack_txt
                )
            }
            EtapaConexion::Cierre(PasoCierre::AckFin) => {
                let ack_txt = self
                    .cabecera
                    .ack_rel
                    .map_or("-".to_string(), |a| a.to_string());
                format!(
                    "El extremo confirma el cierre del canal previo [ACK: {}] y al mismo tiempo solicita formalmente el cierre de su propio sentido [FIN].",
                    ack_txt
                )
            }
            EtapaConexion::Reset => {
                format!(
                    "El host envía un flag RST: se aborta o rechaza la conexión de forma inmediata. \
                    Comportamiento típico cuando se intenta conectar a un puerto cerrado sin ningún servicio a la escucha."
                )
            }
        }
    }

    pub fn bloque_completo(&self) -> String {
        let flecha = match self.direccion {
            Direccion::ClienteServidor => "Cliente ──────────> Servidor",
            Direccion::ServidorCliente => "Cliente <────────── Servidor",
        };

        let encabezado = format!(
            "══════════════════════════════════════════════════════════════════════════\n\
             [ Paso #{} ]  (+{:.3}s)   {}\n\
             Etapa: {}\n\
            ══════════════════════════════════════════════════════════════════════════",
            self.indice,
            self.tiempo_relativo.as_secs_f64(),
            flecha,
            self.etapa.titulo()
        );

        format!(
            "{}\n\n{}\n\nExplicación didáctica:\n{}\n",
            encabezado,
            self.cabecera,
            self.explicacion()
        )
    }

    pub fn new(
        indice: usize,
        tiempo_relativo: std::time::Duration,
        origen: Extremo,
        destino: Extremo,
        direccion: Direccion,
        etapa: EtapaConexion,
        seq_abs: u32,
        seq_rel: u32,
        ack_abs: Option<u32>,
        ack_rel: Option<u32>,
        longitud_cabecera_bytes: usize,
        flags_activas: FlagsTCP,
        ventana_recepcion: u16,
        checksum: u16,
        puntero_urgente: u16,
        payload: Option<Vec<u8>>,
        opciones: Vec<OpcionesTCP>,
        longitud_datos: usize,
    ) -> Self {
        return PasoTCP {
            indice,
            tiempo_relativo,
            cabecera: CabeceraTCP {
                origen,
                destino,
                seq_abs,
                seq_rel,
                ack_abs,
                ack_rel,
                longitud_cabecera_bytes,
                flags_activas,
                ventana_recepcion,
                checksum,
                puntero_urgente,
                payload,
                opciones,
                longitud_datos,
            },
            direccion,
            etapa,
        };
    }
}
