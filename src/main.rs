use std::net::{IpAddr, Ipv4Addr};

use TcpStoryTeller::{
    Extremo, PasoTCP, ejecutar_escenario_web_rafaga, ejecutar_escenario_web_simple, escenario_local,
};
use TcpStoryTeller::{enviar_datos, escucha_pasiva, renderizar_diapositivas};
use clap::{Parser, Subcommand};

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
    },

    Conectar {
        #[arg(short, long)]
        ip: IpAddr,
        #[arg(short, long, default_value_t = 4000)]
        puerto: u16,
        #[arg(short, long)]
        interfaz: Option<String>,
    },

    Web {
        #[arg(short, long)]
        simple: bool,
        #[arg(short, long)]
        verbose: bool,
        #[arg(short, long)]
        interfaz: Option<String>,
    },
    Localmente,
}

fn main() {
    let cli = CLI::parse();

    let diapositivas: Option<Vec<PasoTCP>> = match cli.comando {
        Comando::Escuchar { puerto, interfaz } => match escucha_pasiva(puerto, interfaz) {
            Ok(d) => Some(d.pasos),
            Err(_) => None,
        },
        Comando::Conectar {
            ip,
            puerto,
            interfaz,
        } => {
            let otra_pc = Extremo { ip, puerto };
            match enviar_datos(otra_pc, interfaz) {
                Ok(d) => Some(d.pasos),
                Err(_) => None,
            }
        }
        Comando::Web {
            simple,
            verbose,
            interfaz,
        } => {
            let diapositivas = if verbose {
                match ejecutar_escenario_web_rafaga(interfaz) {
                    Ok(d) => Some(d.pasos),
                    Err(_) => None,
                }
            } else {
                match ejecutar_escenario_web_simple(interfaz) {
                    Ok(d) => Some(d.pasos),
                    Err(_) => None,
                }
            };
            diapositivas
        }
        Comando::Localmente => match escenario_local() {
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
