# TCP Story Teller

> Analizador y narrador interactivo del protocolo TCP en la terminal. Diseñado para aprender y enseñar el protocolo a bajo nivel paso a paso, eliminando el ruido y la sobrecarga visual de herramientas como Wireshark.

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2024-orange?logo=rust" alt="Rust Edition 2024">
  <img src="https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-blue" alt="Platform: Linux | macOS | Windows">
  <img src="https://img.shields.io/badge/Binary%20Size-1.1%20MB-brightgreen" alt="Binary Size">
  <img src="https://img.shields.io/badge/License-GPLv3-blue.svg" alt="License: GPL v3">
</p>

---

## Demostración de la Herramienta

La herramienta cuenta con una interfaz **TUI (Terminal User Interface)** en pantalla alterna que dibuja una consola modular de 3 paneles ANSI, permitiendo navegar cada paquete de la sesión como una diapositiva interactiva:

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║  PASO #3   (+0.148s)                           Cliente ──────────> Servidor  ║
║  Etapa: Establecimiento (3/3): Conexión Establecida [ACK]                    ║
╚══════════════════════════════════════════════════════════════════════════════╝

┌── [ INSPECCIÓN DE CABECERA TCP ] ────────────────────────────────────────────┐
│                                                                              │
│  Flujo:       192.168.1.40:46618  ────────────►  44.219.150.215:80           │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│  Secuencia (Seq):         1          (Absoluto: 2288953654)                  │
│  Reconocimiento (Ack):    1          (Absoluto: 468406177)                   │
│  Flags de Control:        [ ACK ]                                            │
├──────────────────────────────────────────────────────────────────────────────┤
│  Ventana de Recepción:    63 bytes                                           │
│  Factor de Escala (WS):   ×1024 (shift: 10)                                  │
│  Tamaño Cabecera:         32 bytes (Base: 20B + Opciones: 12B)               │
│  Opciones TCP:            Ninguna                                            │
│                                                                              │
│  Carga Útil (Payload):    0 bytes (Tráfico de control puro)                  │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘

┌── [ EXPLICACIÓN DIDÁCTICA ] ─────────────────────────────────────────────────┐
│                                                                              │
│  El cliente confirma el SYN-ACK del servidor (Ack: 1). Concluye el saludo    │
│  de tres vías: la conexión pasa a estado ESTABLECIDA y ambos extremos        │
│  quedan habilitados para transmitir datos.                                   │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Controles de Navegación

| Tecla | Acción |
| :--- | :--- |
| `→` o `Enter` | Avanzar al siguiente paso del protocolo |
| `←` | Retroceder al paso anterior |
| `q` | Salir del visor y restaurar la terminal |

---

## Características Principales

- **Enfoque Didáctico (Estilo Kurose & Ross):** Inspirado en el enfoque *Top-Down* de redes de computadoras. Cada paso del ciclo de vida (Handshake de 3 vías, Transferencia de Datos, Control de Flujo con ACKs acumulativos y Cierre ordenado/abrupto) se explica en lenguaje claro y riguroso.
- **Normalización de Secuencias (ISN Relativo):** En una conexión real, los números de secuencia iniciales son enteros pseudoaleatorios de 32 bits (ej: `2288953654`). La herramienta normaliza automáticamente la secuencia arrancando desde `Seq: 0..N` y `Ack: 0..N`, facilitando el seguimiento de los bytes transmitidos.
- **Control de Flujo Real (RFC 7323):** Detección automática del factor de escala (*Window Scale*). Muestra tanto la ventana cruda que viaja en la cabecera como el multiplicador (`×1024`, `×256`), permitiendo al estudiante calcular el buffer real y observar el crecimiento dinámico (*TCP Autotuning*).
- **Alto Rendimiento y Zero-Copy:** Construido 100% en Rust. Decodificación a nivel de bits con `etherparse` sobre sockets crudos con filtros BPF (`pcapture`). Binario final ultra-optimizado de tan solo **1.1 MB** con tiempo de inicio inferior a 1 milisegundo.
- **TUI Segura y Robusta (RAII):** Manejo de terminal con `crossterm` ejecutado en pantalla alterna (*Alternate Screen*) y modo *Raw*. Implementa el patrón *Drop Guard* para garantizar que la terminal se restaure siempre limpiamente, incluso ante cancelaciones abruptas.

---

## Instalación y Requisitos

### Requisitos Previos

- **Rust y Cargo:** Versión estable reciente (edición 2024 soportada).
- **Dependencias nativas por plataforma:**
  - **Linux:** Requiere las bibliotecas de desarrollo de `libpcap`:
    ```bash
    sudo apt install libpcap-dev    # Debian / Ubuntu / Mint
    sudo dnf install libpcap-devel  # Fedora / RHEL
    sudo pacman -S libpcap          # Arch Linux
    ```
  - **macOS:** Ya incluye `libpcap` de manera predeterminada en el sistema.
  - **Windows:** No requiere configurar variables de entorno del SDK manualmente. El repositorio ya incluye las bibliotecas de importación (`Packet.lib` y `wpcap.lib`) en el directorio `./lib/`, y `build.rs` las enlaza automáticamente al compilar.

### 1. Clonar y Compilar

```bash
git clone https://github.com/ImAguss/tcp_story_teller
cd tcp_story_teller
cargo build --release
```

### 2. Configuración de Permisos y Controladores de Red

La captura y decodificación de paquetes crudos en interfaces de red requiere privilegios de bajo nivel en el sistema operativo.

#### Linux
El binario puede ejecutarse anteponiendo `sudo` o, de forma recomendada, otorgándole capacidades de red para no requerir privilegios de superusuario en cada ejecución:

```bash
sudo setcap cap_net_raw,cap_net_admin=eip target/release/tcp_story_teller
./target/release/tcp_story_teller web
```

#### macOS
En sistemas basados en BSD/macOS, la captura accede a los dispositivos `/dev/bpf*`. Requiere ejecutar el binario anteponiendo `sudo`:

```bash
sudo ./target/release/tcp_story_teller web
```

*(Opcional: Si se tiene instalado Wireshark, el paquete complementario `ChmodBPF` permite dar acceso a los dispositivos BPF a usuarios locales sin usar `sudo`).*

#### Windows
En Windows, el sistema operativo no expone sockets crudos directamente para captura en modo promiscuo. Se requiere instalar el controlador de captura **Npcap**:

1. Descargar e instalar [Npcap](https://npcap.com/#download).
2. Durante el asistente de instalación, asegurarse de marcar obligatoriamente las siguientes casillas:
   - **"Install Npcap in WinPcap API-compatible Mode"** (provee las DLLs en el sistema para la captura de red).
   - **"Support loopback traffic ("Npcap Loopback Adapter")"** (indispensable para capturar en el modo de simulación `localmente`).
3. Abrir **PowerShell** o **Windows Terminal** con la opción **"Ejecutar como Administrador"**.
4. Ejecutar el binario compilado:
   ```powershell
   .\target\release\tcp_story_teller.exe web
   ```

---

## Modos de Uso

La herramienta cuenta con una interfaz de línea de comandos declarativa construida con `clap`:

### 1. Simulación Local (`localmente`)
Ejecuta una transacción cliente-servidor atómica a través de la interfaz de loopback (`lo`). Es ideal para aprender y experimentar sin necesidad de conexión a internet:
```bash
./target/release/tcp_story_teller localmente
```

### 2. Conexión WAN Real contra Servidores Web (`web`)
Realiza peticiones HTTP reales contra `httpbin.org` cruzando la red WAN pública. Ofrece tres modalidades:

- **Modo Simple (por defecto):** Petición `GET` básica con cierre inmediato:
  ```bash
  ./target/release/tcp_story_teller web
  ```
- **Modo Pesado (`--pesado` o `-p`):** Envío de un payload POST de gran tamaño para observar la segmentación en MSS, los ACKs acumulativos y cómo el kernel del servidor expande dinámicamente su ventana de recepción:
  ```bash
  ./target/release/tcp_story_teller web --pesado
  ```
- **Modo Ráfaga (`--rafaga` o `-r`):** Pipelining HTTP con peticiones simultáneas sobre la misma conexión TCP persistente (`keep-alive` y `close`):
  ```bash
  ./target/release/tcp_story_teller web --rafaga
  ```
- *(Opcional: podés especificar la interfaz de red física con `-I <interfaz>`, por ejemplo `-I eth0`).*

### 3. Modo Peer-to-Peer (Entre dos Computadoras en LAN)
Permite inspeccionar el intercambio de paquetes entre dos máquinas físicas distintas dentro de la misma red local:

- **Paso 1 (Máquina Servidor):** Se pone a la escucha en un puerto específico:
  ```bash
  ./target/release/tcp_story_teller escuchar -p 4000
  ```
- **Paso 2 (Máquina Cliente):** Se conecta a la IP del servidor y transmite los datos:
  ```bash
  ./target/release/tcp_story_teller conectar -i 192.168.1.40 -p 4000
  ```

Una vez concluida la transmisión, ambas computadoras desplegarán en simultáneo el visor interactivo con la historia exacta de los paquetes capturados.

---

## Arquitectura y Stack Tecnológico

| Componente | Crate | Responsabilidad |
| :--- | :--- | :--- |
| **Captura de Red** | [`pcapture`](https://crates.io/crates/pcapture) | Ingesta de paquetes crudos mediante `AF_PACKET` con filtrado BPF en kernel (`tcp port X`). |
| **Parseo Zero-Copy** | [`etherparse`](https://crates.io/crates/etherparse) | Decodificación a nivel de bits de capas Ethernet, IPv4/IPv6, cabecera TCP y opciones. |
| **Interfaz TUI** | [`crossterm`](https://crates.io/crates/crossterm) | Renderizado en pantalla alterna, modo raw de teclado y centrado de paneles ANSI. |
| **CLI Declarativa** | [`clap`](https://crates.io/crates/clap) | Parseo estructurado de subcomandos, argumentos y flags. |
| **Inspección de Payload** | [`serde_json`](https://crates.io/crates/serde_json) | Pretty-printing automático de cargas útiles estructuradas en JSON. |

---

## Licencia y Créditos

- **Licencia:** Distribuido bajo la [Licencia Pública General de GNU v3.0 (GPLv3)](LICENSE).
- **Autor:** Agustin Samper ([@ImAguss](https://github.com/ImAguss))
- **Contacto:** samperagustin19@gmail.com
