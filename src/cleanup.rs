use crate::types::{CleanStats, OperationState};
use std::fs;
use std::process::Command;

/// Ejecuta la operación de limpieza de archivos temporales
pub fn execute_clean(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    app.operation_logs
        .push("🧹 Iniciando limpieza de archivos temporales...".to_string());

    let temp_dir = std::env::temp_dir();
    app.operation_logs
        .push(format!("📁 Directorio: {}", temp_dir.to_string_lossy()));

    let mut deleted_count = 0;
    let mut size_freed: u64 = 0;
    let mut failed_count = 0;

    match fs::read_dir(&temp_dir) {
        Ok(entries) => {
            let entries_vec: Vec<_> = entries.flatten().collect();
            let total = entries_vec.len();

            app.operation_logs
                .push(format!("📊 Elementos encontrados: {}", total));

            for (idx, entry) in entries_vec.iter().enumerate() {
                let path = entry.path();

                if path.is_file() {
                    if let Ok(metadata) = fs::metadata(&path) {
                        size_freed += metadata.len();
                    }
                    if fs::remove_file(&path).is_ok() {
                        deleted_count += 1;
                    } else {
                        failed_count += 1;
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
                    } else {
                        failed_count += 1;
                    }
                }

                if idx % 10 == 0 {
                    app.operation_logs
                        .push(format!("Procesando... {}/{}", idx + 1, total));
                }
            }

            app.clean_stats = CleanStats {
                deleted_count,
                failed_count,
                size_freed,
            };

            app.operation_logs
                .push("✅ Limpieza completada".to_string());
        }
        Err(_) => {
            app.operation_logs
                .push("❌ Error al leer el directorio temporal".to_string());
        }
    }

    app.operation_state = OperationState::Completed;
}

/// Ejecuta limpieza de caché de navegadores
pub fn execute_browser_cache(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    app.operation_logs
        .push("🌐 Iniciando limpieza de caché de navegadores...".to_string());

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
        app.operation_logs.push("".to_string());
        app.operation_logs
            .push(format!("🗑️  Limpiando caché de {}...", browser_name));

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
                } else {
                    total_failed += 1;
                }
            }
            app.operation_logs
                .push(format!("✅ {} - Caché limpiada", browser_name));
        } else {
            app.operation_logs.push(format!(
                "⚠️  {} - No encontrado o inaccesible",
                browser_name
            ));
        }
    }

    app.operation_logs.push("".to_string());
    app.operation_logs
        .push(format!("✅ Archivos eliminados: {}", total_cleaned));
    app.operation_logs
        .push(format!("⚠️  Archivos omitidos: {}", total_failed));
    app.operation_logs.push(
        "ℹ️  Cierra los navegadores antes de ejecutar esta operación para mejores resultados"
            .to_string(),
    );

    app.operation_state = OperationState::Completed;
}

/// Ejecuta limpieza de logs del sistema
pub fn execute_system_logs(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    app.operation_logs
        .push("📋 Iniciando limpieza de logs del sistema...".to_string());

    let log_paths = [
        "C:\\Windows\\Logs",
        "C:\\Windows\\Temp",
        "C:\\Windows\\Prefetch",
    ];

    let mut total_deleted = 0;
    let mut total_failed = 0;

    for log_path in log_paths {
        app.operation_logs.push("".to_string());
        app.operation_logs
            .push(format!("🗑️  Limpiando: {}...", log_path));

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
                    } else {
                        total_failed += 1;
                    }
                }
            }
            app.operation_logs
                .push(format!("✅ {} procesado", log_path));
        } else {
            app.operation_logs.push(format!(
                "⚠️  {} - Requiere permisos de administrador",
                log_path
            ));
        }
    }

    app.operation_logs.push("".to_string());
    app.operation_logs
        .push(format!("✅ Archivos eliminados: {}", total_deleted));
    app.operation_logs
        .push(format!("⚠️  Archivos omitidos: {}", total_failed));

    app.operation_state = OperationState::Completed;
}

/// Ejecuta vaciado de papelera de reciclaje
pub fn execute_recycle_bin(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    app.operation_logs
        .push("🗑️  Iniciando vaciado de papelera de reciclaje...".to_string());

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
                app.operation_logs
                    .push("✅ Papelera de reciclaje vaciada exitosamente".to_string());
            } else {
                app.operation_logs
                    .push("⚠️  Advertencia: Algunas carpetas no pudieron vaciarse".to_string());
            }
        }
        Err(e) => {
            app.operation_logs
                .push(format!("❌ Error al vaciar papelera: {}", e));
        }
    }

    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("ℹ️  Espacio en disco liberado".to_string());

    app.operation_state = OperationState::Completed;
}
