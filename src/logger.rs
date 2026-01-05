//! Módulo de logging estructurado para win_opt
//!
//! Este módulo proporciona funciones para registrar eventos de la aplicación
//! tanto en archivos de log como en la interfaz de usuario.

use crate::app::App;
use std::path::PathBuf;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Inicializa el sistema de logging
///
/// Configura tracing para escribir logs en archivos rotativos diarios
/// en el directorio de logs de la aplicación.
///
/// # Errores
///
/// Retorna un error si no se puede crear el directorio de logs o inicializar el logger.
pub fn init() -> std::io::Result<()> {
    let log_dir = get_log_directory()?;

    // Crear directorio de logs si no existe
    std::fs::create_dir_all(&log_dir)?;

    // Configurar appender con rotación diaria
    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "win_opt.log");

    // Configurar nivel de logging según variable de entorno o usar INFO por defecto
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"))
        .add_directive("win_opt=debug".parse().unwrap());

    // Configurar subscriber con formato estructurado
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true),
        )
        .init();

    tracing::info!("Sistema de logging inicializado");

    Ok(())
}

/// Obtiene el directorio donde se almacenarán los logs
///
/// En Windows, usa %APPDATA%\win_opt\logs
fn get_log_directory() -> std::io::Result<PathBuf> {
    let app_data = std::env::var("APPDATA")
        .or_else(|_| std::env::var("USERPROFILE").map(|p| format!("{p}\\AppData\\Roaming")))
        .unwrap_or_else(|_| "C:\\ProgramData".to_string());

    Ok(PathBuf::from(app_data).join("win_opt").join("logs"))
}

/// Niveles de logging para la aplicación
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    /// Información de debug detallada
    Debug,
    /// Información general de operaciones
    Info,
    /// Advertencias sobre situaciones no ideales
    Warning,
    /// Errores que requieren atención
    Error,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warning => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

/// Registra un mensaje en el sistema de logging y opcionalmente en la UI
///
/// # Argumentos
///
/// * `app` - Referencia opcional a la aplicación para agregar el log a la UI
/// * `level` - Nivel de severidad del log
/// * `message` - Mensaje a registrar
///
/// # Ejemplo
///
/// ```no_run
/// use win_opt::logger::{log, LogLevel};
///
/// log(None, LogLevel::Info, "Operación iniciada");
/// ```
pub fn log(app: Option<&mut App>, level: LogLevel, message: impl AsRef<str>) {
    let msg = message.as_ref();

    // Registrar en el sistema de logging estructurado
    match level {
        LogLevel::Debug => tracing::debug!("{}", msg),
        LogLevel::Info => tracing::info!("{}", msg),
        LogLevel::Warning => tracing::warn!("{}", msg),
        LogLevel::Error => tracing::error!("{}", msg),
    }

    // Agregar a la UI si se proporciona la app
    if let Some(app) = app {
        app.operation_logs.push(msg.to_string());
    }
}

/// Macro para simplificar el logging
///
/// # Ejemplo
///
/// ```ignore
/// log_info!(app, "Operación completada");
/// log_error!(app, "Error al procesar archivo");
/// ```
#[macro_export]
macro_rules! log_info {
    ($app:expr, $($arg:tt)*) => {
        $crate::logger::log(Some($app), $crate::logger::LogLevel::Info, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($app:expr, $($arg:tt)*) => {
        $crate::logger::log(Some($app), $crate::logger::LogLevel::Debug, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($app:expr, $($arg:tt)*) => {
        $crate::logger::log(Some($app), $crate::logger::LogLevel::Warning, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($app:expr, $($arg:tt)*) => {
        $crate::logger::log(Some($app), $crate::logger::LogLevel::Error, format!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_directory() {
        let log_dir = get_log_directory().unwrap();
        assert!(log_dir.to_string_lossy().contains("win_opt"));
        assert!(log_dir.to_string_lossy().contains("logs"));
    }

    #[test]
    fn test_log_level_conversion() {
        assert_eq!(Level::from(LogLevel::Debug), Level::DEBUG);
        assert_eq!(Level::from(LogLevel::Info), Level::INFO);
        assert_eq!(Level::from(LogLevel::Warning), Level::WARN);
        assert_eq!(Level::from(LogLevel::Error), Level::ERROR);
    }
}
