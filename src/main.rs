use std::net::{IpAddr, Ipv4Addr};

use clap::{Parser, Subcommand};
use tcp_story_teller::{
    Extremo, PasoTCP, ejecutar_escenario_web_pesado, ejecutar_escenario_web_rafaga,
    ejecutar_escenario_web_simple, escenario_local,
};
use tcp_story_teller::{enviar_datos, escucha_pasiva, renderizar_diapositivas};

#[derive(Parser, Debug)]
#[command(
    name = "TcpStoryTeller",
    author = "ImAguss <samperagustin19@gmail.com>",
    version,
    about = "Analizador de paquetes interactivo del protocolo TCP.",
    long_about = None
)]
struct CLI {
    #[command(subcommand)]
    comando: Comando,
}

#[derive(Subcommand, Debug, Clone)]
enum Comando {
    Escuchar {
        #[arg(short, long, default_value_t = 4000)]
        puerto: u16,
        #[arg(short, long)]
        interfaz: Option<String>,
        #[arg(short, long, default_value_t = false)]
        wireshark: bool,
    },

    Conectar {
        #[arg(short = 'i', long)]
        ip: IpAddr,
        #[arg(short, long, default_value_t = 4000)]
        puerto: u16,
        #[arg(short = 'I', long)]
        interfaz: Option<String>,
        #[arg(short, long, default_value_t = false)]
        wireshark: bool,
    },

    Web {
        #[arg(short, long)]
        rafaga: bool,
        #[arg(short, long)]
        pesado: bool,
        #[arg(short, long)]
        interfaz: Option<String>,
        #[arg(short, long, default_value_t = false)]
        wireshark: bool,
    },
    Localmente {
        #[arg(short, long, default_value_t = false)]
        wireshark: bool,
    },
}

fn main() {
    let cli = CLI::parse();

    let diapositivas: Option<Vec<PasoTCP>> = match cli.comando {
        Comando::Escuchar {
            puerto,
            interfaz,
            wireshark,
        } => {
            let pc_local = Extremo {
                ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                puerto,
            };
            let diapositivas = match escucha_pasiva(pc_local, interfaz, wireshark) {
                Ok(d) => Some(d.pasos),
                Err(_) => None,
            };
            diapositivas
        }

        Comando::Conectar {
            ip,
            puerto,
            interfaz,
            wireshark,
        } => {
            let otra_pc = Extremo { ip, puerto };
            match enviar_datos(otra_pc, interfaz, wireshark) {
                Ok(d) => Some(d.pasos),
                Err(_) => None,
            }
        }

        Comando::Web {
            rafaga,
            pesado,
            interfaz,
            wireshark,
        } => {
            let diapositivas = if rafaga {
                match ejecutar_escenario_web_rafaga(interfaz, wireshark) {
                    Ok(d) => Some(d.pasos),
                    Err(_) => None,
                }
            } else if pesado {
                match ejecutar_escenario_web_pesado(interfaz, wireshark) {
                    Ok(d) => Some(d.pasos),
                    Err(_) => None,
                }
            } else {
                match ejecutar_escenario_web_simple(interfaz, wireshark) {
                    Ok(d) => Some(d.pasos),
                    Err(_) => None,
                }
            };
            diapositivas
        }

        Comando::Localmente { wireshark } => match escenario_local(wireshark) {
            Ok(d) => Some(d.pasos),
            Err(_) => None,
        },
    };

    if let Some(diapositivas) = diapositivas {
        if let Err(e) = renderizar_diapositivas(diapositivas) {
            eprintln!("Error al renderizar las diapositivas: {}", e);
        }
    } else {
        eprintln!("No se pudieron extraer diapositivas.")
    }
}
