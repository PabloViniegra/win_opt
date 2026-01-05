// Biblioteca principal de win_opt
//
// Este módulo expone todos los componentes de la aplicación
// organizados en submódulos según su responsabilidad.

pub mod animation;
pub mod app;
pub mod cleanup;
pub mod config;
pub mod error;
pub mod i18n;
pub mod logger;
pub mod optimization;
pub mod theme;
pub mod types;
pub mod utils;

// Re-exportar los tipos principales para facilitar su uso
pub use animation::{progress_bar, sparkline, Pulse, Spinner};
pub use app::App;
pub use config::Config;
pub use error::{Result, WinOptError};
pub use i18n::{I18n, I18nKey, Language};
pub use logger::{log, LogLevel};
pub use theme::{ColorPalette, Theme};
pub use types::{CleanStats, OperationState, View};
pub use utils::{format_uptime, is_admin};
