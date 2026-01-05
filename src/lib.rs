// Biblioteca principal de win_opt
//
// Este módulo expone todos los componentes de la aplicación
// organizados en submódulos según su responsabilidad.

pub mod app;
pub mod cleanup;
pub mod error;
pub mod optimization;
pub mod theme;
pub mod types;
pub mod utils;

// Re-exportar los tipos principales para facilitar su uso
pub use app::App;
pub use error::{Result, WinOptError};
pub use theme::{ColorPalette, Theme};
pub use types::{CleanStats, OperationState, View};
pub use utils::{format_uptime, is_admin};
