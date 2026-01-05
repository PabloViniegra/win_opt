use thiserror::Error;

/// Errores de la aplicación win_opt
#[derive(Error, Debug)]
pub enum WinOptError {
    #[error("Operación requiere permisos de administrador")]
    AdminRequired,

    #[error("Error de I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("Comando {command} falló: {reason}")]
    CommandFailed { command: String, reason: String },

    #[error("Ruta no válida: {0}")]
    InvalidPath(String),

    #[error("Servicio no permitido: {0}")]
    InvalidService(String),

    #[error("Error inesperado: {0}")]
    Unknown(String),
}

/// Alias para Result con WinOptError
pub type Result<T> = std::result::Result<T, WinOptError>;
