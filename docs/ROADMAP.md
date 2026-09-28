# Roadmap de tcp_story_teller

## Completado

- [x] **Captura de Red**: Ingesta de paquetes crudos con BPF (`pcapture`).
- [x] **Parseo Zero-Copy**: Decodificación de capas Ethernet, IPv4/IPv6 y TCP (`etherparse`).
- [x] **Tracking e ISN Relativos**: Normalización de secuencia y acuse de recibo desde 0 (`seq_rel`, `ack_rel`).
- [x] **Deducción de Estados FSM**:
  - Handshake de 3 vías (`SYN`, `SYN-ACK`, `ACK`).
  - Transferencia de datos con flags `PSH`.
  - Cierre ordenado (`FIN-ACK`) y aborto inmediato (`RST`).
- [x] **Opciones TCP y Factor de Escala**: Extracción de `MSS` y factor de escala de ventana `WScale` (RFC 7323) con preservación del dato crudo y multiplicador dedicado.
- [x] **Formateo Dinámico de Flags**: Soporte para combinaciones arbitrarias de banderas activas.
- [x] **TUI Interactiva (`crossterm`)**:
  - Consola modular en 3 paneles ANSI (Flujo, Inspección de cabecera y Explicación didáctica).
  - Centrado dinámico sensible al tamaño de terminal.
  - Navegación bidireccional (`←`, `→`, `Enter`, `q`).
  - Limpieza segura de terminal garantizada por RAII (`Drop`).
- [x] **CLI con `clap`**: Subcomandos `escuchar`, `conectar`, `web` y `localmente`.
- [x] **Escenarios Operativos**:
  - `localmente`: Transacción atómica offline sin dependencias externas.
  - `web`: Peticiones contra WAN real (`httpbin.org`), modo simple, pesado (autotuning de ventana) y ráfaga con pipelining.
  - P2P en red local: Comunicación cliente-servidor validada entre dos computadoras físicas.
- [x] **Optimización de Compilación**: Perfil release con LTO y strip habilitados (binario de 1.1 MB).
- [x] **Manejo de Errores Idiomático**: Propagación limpia con `?` y eliminación de panics/unwraps en captura e inicialización.

---

## Próximas Tareas

### 1. Nuevos Protocolos de Red y Transporte

- [ ] **UDP (User Datagram Protocol)**:
  - Ingesta y parseo de datagramas no orientados a conexión.
  - Inspección de cabecera de 8 bytes (puerto origen, puerto destino, longitud y checksum).
  - Demostración de ausencia de estados (sin handshake ni acuses de recibo).
- [ ] **DNS (Domain Name System, RFC 1035)**:
  - Captura de consultas y respuestas sobre UDP (puerto 53).
  - Desglose de flags: `QR` (Query/Response), `AA` (Authoritative), `RD` (Recursion Desired), `RA` (Recursion Available).
  - Parseo de secciones `Questions`, `Answers` (registros A, AAAA, CNAME, MX) y punteros de compresión de nombres (`0xc0..`).
- [ ] **TLS 1.3 (Transport Layer Security)**:
  - Inspección del establecimiento de canal seguro sobre TCP (puerto 443).
  - Desglose de mensajes en texto claro del handshake: `ClientHello` (ciphersuites, extensiones SNI y KeyShare) y `ServerHello`.
  - Visualización del punto de transición a registros cifrados (`Application Data`).
- [ ] **ICMP e ICMPv6 (Internet Control Message Protocol)**:
  - Soporte de datagramas encapsulados directamente en IP (sin capa de transporte).
  - `Echo Request` y `Echo Reply` (mecanismo básico de ping).
  - `Time Exceeded` (Type 11): Demostración didáctica del funcionamiento de traceroute mediante manipulación del TTL.
  - `Destination Unreachable` / `Fragmentation Needed` (Type 3, Code 4): Demostración de Path MTU Discovery (PMTUD) ante datagramas con flag `DF` que exceden el MTU.
- [ ] **DHCP (Dynamic Host Configuration Protocol)**:
  - Seguimiento del ciclo DORA completo en broadcast sobre UDP (puertos 67/68): `Discover`, `Offer`, `Request` y `Acknowledge`.
  - Inspección de opciones de configuración de red entregadas al host (IP, máscara de subred, gateway y DNS).
- [ ] **ARP (Address Resolution Protocol)**:
  - Resolución de direcciones entre Capa 3 (IP) y Capa 2 (direcciones físicas MAC Ethernet).
  - Inspección de tramas de petición en broadcast (`who-has`) y respuesta unicast (`is-at`).

### 2. Funcionalidades de Alto Nivel

- [ ] **Exportación e Importación de archivos PCAP / PCAPNG**:
  - Apertura de archivos de captura generados externamente con Wireshark o tcpdump (`tcp_story_teller abrir captura.pcap`) para reproducir sesiones paso a paso en la TUI.
  - Exportación de la sesión analizada a formato `.pcap` / `.pcapng` para persistencia y uso docente.
- [ ] **Refactor de Errores con `thiserror`**:
  - Definición de tipos de error estructurados y fuertemente tipados en `captura::errores`.
  - Reporte descriptivo y contextualizado en `main.rs`, eliminando descarte silencioso de fallos.

### 3. Inyección de Anomalías (Modo Caos y Diagnóstico)

- [ ] **Retransmisión Rápida Forzada**:
  - Simulación de pérdida deliberada de un segmento de datos para forzar la llegada de 3 ACKs duplicados y verificar el disparo inmediato de retransmisión rápida.
- [ ] **Corrupción de Checksum**:
  - Inyección de bytes corruptos en el campo de checksum para evidenciar el descarte silencioso en el kernel del receptor.
- [ ] **Entrega Fuera de Orden (Out-of-Order Delivery)**:
  - Alteración deliberada del orden de los paquetes para observar cómo el receptor almacena en buffer intermedio y solicita los segmentos faltantes mediante ACKs acumulativos selectivos.
