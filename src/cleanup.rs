use crate::types::{CleanStats, OperationState};
use crate::{log_debug, log_error, log_info, log_warn};
use std::fs;
use std::process::Command;

/// Ejecuta la operación de limpieza de archivos temporales
pub fn execute_clean(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    log_info!(app, "🧹 Iniciando limpieza de archivos temporales...");

    let temp_dir = std::env::temp_dir();
    log_info!(app, "📁 Directorio: {}", temp_dir.to_string_lossy());

    let mut deleted_count = 0;
    let mut size_freed: u64 = 0;
    let mut failed_count = 0;

    match fs::read_dir(&temp_dir) {
        Ok(entries) => {
            let entries_vec: Vec<_> = entries.flatten().collect();
            let total = entries_vec.len();

            log_info!(app, "📊 Elementos encontrados: {}", total);

            for (idx, entry) in entries_vec.iter().enumerate() {
                let path = entry.path();

                if path.is_file() {
                    if let Ok(metadata) = fs::metadata(&path) {
                        size_freed += metadata.len();
                    }
                    if fs::remove_file(&path).is_ok() {
                        deleted_count += 1;
                        log_debug!(app, "Archivo eliminado: {}", path.display());
                    } else {
                        failed_count += 1;
                        log_warn!(app, "No se pudo eliminar archivo: {}", path.display());
                    }
                } else if path.is_dir() {
                    if let Ok(entries) = fs::read_dir(&path) {
                        for entry in entries.flatten() {
                            if let Ok(meta) = entry.metadata() {
                                size_freed += meta.len();
                            }
                        }
                    }
                    if fs::remove_dir_all(&path).is_ok() {
                        deleted_count += 1;
                        log_debug!(app, "Directorio eliminado: {}", path.display());
                    } else {
                        failed_count += 1;
                        log_warn!(app, "No se pudo eliminar directorio: {}", path.display());
                    }
                }

                if idx % 10 == 0 {
                    log_debug!(app, "Procesando... {}/{}", idx + 1, total);
                }
            }

            app.clean_stats = CleanStats {
                deleted_count,
                failed_count,
                size_freed,
            };

            log_info!(
                app,
                "✅ Limpieza completada - Eliminados: {}, Omitidos: {}, Espacio: {} bytes",
                deleted_count,
                failed_count,
                size_freed
            );
        }
        Err(e) => {
            log_error!(app, "❌ Error al leer el directorio temporal: {}", e);
        }
    }

    app.operation_state = OperationState::Completed;
}

/// Ejecuta limpieza de caché de navegadores
pub fn execute_browser_cache(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    log_info!(app, "🌐 Iniciando limpieza de caché de navegadores...");

    let user_profile =
        std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());

    // Rutas de caché de navegadores
    let cache_paths = [
        (
            "Google Chrome",
            format!(
                "{}\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\Cache",
                user_profile
            ),
        ),
        (
            "Microsoft Edge",
            format!(
                "{}\\AppData\\Local\\Microsoft\\Edge\\User Data\\Default\\Cache",
                user_profile
            ),
        ),
        (
            "Mozilla Firefox",
            format!(
                "{}\\AppData\\Local\\Mozilla\\Firefox\\Profiles",
                user_profile
            ),
        ),
    ];

    let mut total_cleaned = 0;
    let mut total_failed = 0;

    for (browser_name, cache_path) in cache_paths {
        log_info!(app, "");
        log_info!(app, "🗑️  Limpiando caché de {}...", browser_name);

        if let Ok(entries) = fs::read_dir(&cache_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let result = if path.is_dir() {
                    fs::remove_dir_all(&path)
                } else {
                    fs::remove_file(&path)
                };

                if result.is_ok() {
                    total_cleaned += 1;
                    log_debug!(app, "Eliminado: {}", path.display());
                } else {
                    total_failed += 1;
                    log_debug!(app, "Omitido: {}", path.display());
                }
            }
            log_info!(app, "✅ {} - Caché limpiada", browser_name);
        } else {
            log_warn!(app, "⚠️  {} - No encontrado o inaccesible", browser_name);
        }
    }

    log_info!(app, "");
    log_info!(app, "✅ Archivos eliminados: {}", total_cleaned);
    log_info!(app, "⚠️  Archivos omitidos: {}", total_failed);
    log_info!(
        app,
        "ℹ️  Cierra los navegadores antes de ejecutar esta operación para mejores resultados"
    );

    app.operation_state = OperationState::Completed;
}

/// Ejecuta limpieza de logs del sistema
pub fn execute_system_logs(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    log_info!(app, "📋 Iniciando limpieza de logs del sistema...");

    let log_paths = [
        "C:\\Windows\\Logs",
        "C:\\Windows\\Temp",
        "C:\\Windows\\Prefetch",
    ];

    let mut total_deleted = 0;
    let mut total_failed = 0;

    for log_path in log_paths {
        log_info!(app, "");
        log_info!(app, "🗑️  Limpiando: {}...", log_path);

        if let Ok(entries) = fs::read_dir(log_path) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Solo eliminar archivos .log, .txt y .etl
                if let Some(ext) = path.extension()
                    && (ext == "log" || ext == "txt" || ext == "etl" || ext == "tmp")
                {
                    let result = if path.is_dir() {
                        fs::remove_dir_all(&path)
                    } else {
                        fs::remove_file(&path)
                    };

                    if result.is_ok() {
                        total_deleted += 1;
                        log_debug!(app, "Eliminado: {}", path.display());
                    } else {
                        total_failed += 1;
                        log_debug!(app, "Omitido: {}", path.display());
                    }
                }
            }
            log_info!(app, "✅ {} procesado", log_path);
        } else {
            log_warn!(app, "⚠️  {} - Requiere permisos de administrador", log_path);
        }
    }

    log_info!(app, "");
    log_info!(app, "✅ Archivos eliminados: {}", total_deleted);
    log_info!(app, "⚠️  Archivos omitidos: {}", total_failed);

    app.operation_state = OperationState::Completed;
}

/// Ejecuta vaciado de papelera de reciclaje
pub fn execute_recycle_bin(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    log_info!(app, "🗑️  Iniciando vaciado de papelera de reciclaje...");

    // Vaciar papelera usando PowerShell
    let result = Command::new("powershell")
        .args([
            "-Command",
            "Clear-RecycleBin -Force -ErrorAction SilentlyContinue",
        ])
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                log_info!(app, "✅ Papelera de reciclaje vaciada exitosamente");
            } else {
                log_warn!(
                    app,
                    "⚠️  Advertencia: Algunas carpetas no pudieron vaciarse"
                );
                log_debug!(
                    app,
                    "Salida del comando: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            log_error!(app, "❌ Error al vaciar papelera: {}", e);
        }
    }

    log_info!(app, "");
    log_info!(app, "ℹ️  Espacio en disco liberado");

    app.operation_state = OperationState::Completed;
}
