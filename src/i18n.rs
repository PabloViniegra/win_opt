//! Módulo de internacionalización (i18n) para win_opt
//!
//! Proporciona soporte para múltiples idiomas (Español e Inglés).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Idiomas soportados por la aplicación
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    /// Español
    #[default]
    Spanish,
    /// Inglés
    English,
}

impl Language {
    /// Obtiene el código del idioma (para logging)
    pub fn code(&self) -> &'static str {
        match self {
            Language::Spanish => "es",
            Language::English => "en",
        }
    }

    /// Obtiene el nombre del idioma en su propio idioma
    pub fn native_name(&self) -> &'static str {
        match self {
            Language::Spanish => "Español",
            Language::English => "English",
        }
    }
}

/// Claves de traducción
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum I18nKey {
    // === App Info ===
    AppTitle,
    AppSubtitle,
    AppVersion,
    MainMenu,
    OperationsLog,

    // === Menu Items ===
    MenuTempFiles,
    MenuTempFilesDesc,
    MenuRecycleBin,
    MenuRecycleBinDesc,
    MenuBrowserCache,
    MenuBrowserCacheDesc,
    MenuSystemLogs,
    MenuSystemLogsDesc,
    MenuWindowsUpdate,
    MenuWindowsUpdateDesc,
    MenuOptimize,
    MenuOptimizeDesc,
    MenuStartup,
    MenuStartupDesc,
    MenuVisualEffects,
    MenuVisualEffectsDesc,
    MenuNetwork,
    MenuNetworkDesc,
    MenuRepair,
    MenuRepairDesc,
    MenuPrivacy,
    MenuPrivacyDesc,
    MenuInfo,
    MenuInfoDesc,
    MenuExit,
    MenuExitDesc,

    // === Footer ===
    FooterNavigate,
    FooterSelect,
    FooterBack,
    FooterExit,
    FooterScroll,
    FooterTheme,
    FooterLanguage,

    // === Operations ===
    OpStarting,
    OpCompleted,
    OpError,
    OpRequiresAdmin,
    OpPleaseRunAsAdmin,
    OpRebootRecommended,

    // === Clean Operation ===
    CleanTitle,
    CleanStarting,
    CleanDirectory,
    CleanItemsFound,
    CleanProcessing,
    CleanCompleted,
    CleanErrorReading,

    // === Statistics ===
    StatsTitle,
    StatsDeleted,
    StatsSkipped,
    StatsFreed,

    // === Network Operation ===
    NetworkTitle,
    NetworkStarting,
    NetworkDnsFlush,
    NetworkDnsSuccess,
    NetworkDnsError,
    NetworkWinsock,
    NetworkWinsockSuccess,
    NetworkWinsockError,

    // === Repair Operation ===
    RepairTitle,
    RepairStarting,
    RepairDism,
    RepairDismSuccess,
    RepairDismError,
    RepairSfc,
    RepairSfcSuccess,
    RepairSfcWarning,
    RepairWait,

    // === System Info ===
    InfoTitle,
    InfoOs,
    InfoVersion,
    InfoKernel,
    InfoHost,
    InfoArch,
    InfoUptime,
    InfoCpu,
    InfoCores,
    InfoMemTotal,
    InfoMemUsed,
    InfoMemUsage,
    InfoDisks,

    // === Browser Cache ===
    BrowserCacheTitle,
    BrowserCacheStarting,
    BrowserCacheCleaning,
    BrowserCacheSuccess,
    BrowserCacheNotFound,
    BrowserCacheCloseWarning,

    // === System Logs ===
    SystemLogsTitle,
    SystemLogsStarting,
    SystemLogsCleaning,
    SystemLogsProcessed,
    SystemLogsRequiresAdmin,

    // === Recycle Bin ===
    RecycleBinTitle,
    RecycleBinStarting,
    RecycleBinSuccess,
    RecycleBinWarning,
    RecycleBinFreed,

    // === Windows Update ===
    WindowsUpdateTitle,
    WindowsUpdateStarting,
    WindowsUpdateCleaning,
    WindowsUpdateDiskCleanup,
    WindowsUpdateComponents,
    WindowsUpdateCompleted,

    // === Optimization ===
    OptimizeTitle,
    OptimizeStarting,
    OptimizePrefetch,
    OptimizePower,
    OptimizeServices,
    OptimizeCompleted,

    // === Privacy ===
    PrivacyTitle,
    PrivacyStarting,
    PrivacyTelemetry,
    PrivacyTasks,
    PrivacyCompleted,

    // === Startup Optimizer ===
    StartupTitle,
    StartupAnalyzing,
    StartupGettingList,
    StartupFound,
    StartupDisableHint,
    StartupRecommendation,

    // === Visual Effects ===
    VisualEffectsTitle,
    VisualEffectsOptimizing,
    VisualEffectsApplying,
    VisualEffectsCompleted,
    VisualEffectsLogoutRequired,
    VisualEffectsHint,

    // === Generic Messages ===
    Success,
    Warning,
    Error,
    Info,
}

/// HashMap global de traducciones (inicializado una sola vez)
static TRANSLATIONS: OnceLock<HashMap<(Language, I18nKey), &'static str>> = OnceLock::new();

/// Sistema de traducciones (solo almacena el idioma actual)
pub struct I18n {
    current_language: Language,
}

impl I18n {
    /// Crea un nuevo sistema de i18n
    pub fn new(language: Language) -> Self {
        // Inicializa las traducciones globales si aún no están cargadas
        TRANSLATIONS.get_or_init(Self::init_translations);
        Self {
            current_language: language,
        }
    }

    /// Cambia el idioma actual
    pub fn set_language(&mut self, language: Language) {
        self.current_language = language;
    }

    /// Obtiene el idioma actual
    pub fn current_language(&self) -> Language {
        self.current_language
    }

    /// Alterna entre idiomas disponibles
    pub fn toggle_language(&mut self) {
        self.current_language = match self.current_language {
            Language::Spanish => Language::English,
            Language::English => Language::Spanish,
        };
    }

    /// Obtiene una traducción para la clave especificada
    pub fn t(&self, key: I18nKey) -> &str {
        TRANSLATIONS
            .get()
            .and_then(|t| t.get(&(self.current_language, key)))
            .copied()
            .unwrap_or("[MISSING TRANSLATION]")
    }

    /// Inicializa el HashMap de traducciones (se llama una sola vez)
    fn init_translations() -> HashMap<(Language, I18nKey), &'static str> {
        use I18nKey::*;
        use Language::*;

        // ============ ESPAÑOL ============
        let es_translations = [
            // App Info
            (AppTitle, "WIN OPT"),
            (AppSubtitle, "Windows 11 Optimizer"),
            (AppVersion, "v1.2.0"),
            (MainMenu, "Menú Principal"),
            (OperationsLog, "Registro de Operaciones"),
            // Menu Items
            (MenuTempFiles, "Archivos Temporales"),
            (MenuTempFilesDesc, "Limpia archivos temp del sistema"),
            (MenuRecycleBin, "Papelera de Reciclaje"),
            (MenuRecycleBinDesc, "Vacía la papelera completamente"),
            (MenuBrowserCache, "Caché de Navegadores"),
            (MenuBrowserCacheDesc, "Limpia Chrome, Firefox, Edge"),
            (MenuSystemLogs, "Logs del Sistema"),
            (MenuSystemLogsDesc, "Elimina archivos de registro"),
            (MenuWindowsUpdate, "Windows Update"),
            (MenuWindowsUpdateDesc, "Limpia archivos de actualización"),
            (MenuOptimize, "Optimización Avanzada"),
            (MenuOptimizeDesc, "Servicios, energía y prefetch"),
            (MenuStartup, "Programas de Inicio"),
            (MenuStartupDesc, "Optimiza arranque de Windows"),
            (MenuVisualEffects, "Efectos Visuales"),
            (MenuVisualEffectsDesc, "Deshabilita animaciones"),
            (MenuNetwork, "Red"),
            (MenuNetworkDesc, "DNS flush & Winsock reset"),
            (MenuRepair, "Reparación"),
            (MenuRepairDesc, "DISM & SFC scan"),
            (MenuPrivacy, "Privacidad"),
            (MenuPrivacyDesc, "Desactiva telemetría"),
            (MenuInfo, "Info del Sistema"),
            (MenuInfoDesc, "Detalles del hardware"),
            (MenuExit, "Salir"),
            (MenuExitDesc, "Cerrar aplicación"),
            // Footer
            (FooterNavigate, "Navegar"),
            (FooterSelect, "Seleccionar"),
            (FooterBack, "Volver al menú"),
            (FooterExit, "Salir"),
            (FooterScroll, "Scroll"),
            (FooterTheme, "Tema"),
            (FooterLanguage, "Idioma"),
            // Operations
            (OpStarting, "Iniciando operación..."),
            (OpCompleted, "Operación completada"),
            (OpError, "Error en la operación"),
            (
                OpRequiresAdmin,
                "ERROR: Esta operación requiere permisos de Administrador",
            ),
            (
                OpPleaseRunAsAdmin,
                "Por favor, ejecuta la aplicación como Administrador",
            ),
            (
                OpRebootRecommended,
                "Se recomienda reiniciar el sistema para aplicar los cambios",
            ),
            // Clean Operation
            (CleanTitle, "Limpieza de Archivos Temporales"),
            (
                CleanStarting,
                "Iniciando limpieza de archivos temporales...",
            ),
            (CleanDirectory, "Directorio:"),
            (CleanItemsFound, "Elementos encontrados:"),
            (CleanProcessing, "Procesando..."),
            (CleanCompleted, "Limpieza completada"),
            (CleanErrorReading, "Error al leer el directorio temporal"),
            // Statistics
            (StatsTitle, "Estadísticas"),
            (StatsDeleted, "Elementos eliminados:"),
            (StatsSkipped, "Elementos omitidos:"),
            (StatsFreed, "Espacio liberado:"),
            // Network
            (NetworkTitle, "Limpieza de Red"),
            (NetworkStarting, "Iniciando operaciones de red..."),
            (NetworkDnsFlush, "Ejecutando: ipconfig /flushdns"),
            (NetworkDnsSuccess, "Caché DNS limpiada exitosamente"),
            (NetworkDnsError, "Error al limpiar la caché DNS"),
            (NetworkWinsock, "Ejecutando: netsh winsock reset"),
            (NetworkWinsockSuccess, "Winsock reiniciado exitosamente"),
            (
                NetworkWinsockError,
                "Falló el reinicio de Winsock (se requieren permisos de administrador)",
            ),
            // Repair
            (RepairTitle, "Reparación del Sistema"),
            (RepairStarting, "Iniciando reparación del sistema..."),
            (
                RepairDism,
                "Ejecutando DISM (Deployment Image Servicing and Management)...",
            ),
            (RepairDismSuccess, "DISM finalizado correctamente"),
            (RepairDismError, "DISM finalizó con errores"),
            (RepairSfc, "Ejecutando SFC (System File Checker)..."),
            (RepairSfcSuccess, "Escaneo de archivos finalizado"),
            (RepairSfcWarning, "Escaneo finalizado con advertencias"),
            (RepairWait, "Esto puede tardar varios minutos..."),
            // System Info
            (InfoTitle, "Información del Sistema"),
            (InfoOs, "OS:"),
            (InfoVersion, "Versión:"),
            (InfoKernel, "Kernel:"),
            (InfoHost, "Host:"),
            (InfoArch, "Arquitectura:"),
            (InfoUptime, "Tiempo activo:"),
            (InfoCpu, "CPU:"),
            (InfoCores, "Núcleos:"),
            (InfoMemTotal, "Memoria Total:"),
            (InfoMemUsed, "Memoria Usada:"),
            (InfoMemUsage, "Uso de Memoria"),
            (InfoDisks, "Discos"),
            // Browser Cache
            (BrowserCacheTitle, "Caché de Navegadores"),
            (
                BrowserCacheStarting,
                "Iniciando limpieza de caché de navegadores...",
            ),
            (BrowserCacheCleaning, "Limpiando caché de"),
            (BrowserCacheSuccess, "Caché limpiada"),
            (BrowserCacheNotFound, "No encontrado o inaccesible"),
            (
                BrowserCacheCloseWarning,
                "Cierra los navegadores antes de ejecutar esta operación para mejores resultados",
            ),
            // System Logs
            (SystemLogsTitle, "Logs del Sistema"),
            (
                SystemLogsStarting,
                "Iniciando limpieza de logs del sistema...",
            ),
            (SystemLogsCleaning, "Limpiando:"),
            (SystemLogsProcessed, "procesado"),
            (
                SystemLogsRequiresAdmin,
                "Requiere permisos de administrador",
            ),
            // Recycle Bin
            (RecycleBinTitle, "Papelera de Reciclaje"),
            (
                RecycleBinStarting,
                "Iniciando vaciado de papelera de reciclaje...",
            ),
            (
                RecycleBinSuccess,
                "Papelera de reciclaje vaciada exitosamente",
            ),
            (
                RecycleBinWarning,
                "Advertencia: Algunas carpetas no pudieron vaciarse",
            ),
            (RecycleBinFreed, "Espacio en disco liberado"),
            // Windows Update
            (WindowsUpdateTitle, "Limpieza de Windows Update"),
            (
                WindowsUpdateStarting,
                "Iniciando limpieza de Windows Update...",
            ),
            (
                WindowsUpdateCleaning,
                "Eliminando archivos de actualización antiguos...",
            ),
            (WindowsUpdateDiskCleanup, "Limpieza de disco iniciada"),
            (
                WindowsUpdateComponents,
                "Ejecutando limpieza de componentes...",
            ),
            (
                WindowsUpdateCompleted,
                "Limpieza de Windows Update completada",
            ),
            // Optimization
            (OptimizeTitle, "Optimización Avanzada"),
            (
                OptimizeStarting,
                "Iniciando optimización avanzada del sistema...",
            ),
            (OptimizePrefetch, "Limpiando archivos Prefetch..."),
            (
                OptimizePower,
                "Configurando plan de energía de alto rendimiento...",
            ),
            (OptimizeServices, "Optimizando servicios del sistema..."),
            (OptimizeCompleted, "Optimización avanzada completada"),
            // Privacy
            (PrivacyTitle, "Privacidad y Telemetría"),
            (PrivacyStarting, "Iniciando configuración de privacidad..."),
            (PrivacyTelemetry, "Deshabilitando telemetría de Windows..."),
            (
                PrivacyTasks,
                "Deshabilitando tareas programadas de telemetría...",
            ),
            (PrivacyCompleted, "Configuración de privacidad completada"),
            // Startup
            (StartupTitle, "Programas de Inicio"),
            (StartupAnalyzing, "Analizando programas de inicio..."),
            (
                StartupGettingList,
                "Obteniendo lista de programas de inicio...",
            ),
            (StartupFound, "Programas de inicio encontrados:"),
            (
                StartupDisableHint,
                "Para deshabilitar programas: Ejecuta 'msconfig' o 'Administrador de tareas'",
            ),
            (
                StartupRecommendation,
                "Recomendación: Deshabilita programas innecesarios para acelerar el inicio",
            ),
            // Visual Effects
            (VisualEffectsTitle, "Efectos Visuales"),
            (VisualEffectsOptimizing, "Optimizando efectos visuales..."),
            (
                VisualEffectsApplying,
                "Aplicando configuraciones de rendimiento...",
            ),
            (VisualEffectsCompleted, "Efectos visuales optimizados"),
            (
                VisualEffectsLogoutRequired,
                "Los cambios se aplicarán después de cerrar sesión o reiniciar",
            ),
            (
                VisualEffectsHint,
                "Esto puede mejorar significativamente el rendimiento en equipos antiguos",
            ),
            // Generic
            (Success, "Éxito"),
            (Warning, "Advertencia"),
            (Error, "Error"),
            (Info, "Información"),
        ];

        // ============ ENGLISH ============
        let en_translations = [
            // App Info
            (AppTitle, "WIN OPT"),
            (AppSubtitle, "Windows 11 Optimizer"),
            (AppVersion, "v1.2.0"),
            (MainMenu, "Main Menu"),
            (OperationsLog, "Operation Log"),
            // Menu Items
            (MenuTempFiles, "Temporary Files"),
            (MenuTempFilesDesc, "Clean system temp files"),
            (MenuRecycleBin, "Recycle Bin"),
            (MenuRecycleBinDesc, "Empty recycle bin completely"),
            (MenuBrowserCache, "Browser Cache"),
            (MenuBrowserCacheDesc, "Clean Chrome, Firefox, Edge"),
            (MenuSystemLogs, "System Logs"),
            (MenuSystemLogsDesc, "Remove log files"),
            (MenuWindowsUpdate, "Windows Update"),
            (MenuWindowsUpdateDesc, "Clean update files"),
            (MenuOptimize, "Advanced Optimization"),
            (MenuOptimizeDesc, "Services, power and prefetch"),
            (MenuStartup, "Startup Programs"),
            (MenuStartupDesc, "Optimize Windows startup"),
            (MenuVisualEffects, "Visual Effects"),
            (MenuVisualEffectsDesc, "Disable animations"),
            (MenuNetwork, "Network"),
            (MenuNetworkDesc, "DNS flush & Winsock reset"),
            (MenuRepair, "Repair"),
            (MenuRepairDesc, "DISM & SFC scan"),
            (MenuPrivacy, "Privacy"),
            (MenuPrivacyDesc, "Disable telemetry"),
            (MenuInfo, "System Info"),
            (MenuInfoDesc, "Hardware details"),
            (MenuExit, "Exit"),
            (MenuExitDesc, "Close application"),
            // Footer
            (FooterNavigate, "Navigate"),
            (FooterSelect, "Select"),
            (FooterBack, "Back to menu"),
            (FooterExit, "Exit"),
            (FooterScroll, "Scroll"),
            (FooterTheme, "Theme"),
            (FooterLanguage, "Language"),
            // Operations
            (OpStarting, "Starting operation..."),
            (OpCompleted, "Operation completed"),
            (OpError, "Operation error"),
            (
                OpRequiresAdmin,
                "ERROR: This operation requires Administrator permissions",
            ),
            (
                OpPleaseRunAsAdmin,
                "Please run the application as Administrator",
            ),
            (
                OpRebootRecommended,
                "System restart recommended to apply changes",
            ),
            // Clean Operation
            (CleanTitle, "Temporary Files Cleanup"),
            (CleanStarting, "Starting temporary files cleanup..."),
            (CleanDirectory, "Directory:"),
            (CleanItemsFound, "Items found:"),
            (CleanProcessing, "Processing..."),
            (CleanCompleted, "Cleanup completed"),
            (CleanErrorReading, "Error reading temporary directory"),
            // Statistics
            (StatsTitle, "Statistics"),
            (StatsDeleted, "Items deleted:"),
            (StatsSkipped, "Items skipped:"),
            (StatsFreed, "Space freed:"),
            // Network
            (NetworkTitle, "Network Cleanup"),
            (NetworkStarting, "Starting network operations..."),
            (NetworkDnsFlush, "Running: ipconfig /flushdns"),
            (NetworkDnsSuccess, "DNS cache cleared successfully"),
            (NetworkDnsError, "Error clearing DNS cache"),
            (NetworkWinsock, "Running: netsh winsock reset"),
            (NetworkWinsockSuccess, "Winsock reset successfully"),
            (
                NetworkWinsockError,
                "Winsock reset failed (administrator permissions required)",
            ),
            // Repair
            (RepairTitle, "System Repair"),
            (RepairStarting, "Starting system repair..."),
            (
                RepairDism,
                "Running DISM (Deployment Image Servicing and Management)...",
            ),
            (RepairDismSuccess, "DISM completed successfully"),
            (RepairDismError, "DISM completed with errors"),
            (RepairSfc, "Running SFC (System File Checker)..."),
            (RepairSfcSuccess, "File scan completed"),
            (RepairSfcWarning, "Scan completed with warnings"),
            (RepairWait, "This may take several minutes..."),
            // System Info
            (InfoTitle, "System Information"),
            (InfoOs, "OS:"),
            (InfoVersion, "Version:"),
            (InfoKernel, "Kernel:"),
            (InfoHost, "Host:"),
            (InfoArch, "Architecture:"),
            (InfoUptime, "Uptime:"),
            (InfoCpu, "CPU:"),
            (InfoCores, "Cores:"),
            (InfoMemTotal, "Total Memory:"),
            (InfoMemUsed, "Used Memory:"),
            (InfoMemUsage, "Memory Usage"),
            (InfoDisks, "Disks"),
            // Browser Cache
            (BrowserCacheTitle, "Browser Cache"),
            (BrowserCacheStarting, "Starting browser cache cleanup..."),
            (BrowserCacheCleaning, "Cleaning cache from"),
            (BrowserCacheSuccess, "Cache cleaned"),
            (BrowserCacheNotFound, "Not found or inaccessible"),
            (
                BrowserCacheCloseWarning,
                "Close browsers before running this operation for best results",
            ),
            // System Logs
            (SystemLogsTitle, "System Logs"),
            (SystemLogsStarting, "Starting system logs cleanup..."),
            (SystemLogsCleaning, "Cleaning:"),
            (SystemLogsProcessed, "processed"),
            (
                SystemLogsRequiresAdmin,
                "Requires administrator permissions",
            ),
            // Recycle Bin
            (RecycleBinTitle, "Recycle Bin"),
            (RecycleBinStarting, "Starting recycle bin cleanup..."),
            (RecycleBinSuccess, "Recycle bin emptied successfully"),
            (
                RecycleBinWarning,
                "Warning: Some folders could not be emptied",
            ),
            (RecycleBinFreed, "Disk space freed"),
            // Windows Update
            (WindowsUpdateTitle, "Windows Update Cleanup"),
            (WindowsUpdateStarting, "Starting Windows Update cleanup..."),
            (WindowsUpdateCleaning, "Removing old update files..."),
            (WindowsUpdateDiskCleanup, "Disk cleanup started"),
            (WindowsUpdateComponents, "Running component cleanup..."),
            (WindowsUpdateCompleted, "Windows Update cleanup completed"),
            // Optimization
            (OptimizeTitle, "Advanced Optimization"),
            (OptimizeStarting, "Starting advanced system optimization..."),
            (OptimizePrefetch, "Cleaning Prefetch files..."),
            (OptimizePower, "Configuring high performance power plan..."),
            (OptimizeServices, "Optimizing system services..."),
            (OptimizeCompleted, "Advanced optimization completed"),
            // Privacy
            (PrivacyTitle, "Privacy and Telemetry"),
            (PrivacyStarting, "Starting privacy configuration..."),
            (PrivacyTelemetry, "Disabling Windows telemetry..."),
            (PrivacyTasks, "Disabling telemetry scheduled tasks..."),
            (PrivacyCompleted, "Privacy configuration completed"),
            // Startup
            (StartupTitle, "Startup Programs"),
            (StartupAnalyzing, "Analyzing startup programs..."),
            (StartupGettingList, "Getting startup programs list..."),
            (StartupFound, "Startup programs found:"),
            (
                StartupDisableHint,
                "To disable programs: Run 'msconfig' or 'Task Manager'",
            ),
            (
                StartupRecommendation,
                "Recommendation: Disable unnecessary programs to speed up startup",
            ),
            // Visual Effects
            (VisualEffectsTitle, "Visual Effects"),
            (VisualEffectsOptimizing, "Optimizing visual effects..."),
            (VisualEffectsApplying, "Applying performance settings..."),
            (VisualEffectsCompleted, "Visual effects optimized"),
            (
                VisualEffectsLogoutRequired,
                "Changes will apply after logging out or restarting",
            ),
            (
                VisualEffectsHint,
                "This can significantly improve performance on older systems",
            ),
            // Generic
            (Success, "Success"),
            (Warning, "Warning"),
            (Error, "Error"),
            (Info, "Information"),
        ];

        // Crear HashMap e insertar traducciones
        let mut translations = HashMap::new();

        // Insertar traducciones en español
        for (key, text) in es_translations {
            translations.insert((Spanish, key), text);
        }

        // Insertar traducciones en inglés
        for (key, text) in en_translations {
            translations.insert((English, key), text);
        }

        translations
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new(Language::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_code() {
        assert_eq!(Language::Spanish.code(), "es");
        assert_eq!(Language::English.code(), "en");
    }

    #[test]
    fn test_language_native_name() {
        assert_eq!(Language::Spanish.native_name(), "Español");
        assert_eq!(Language::English.native_name(), "English");
    }

    #[test]
    fn test_i18n_spanish() {
        let i18n = I18n::new(Language::Spanish);
        assert_eq!(i18n.t(I18nKey::AppTitle), "WIN OPT");
        assert_eq!(i18n.t(I18nKey::MenuTempFiles), "Archivos Temporales");
    }

    #[test]
    fn test_i18n_english() {
        let i18n = I18n::new(Language::English);
        assert_eq!(i18n.t(I18nKey::AppTitle), "WIN OPT");
        assert_eq!(i18n.t(I18nKey::MenuTempFiles), "Temporary Files");
    }

    #[test]
    fn test_toggle_language() {
        let mut i18n = I18n::new(Language::Spanish);
        assert_eq!(i18n.current_language(), Language::Spanish);

        i18n.toggle_language();
        assert_eq!(i18n.current_language(), Language::English);

        i18n.toggle_language();
        assert_eq!(i18n.current_language(), Language::Spanish);
    }
}
