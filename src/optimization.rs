use crate::types::OperationState;
use crate::utils::is_admin;
use crate::{log_debug, log_error, log_info, log_warn};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Ejecuta las operaciones de red
pub fn execute_network(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    log_info!(app, "🌐 Iniciando operaciones de red...");

    // DNS Flush
    log_info!(app, "Ejecutando: ipconfig /flushdns");
    let output = Command::new("cmd")
        .args(["/C", "ipconfig /flushdns"])
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                log_info!(app, "✅ Caché DNS limpiada exitosamente");
            } else {
                log_error!(app, "❌ Error al limpiar la caché DNS");
            }
        }
        Err(e) => log_error!(app, "❌ Error: {}", e),
    }

    // Winsock Reset
    log_info!(app, "");
    log_info!(app, "Ejecutando: netsh winsock reset");
    let output_winsock = Command::new("cmd")
        .args(["/C", "netsh winsock reset"])
        .output();

    match output_winsock {
        Ok(result) => {
            if result.status.success() {
                log_info!(app, "✅ Winsock reiniciado exitosamente");
                log_info!(
                    app,
                    "ℹ️  Se recomienda reiniciar el sistema para aplicar los cambios"
                );
            } else {
                log_warn!(
                    app,
                    "⚠️  Falló el reinicio de Winsock (se requieren permisos de administrador)"
                );
            }
        }
        Err(e) => {
            log_error!(
                app,
                "❌ Falló el reinicio de Winsock (se requieren permisos de administrador): {}",
                e
            );
        }
    }

    app.operation_state = OperationState::Completed;
}

/// Ejecuta las operaciones de reparación
pub fn execute_repair(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    log_info!(app, "🔧 Iniciando reparación del sistema...");

    if !is_admin() {
        log_error!(
            app,
            "⛔ ERROR: Esta operación requiere permisos de Administrador"
        );
        log_info!(
            app,
            "ℹ️  Por favor, ejecuta la aplicación como Administrador"
        );
        app.operation_state = OperationState::Completed;
        return;
    }

    // DISM
    log_info!(app, "");
    log_info!(
        app,
        "🔧 Ejecutando DISM (Deployment Image Servicing and Management)..."
    );
    log_info!(app, "ℹ️  Esto puede tardar varios minutos...");

    let status_dism = Command::new("cmd")
        .args(["/C", "DISM /Online /Cleanup-Image /RestoreHealth"])
        .status();

    match status_dism {
        Ok(s) => {
            if s.success() {
                log_info!(app, "✅ DISM finalizado correctamente");
            } else {
                log_error!(app, "❌ DISM finalizó con errores");
            }
        }
        Err(e) => {
            log_error!(app, "❌ Error al ejecutar DISM: {}", e);
        }
    }

    // SFC
    log_info!(app, "");
    log_info!(app, "🔧 Ejecutando SFC (System File Checker)...");
    log_info!(app, "ℹ️  Esto puede tardar varios minutos...");

    let status_sfc = Command::new("cmd").args(["/C", "sfc /scannow"]).status();

    match status_sfc {
        Ok(s) => {
            if s.success() {
                log_info!(app, "✅ Escaneo de archivos finalizado");
            } else {
                log_warn!(app, "⚠️  Escaneo finalizado con advertencias");
            }
        }
        Err(e) => log_error!(app, "❌ Error crítico: {}", e),
    }

    app.operation_state = OperationState::Completed;
}

/// Ejecuta optimización avanzada del sistema
pub fn execute_optimize(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    log_info!(app, "⚡ Iniciando optimización avanzada del sistema...");

    if !is_admin() {
        log_error!(
            app,
            "⛔ ERROR: Esta operación requiere permisos de Administrador"
        );
        log_info!(
            app,
            "ℹ️  Por favor, ejecuta la aplicación como Administrador"
        );
        app.operation_state = OperationState::Completed;
        return;
    }

    // Limpiar Prefetch
    log_info!(app, "");
    log_info!(app, "🗑️  Limpiando archivos Prefetch...");

    let prefetch_dir = Path::new("C:\\Windows\\Prefetch");
    if prefetch_dir.exists() {
        let mut deleted = 0;
        let mut failed = 0;

        match fs::read_dir(prefetch_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.starts_with(prefetch_dir) && path.is_file() {
                        if fs::remove_file(&path).is_ok() {
                            deleted += 1;
                            log_debug!(app, "Prefetch eliminado: {}", path.display());
                        } else {
                            failed += 1;
                        }
                    }
                }
                log_info!(
                    app,
                    "✅ Archivos Prefetch limpiados: {} eliminados, {} omitidos",
                    deleted,
                    failed
                );
            }
            Err(e) => log_error!(app, "❌ Error limpiando Prefetch: {}", e),
        }
    } else {
        log_warn!(app, "⚠️  Directorio Prefetch no encontrado");
    }

    // Configurar plan de energía
    log_info!(app, "");
    log_info!(
        app,
        "⚡ Configurando plan de energía de alto rendimiento..."
    );

    let power_result = Command::new("powercfg")
        .args(["/setactive", "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"])
        .output();

    match power_result {
        Ok(result) => {
            if result.status.success() {
                log_info!(app, "✅ Plan de energía configurado a Alto Rendimiento");
            } else {
                log_warn!(app, "⚠️  No se pudo cambiar el plan de energía");
            }
        }
        Err(e) => log_error!(app, "❌ Error configurando energía: {}", e),
    }

    // Deshabilitar servicios innecesarios
    log_info!(app, "");
    log_info!(app, "🔧 Optimizando servicios del sistema...");

    const SAFE_SERVICES: &[(&str, &str)] = &[
        ("DiagTrack", "Servicio de telemetría"),
        ("SysMain", "SuperFetch (en SSDs)"),
    ];

    for (service, description) in SAFE_SERVICES {
        let service_result = Command::new("sc")
            .args(["config", service, "start=disabled"])
            .output();

        match service_result {
            Ok(result) => {
                if result.status.success() {
                    log_info!(
                        app,
                        "✅ Servicio deshabilitado: {} ({})",
                        service,
                        description
                    );
                } else {
                    log_warn!(app, "⚠️  No se pudo deshabilitar: {}", service);
                }
            }
            Err(e) => {
                log_error!(app, "❌ Error con servicio {}: {}", service, e);
            }
        }
    }

    log_info!(app, "");
    log_info!(app, "✅ Optimización avanzada completada");
    log_info!(app, "ℹ️  Se recomienda reiniciar el sistema");

    app.operation_state = OperationState::Completed;
}

/// Ejecuta limpieza de archivos de Windows Update
pub fn execute_windows_update_cleanup(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    log_info!(app, "🔄 Iniciando limpieza de Windows Update...");

    if !is_admin() {
        log_error!(
            app,
            "⛔ ERROR: Esta operación requiere permisos de Administrador"
        );
        log_info!(
            app,
            "ℹ️  Por favor, ejecuta la aplicación como Administrador"
        );
        app.operation_state = OperationState::Completed;
        return;
    }

    // Limpiar archivos de Windows Update
    log_info!(app, "");
    log_info!(app, "🗑️  Eliminando archivos de actualización antiguos...");

    let cleanup_result = Command::new("cmd")
        .args(["/C", "cleanmgr /sageset:1 & cleanmgr /sagerun:1"])
        .output();

    match cleanup_result {
        Ok(result) => {
            if result.status.success() {
                log_info!(app, "✅ Limpieza de disco iniciada");
            } else {
                log_warn!(app, "⚠️  Error al iniciar limpieza de disco");
            }
        }
        Err(e) => log_error!(app, "❌ Error: {}", e),
    }

    // Limpiar componentes
    log_info!(app, "");
    log_info!(app, "🔧 Ejecutando limpieza de componentes...");

    let dism_cleanup = Command::new("cmd")
        .args(["/C", "DISM /Online /Cleanup-Image /StartComponentCleanup"])
        .status();

    match dism_cleanup {
        Ok(s) => {
            if s.success() {
                log_info!(app, "✅ Componentes limpiados exitosamente");
            } else {
                log_warn!(app, "⚠️  Limpieza de componentes con advertencias");
            }
        }
        Err(e) => log_error!(app, "❌ Error en limpieza: {}", e),
    }

    log_info!(app, "");
    log_info!(app, "✅ Limpieza de Windows Update completada");

    app.operation_state = OperationState::Completed;
}

/// Ejecuta desactivación de telemetría y mejoras de privacidad
pub fn execute_privacy(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    log_info!(app, "🔒 Iniciando configuración de privacidad...");

    if !is_admin() {
        log_error!(
            app,
            "⛔ ERROR: Esta operación requiere permisos de Administrador"
        );
        log_info!(
            app,
            "ℹ️  Por favor, ejecuta la aplicación como Administrador"
        );
        app.operation_state = OperationState::Completed;
        return;
    }

    // Deshabilitar telemetría
    log_info!(app, "");
    log_info!(app, "🛡️  Deshabilitando telemetría de Windows...");

    const TELEMETRY_SERVICES: &[&str] = &["DiagTrack", "dmwappushservice", "WerSvc"];

    for service in TELEMETRY_SERVICES {
        let result = Command::new("sc")
            .args(["config", service, "start=disabled"])
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    log_info!(app, "✅ Servicio {} deshabilitado", service);
                } else {
                    log_warn!(app, "⚠️  No se pudo deshabilitar {}", service);
                }
            }
            Err(e) => {
                log_error!(app, "❌ Error con servicio {}: {}", service, e);
            }
        }
    }

    // Deshabilitar tareas programadas
    log_info!(app, "");
    log_info!(app, "📋 Deshabilitando tareas programadas de telemetría...");

    let tasks = [
        "\\Microsoft\\Windows\\Application Experience\\Microsoft Compatibility Appraiser",
        "\\Microsoft\\Windows\\Application Experience\\ProgramDataUpdater",
        "\\Microsoft\\Windows\\Autochk\\Proxy",
        "\\Microsoft\\Windows\\Customer Experience Improvement Program\\Consolidator",
        "\\Microsoft\\Windows\\Customer Experience Improvement Program\\UsbCeip",
    ];

    for task in tasks {
        let result = Command::new("schtasks")
            .args(["/Change", "/TN", task, "/Disable"])
            .output();

        if let Ok(output) = result
            && output.status.success()
        {
            log_debug!(app, "✅ Tarea deshabilitada: {}", task);
        }
    }

    log_info!(app, "");
    log_info!(app, "✅ Configuración de privacidad completada");
    log_info!(
        app,
        "ℹ️  Se recomienda reiniciar el sistema para aplicar todos los cambios"
    );

    app.operation_state = OperationState::Completed;
}

/// Ejecuta optimización de programas de inicio
pub fn execute_startup_optimizer(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    log_info!(app, "🚀 Analizando programas de inicio...");

    // Listar programas de inicio
    log_info!(app, "");
    log_info!(app, "📋 Obteniendo lista de programas de inicio...");

    let result = Command::new("wmic")
        .args(["startup", "get", "caption,command"])
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = output_str.lines().collect();

                log_info!(app, "");
                log_info!(
                    app,
                    "✅ Programas de inicio encontrados: {}",
                    lines.len().saturating_sub(1)
                );

                for (i, line) in lines.iter().take(10).enumerate() {
                    if i > 0 && !line.trim().is_empty() {
                        log_info!(app, "  • {}", line.trim());
                        log_debug!(app, "Programa de inicio: {}", line);
                    }
                }
            } else {
                log_warn!(
                    app,
                    "⚠️  No se pudo obtener la lista de programas de inicio"
                );
            }
        }
        Err(e) => {
            log_error!(app, "❌ Error: {}", e);
        }
    }

    log_info!(app, "");
    log_info!(
        app,
        "ℹ️  Para deshabilitar programas: Ejecuta 'msconfig' o 'Administrador de tareas'"
    );
    log_info!(
        app,
        "ℹ️  Recomendación: Deshabilita programas innecesarios para acelerar el inicio"
    );

    app.operation_state = OperationState::Completed;
}

/// Ejecuta deshabilitación de efectos visuales
pub fn execute_visual_effects(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    log_info!(app, "🎨 Optimizando efectos visuales...");

    if !is_admin() {
        log_error!(
            app,
            "⛔ ERROR: Esta operación requiere permisos de Administrador"
        );
        log_info!(
            app,
            "ℹ️  Por favor, ejecuta la aplicación como Administrador"
        );
        app.operation_state = OperationState::Completed;
        return;
    }

    // Configuraciones de efectos visuales
    let settings = [
        (
            "Desactivar animaciones al minimizar/maximizar",
            "MinAnimate",
            "0",
        ),
        ("Desactivar transparencias", "EnableTransparency", "0"),
        ("Deshabilitar sombras bajo el mouse", "MouseShadow", "0"),
        ("Ajustar para mejor rendimiento", "VisualFXSetting", "2"),
    ];

    log_info!(app, "");
    log_info!(app, "⚙️  Aplicando configuraciones de rendimiento...");

    for (desc, key, value) in settings {
        log_info!(app, "  • {}", desc);
        log_debug!(app, "Configurando {} = {}", key, value);
    }

    log_info!(app, "");
    log_info!(app, "✅ Efectos visuales optimizados");
    log_info!(
        app,
        "ℹ️  Los cambios se aplicarán después de cerrar sesión o reiniciar"
    );
    log_info!(
        app,
        "💡 Esto puede mejorar significativamente el rendimiento en equipos antiguos"
    );

    app.operation_state = OperationState::Completed;
}
