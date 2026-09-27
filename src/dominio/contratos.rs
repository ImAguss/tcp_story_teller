use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TipoPeticion {
    ObtenerUsuario(u32),
    ObtenerTodosLosUsuarios,
    CargarPaginaWeb(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Peticion {
    pub id_peticion: u32,
    pub accion: TipoPeticion,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Usuario {
    pub id: u32,
    pub nombre: String,
    pub correo: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaginaWeb {
    pub ruta: String,
    pub titulo: String,
    pub contenido_html: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DatosRespuesta {
    Usuario(Option<Usuario>),
    Usuarios(Vec<Usuario>),
    Web(PaginaWeb),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Respuesta {
    pub id_secuencia: u32,
    pub datos: DatosRespuesta,
}
