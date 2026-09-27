# Roadmap de TcpStoryTeller

Documento de seguimiento del estado actual del proyecto, arquitectura completada y tareas pendientes para llevar **TcpStoryTeller** a su versión final.

---

## 📌 Estado Actual del Proyecto (Completado)

El núcleo del flujo de captura, procesamiento y visualización se encuentra operativo y validado en entorno local y LAN:

- [x] **Captura de Red (`pcapture`)**: Ingesta de paquetes mediante sockets crudos (`AF_PACKET`) con soporte para filtros BPF automáticos (`tcp port X`) y detección de interfaces activas.
- [x] **Parseo Zero-Copy (`etherparse`)**: Decodificación de capas Ethernet, IPv4/IPv6 y cabeceras TCP con extracción de flags de control (`SYN`, `ACK`, `FIN`, `RST`, `PSH`, `URG`).
- [x] **Tracking de Sesión e ISN Relativos**:
  - Detección automática de extremos (Cliente vs. Servidor).
  - Seguimiento del ISN (*Initial Sequence Number*) para calcular números de secuencia (`seq_rel`) y acuse de recibo (`ack_rel`) normalizados desde 0.
- [x] **Máquina de Deducción de Etapas (`deducir_etapa`)**:
  - Handshake de 3 vías (`SYN`, `SYN-ACK`, `ACK`).
  - Transferencia de datos con flags `PSH`.
  - Cierre ordenado (`FIN`, `ACK-FIN`, `ACK`) y cierres abruptos (`RST`).
  - Detección preliminar de ACKs acumulativos, duplicados y sondas de ventana.
- [x] **Visor Interactivo en Terminal (`crossterm`)**:
  - Renderizado en pantalla alterna (*Alternate Screen*) con ocultamiento de cursor.
  - Modo *Raw* para captura inmediata de teclado sin esperar `<Enter>`.
  - Limpieza segura de terminal garantizada mediante RAII (*Drop Guard* en `HandleTerminal`).
  - Navegación bidireccional de diapositivas paso a paso (flechas `←` / `→`, `Enter`, tecla `q` para salir).
  - Formateo detallado de diagramas de caja ANSI con inspección de payloads (UTF-8, JSON pretty-printed y vistas hexadecimales).
- [x] **Configuración de Lints**:
  - Supresión de ruidos de desarrollo (`unused`, `nonstandard_style`) configurados a nivel de `Cargo.toml` y `src/main.rs`.

---

## 🚀 Tareas Pendientes

```
TcpStoryTeller/
├── 1. Opciones TCP & Ventana Escalada (Alta prioridad)
├── 2. Parametrización CLI con `clap`
├── 3. Escenarios Faltantes & Escenario Personalizado
├── 4. Soporte P2P / Modo Dos Nodos
└── 5. Ajustes de Robustez y UX
```

---

### 1. Opciones TCP y Cálculo de Ventana Escalada (WScale & MSS)
> **Objetivo:** Resolver el parseo de opciones de cabecera y calcular con precisión matemática el tamaño real del buffer de recepción (`rwnd`).

- [ ] **Parseo de Opciones en `estados.rs` / `datagrama_tcp.rs`**:
  - Actualmente `PasoTCP::new` recibe `None` para las opciones TCP.
  - Integrar el iterador de opciones expuesto por `etherparse` (`tcp.options()` o slice de cabecera).
  - Modelar y extraer:
    - **MSS (*Maximum Segment Size*)**: Anunciado en `SYN` / `SYN-ACK`.
    - **WScale (*Window Scale*, RFC 7323)**: Factor de desplazamiento de ventana ($2^{\text{scale}}$).
    - **SACK Permitted**: Soporte para acuses de recibo selectivos.
    - **Timestamps** (`TSval` / `TSecr`): Marcas de tiempo de ida y vuelta.
    - **NOP / EOL**: Padding de cabecera.
- [ ] **Cálculo de Ventana Real**:
  - Almacenar los factores de escala negociados en el handshake (`wscale_cliente` y `wscale_servidor`) dentro de `InformeSesion`.
  - Multiplicar la ventana cruda del header por el factor correspondiente:
    $$\text{rwnd\_real} = \text{window} \times 2^{\text{wscale}}$$
  - Mostrar en la TUI tanto el valor crudo como el valor escalado real (evitando que la ventana figure de 64 bytes durante la transferencia de datos).

---

### 2. Parametrización de Línea de Comandos (`clap`)
> **Objetivo:** Eliminar variables hardcodeadas en `main.rs` para permitir ejecución flexible en cualquier red o escenario.

- [ ] **Estructura CLI Principal**:
  - Flags globales:
    - `-i, --interfaz <IFACE>`: Especificar interfaz de red (`lo`, `wlan0`, `eth0`), con fallback a la primera interfaz activa no-loopback.
    - `-p, --puerto <PORT>`: Puerto de escucha/filtrado (default: `8080`).
    - `-t, --timeout <SEGUNDOS>`: Tiempo de inactividad antes de finalizar la captura (default: 6s).
- [ ] **Subcomandos de Ejecución**:
  - `tcpteller run <escenario>`: Ejecuta y visualiza un escenario automatizado.
  - `tcpteller listen`: **Modo Pasivo (Sniffer Puro)**. No crea conexiones ni abre sockets; simplemente escucha en la interfaz seleccionada y muestra el flujo interactivo de cualquier tráfico TCP coincidente.

---

### 3. Implementación de Escenarios de Tráfico
> **Objetivo:** Poblar los módulos en `src/escenarios/` para ilustrar diferentes comportamientos del protocolo TCP.

- [ ] **`rechazo_conexion.rs`**:
  - Cliente intentando conectar a un puerto local o remoto sin servicio a la escucha.
  - Demostración de recepción del paquete de aborto inmediato (`RST` / `ACK-RST`).
- [ ] **`peticion_http.rs`**:
  - Handshake de 3 vías.
  - Envío de request `GET / HTTP/1.1\r\nHost: ...\r\n\r\n` con flag `PSH`.
  - Recepción de respuesta HTTP `200 OK` con cabeceras y cuerpo HTML/JSON.
  - Cierre ordenado en 4 pasos (`FIN-ACK`).
- [ ] **`local_rafaga.rs`**:
  - Envío de múltiples segmentos de datos consecutivos sin esperar ACKs individuales inmediatos.
  - Observación del comportamiento de ACKs acumulativos (`ack_acumulativo`) y reducción progresiva de la ventana de recepción.
- [ ] **`personalizado.rs` (Custom)**:
  - Escenario configurable por parámetros:
    - IP y puerto destino arbitrarios.
    - Tamaño del payload en bytes.
    - Delay/pausa configurable entre paquetes (para ver tiempos relativos).
    - Cierre forzado mediante `RST` voluntario (`SO_LINGER = 0`) o cierre estándar `FIN`.

---

### 4. Soporte P2P y Entornos en Red (Dos Máquinas)
> **Objetivo:** Permitir que dos dispositivos distintos en una red local (LAN o Wi-Fi) participen en la captura y visualización.

- [ ] **Modo Servidor Independiente (`tcpteller server`)**:
  - Abre un `TcpListener` en `0.0.0.0:<puerto>`, acepta un socket y responde con un mensaje de bienvenida o eco.
- [ ] **Modo Cliente Independiente (`tcpteller client`)**:
  - Conecta mediante `TcpStream` a la IP de destino (`--target <IP:PORT>`), envía datos y cierra la conexión.
- [ ] **Filtro BPF Cruzado en LAN**:
  - Ajustar el filtro BPF de `pcapture` para aislar el flujo exacto entre los dos pares: `tcp and (host IP_A and host IP_B) and port P`.

---

### 5. Robustez y Pequeños Ajustes
- [ ] **Control de Límites en el Visor (`presentacion.rs`)**:
  - Agregar guarda de rango (`if indice + 1 < diapositivas.len()`) en los eventos de tecla `Right` y `Enter` para prevenir posibles pánicos por índice fuera de límites en la última diapositiva.
- [ ] **Exportación / Guardado de Informes**:
  - Opcional: Flag `--export <archivo.json>` para persistir la traza de la sesión analizada para uso posterior o docencia.
