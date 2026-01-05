//! Módulo de configuración para win_opt
//!
//! Maneja la configuración de la aplicación mediante archivos TOML.

use crate::i18n::Language;
use crate::theme::Theme;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuración de la aplicación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Configuración de apariencia
    pub appearance: AppearanceConfig,

    /// Configuración de idioma
    pub language: LanguageConfig,

    /// Configuración de logging
    pub logging: LoggingConfig,
}

/// Configuración de apariencia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    /// Tema de la aplicación (Light o Dark)
    pub theme: Theme,

    /// Recordar último tema usado
    pub remember_theme: bool,
}

/// Configuración de idioma
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// Idioma de la aplicación
    pub language: Language,

    /// Recordar último idioma usado
    pub remember_language: bool,
}

/// Configuración de logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Nivel de log (trace, debug, info, warn, error)
    pub level: String,

    /// Habilitar logging a archivo
    pub file_logging: bool,

    /// Mantener logs por N días
    pub retention_days: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            appearance: AppearanceConfig {
                theme: Theme::Dark,
                remember_theme: true,
            },
            language: LanguageConfig {
                language: Language::Spanish,
                remember_language: true,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_logging: true,
                retention_days: 7,
            },
        }
    }
}

impl Config {
    /// Obtiene el directorio de configuración de la aplicación
    ///
    /// En Windows: %APPDATA%\win_opt
    fn get_config_dir() -> std::io::Result<PathBuf> {
        let app_data = std::env::var("APPDATA").unwrap_or_else(|_| {
            std::env::var("USERPROFILE")
                .map(|p| format!("{}\\AppData\\Roaming", p))
                .unwrap_or_else(|_| "C:\\ProgramData".to_string())
        });

        let config_dir = PathBuf::from(app_data).join("win_opt");

        // Crear directorio si no existe
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        Ok(config_dir)
    }

    /// Obtiene la ruta del archivo de configuración
    fn get_config_file() -> std::io::Result<PathBuf> {
        let config_dir = Self::get_config_dir()?;
        Ok(config_dir.join("config.toml"))
    }

    /// Carga la configuración desde el archivo
    ///
    /// Si el archivo no existe, retorna la configuración por defecto.
    pub fn load() -> Self {
        match Self::load_from_file() {
            Ok(config) => {
                tracing::info!("Configuración cargada desde archivo");
                config
            }
            Err(e) => {
                tracing::warn!(
                    "No se pudo cargar la configuración: {}. Usando valores por defecto",
                    e
                );
                Self::default()
            }
        }
    }

    /// Intenta cargar la configuración desde el archivo
    fn load_from_file() -> std::io::Result<Self> {
        let config_file = Self::get_config_file()?;

        if !config_file.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Config file not found",
            ));
        }

        let contents = fs::read_to_string(&config_file)?;
        let config: Config = toml::from_str(&contents).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("TOML parse error: {}", e),
            )
        })?;

        Ok(config)
    }

    /// Guarda la configuración en el archivo
    pub fn save(&self) -> std::io::Result<()> {
        let config_file = Self::get_config_file()?;

        let toml_string = toml::to_string_pretty(self).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("TOML serialize error: {}", e),
            )
        })?;

        fs::write(&config_file, toml_string)?;

        tracing::info!("Configuración guardada en: {:?}", config_file);

        Ok(())
    }

    /// Crea un archivo de configuración de ejemplo con valores por defecto
    pub fn create_default_config_file() -> std::io::Result<PathBuf> {
        let config = Self::default();
        config.save()?;
        Self::get_config_file()
    }

    /// Obtiene el tema configurado
    pub fn theme(&self) -> Theme {
        self.appearance.theme
    }

    /// Establece el tema
    pub fn set_theme(&mut self, theme: Theme) {
        self.appearance.theme = theme;
    }

    /// Obtiene el idioma configurado
    pub fn language(&self) -> Language {
        self.language.language
    }

    /// Establece el idioma
    pub fn set_language(&mut self, language: Language) {
        self.language.language = language;
    }

    /// Guarda el tema si está configurado para recordarlo
    pub fn save_if_remember(&self) -> std::io::Result<()> {
        if self.appearance.remember_theme || self.language.remember_language {
            self.save()
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.appearance.theme, Theme::Dark);
        assert_eq!(config.language.language, Language::Spanish);
        assert!(config.appearance.remember_theme);
        assert!(config.logging.file_logging);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_string = toml::to_string(&config).unwrap();
        assert!(toml_string.contains("theme"));
        assert!(toml_string.contains("language"));
        assert!(toml_string.contains("logging"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            [appearance]
            theme = "Light"
            remember_theme = false

            [language]
            language = "English"
            remember_language = true

            [logging]
            level = "debug"
            file_logging = true
            retention_days = 30
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.appearance.theme, Theme::Light);
        assert_eq!(config.language.language, Language::English);
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.logging.retention_days, 30);
    }

    #[test]
    fn test_config_getters_setters() {
        let mut config = Config::default();

        config.set_theme(Theme::Light);
        assert_eq!(config.theme(), Theme::Light);

        config.set_language(Language::English);
        assert_eq!(config.language(), Language::English);
    }
}
