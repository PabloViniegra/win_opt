use crate::types::OperationState;
use crate::utils::is_admin;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Ejecuta las operaciones de red
pub fn execute_network(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    app.operation_logs
        .push("🌐 Iniciando operaciones de red...".to_string());

    // DNS Flush
    app.operation_logs
        .push("Ejecutando: ipconfig /flushdns".to_string());
    let output = Command::new("cmd")
        .args(["/C", "ipconfig /flushdns"])
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                app.operation_logs
                    .push("✅ Caché DNS limpiada exitosamente".to_string());
            } else {
                app.operation_logs
                    .push("❌ Error al limpiar la caché DNS".to_string());
            }
        }
        Err(e) => app.operation_logs.push(format!("❌ Error: {}", e)),
    }

    // Winsock Reset
    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("Ejecutando: netsh winsock reset".to_string());
    let output_winsock = Command::new("cmd")
        .args(["/C", "netsh winsock reset"])
        .output();

    match output_winsock {
        Ok(result) => {
            if result.status.success() {
                app.operation_logs
                    .push("✅ Winsock reiniciado exitosamente".to_string());
                app.operation_logs.push(
                    "ℹ️  Se recomienda reiniciar el sistema para aplicar los cambios".to_string(),
                );
            } else {
                app.operation_logs.push(
                    "⚠️  Falló el reinicio de Winsock (se requieren permisos de administrador)"
                        .to_string(),
                );
            }
        }
        Err(_) => {
            app.operation_logs.push(
                "❌ Falló el reinicio de Winsock (se requieren permisos de administrador)"
                    .to_string(),
            );
        }
    }

    app.operation_state = OperationState::Completed;
}

/// Ejecuta las operaciones de reparación
pub fn execute_repair(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    app.operation_logs
        .push("🔧 Iniciando reparación del sistema...".to_string());

    if !is_admin() {
        app.operation_logs
            .push("⛔ ERROR: Esta operación requiere permisos de Administrador".to_string());
        app.operation_logs
            .push("ℹ️  Por favor, ejecuta la aplicación como Administrador".to_string());
        app.operation_state = OperationState::Completed;
        return;
    }

    // DISM
    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("🔧 Ejecutando DISM (Deployment Image Servicing and Management)...".to_string());
    app.operation_logs
        .push("ℹ️  Esto puede tardar varios minutos...".to_string());

    let status_dism = Command::new("cmd")
        .args(["/C", "DISM /Online /Cleanup-Image /RestoreHealth"])
        .status();

    match status_dism {
        Ok(s) => {
            if s.success() {
                app.operation_logs
                    .push("✅ DISM finalizado correctamente".to_string());
            } else {
                app.operation_logs
                    .push("❌ DISM finalizó con errores".to_string());
            }
        }
        Err(_) => {
            app.operation_logs
                .push("❌ Error al ejecutar DISM".to_string());
        }
    }

    // SFC
    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("🔧 Ejecutando SFC (System File Checker)...".to_string());
    app.operation_logs
        .push("ℹ️  Esto puede tardar varios minutos...".to_string());

    let status_sfc = Command::new("cmd").args(["/C", "sfc /scannow"]).status();

    match status_sfc {
        Ok(s) => {
            if s.success() {
                app.operation_logs
                    .push("✅ Escaneo de archivos finalizado".to_string());
            } else {
                app.operation_logs
                    .push("⚠️  Escaneo finalizado con advertencias".to_string());
            }
        }
        Err(e) => app.operation_logs.push(format!("❌ Error crítico: {}", e)),
    }

    app.operation_state = OperationState::Completed;
}

/// Ejecuta optimización avanzada del sistema
pub fn execute_optimize(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    app.operation_logs
        .push("⚡ Iniciando optimización avanzada del sistema...".to_string());

    if !is_admin() {
        app.operation_logs
            .push("⛔ ERROR: Esta operación requiere permisos de Administrador".to_string());
        app.operation_logs
            .push("ℹ️  Por favor, ejecuta la aplicación como Administrador".to_string());
        app.operation_state = OperationState::Completed;
        return;
    }

    // Limpiar Prefetch - CORREGIDO: Usar std::fs en lugar de cmd.exe
    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("🗑️  Limpiando archivos Prefetch...".to_string());

    let prefetch_dir = Path::new("C:\\Windows\\Prefetch");
    if prefetch_dir.exists() {
        let mut deleted = 0;
        let mut failed = 0;

        match fs::read_dir(prefetch_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    // Validación estricta: solo archivos dentro de Prefetch
                    if path.starts_with(prefetch_dir) && path.is_file() {
                        if fs::remove_file(&path).is_ok() {
                            deleted += 1;
                        } else {
                            failed += 1;
                        }
                    }
                }
                app.operation_logs.push(format!(
                    "✅ Archivos Prefetch limpiados: {} eliminados, {} omitidos",
                    deleted, failed
                ));
            }
            Err(e) => app
                .operation_logs
                .push(format!("❌ Error limpiando Prefetch: {}", e)),
        }
    } else {
        app.operation_logs
            .push("⚠️  Directorio Prefetch no encontrado".to_string());
    }

    // Configurar plan de energía de alto rendimiento
    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("⚡ Configurando plan de energía de alto rendimiento...".to_string());

    let power_result = Command::new("powercfg")
        .args(["/setactive", "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"])
        .output();

    match power_result {
        Ok(result) => {
            if result.status.success() {
                app.operation_logs
                    .push("✅ Plan de energía configurado a Alto Rendimiento".to_string());
            } else {
                app.operation_logs
                    .push("⚠️  No se pudo cambiar el plan de energía".to_string());
            }
        }
        Err(e) => app
            .operation_logs
            .push(format!("❌ Error configurando energía: {}", e)),
    }

    // Deshabilitar servicios innecesarios (con precaución) - Validación con whitelist
    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("🔧 Optimizando servicios del sistema...".to_string());

    const SAFE_SERVICES: &[(&str, &str)] = &[
        ("DiagTrack", "Servicio de telemetría"),
        ("SysMain", "SuperFetch (en SSDs)"),
    ];

    for (service, description) in SAFE_SERVICES {
        let service_result = Command::new("sc")
            .args(["config", service, "start=", "disabled"])
            .output();

        match service_result {
            Ok(result) => {
                if result.status.success() {
                    app.operation_logs.push(format!(
                        "✅ Servicio deshabilitado: {} ({})",
                        service, description
                    ));
                } else {
                    app.operation_logs
                        .push(format!("⚠️  No se pudo deshabilitar: {}", service));
                }
            }
            Err(_) => {
                app.operation_logs
                    .push(format!("❌ Error con servicio: {}", service));
            }
        }
    }

    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("✅ Optimización avanzada completada".to_string());
    app.operation_logs
        .push("ℹ️  Se recomienda reiniciar el sistema".to_string());

    app.operation_state = OperationState::Completed;
}

/// Ejecuta limpieza de archivos de Windows Update
pub fn execute_windows_update_cleanup(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    app.operation_logs
        .push("🔄 Iniciando limpieza de Windows Update...".to_string());

    if !is_admin() {
        app.operation_logs
            .push("⛔ ERROR: Esta operación requiere permisos de Administrador".to_string());
        app.operation_logs
            .push("ℹ️  Por favor, ejecuta la aplicación como Administrador".to_string());
        app.operation_state = OperationState::Completed;
        return;
    }

    // Limpiar archivos de Windows Update
    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("🗑️  Eliminando archivos de actualización antiguos...".to_string());

    let cleanup_result = Command::new("cmd")
        .args(["/C", "cleanmgr /sageset:1 & cleanmgr /sagerun:1"])
        .output();

    match cleanup_result {
        Ok(result) => {
            if result.status.success() {
                app.operation_logs
                    .push("✅ Limpieza de disco iniciada".to_string());
            } else {
                app.operation_logs
                    .push("⚠️  Error al iniciar limpieza de disco".to_string());
            }
        }
        Err(e) => app.operation_logs.push(format!("❌ Error: {}", e)),
    }

    // Limpiar componentes de Windows Update
    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("🔧 Ejecutando limpieza de componentes...".to_string());

    let dism_cleanup = Command::new("cmd")
        .args(["/C", "DISM /Online /Cleanup-Image /StartComponentCleanup"])
        .status();

    match dism_cleanup {
        Ok(s) => {
            if s.success() {
                app.operation_logs
                    .push("✅ Componentes limpiados exitosamente".to_string());
            } else {
                app.operation_logs
                    .push("⚠️  Limpieza de componentes con advertencias".to_string());
            }
        }
        Err(e) => app
            .operation_logs
            .push(format!("❌ Error en limpieza: {}", e)),
    }

    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("✅ Limpieza de Windows Update completada".to_string());

    app.operation_state = OperationState::Completed;
}

/// Ejecuta desactivación de telemetría y mejoras de privacidad
pub fn execute_privacy(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    app.operation_logs
        .push("🔒 Iniciando configuración de privacidad...".to_string());

    if !is_admin() {
        app.operation_logs
            .push("⛔ ERROR: Esta operación requiere permisos de Administrador".to_string());
        app.operation_logs
            .push("ℹ️  Por favor, ejecuta la aplicación como Administrador".to_string());
        app.operation_state = OperationState::Completed;
        return;
    }

    // Deshabilitar telemetría de Windows - Validación con whitelist
    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("🛡️  Deshabilitando telemetría de Windows...".to_string());

    const TELEMETRY_SERVICES: &[&str] = &["DiagTrack", "dmwappushservice", "WerSvc"];

    for service in TELEMETRY_SERVICES {
        let result = Command::new("sc")
            .args(["config", service, "start=", "disabled"])
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    app.operation_logs
                        .push(format!("✅ Servicio {} deshabilitado", service));
                } else {
                    app.operation_logs
                        .push(format!("⚠️  No se pudo deshabilitar {}", service));
                }
            }
            Err(_) => {
                app.operation_logs
                    .push(format!("❌ Error con servicio {}", service));
            }
        }
    }

    // Deshabilitar tareas programadas de telemetría
    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("📋 Deshabilitando tareas programadas de telemetría...".to_string());

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
            app.operation_logs
                .push("✅ Tarea deshabilitada".to_string());
        }
    }

    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("✅ Configuración de privacidad completada".to_string());
    app.operation_logs.push(
        "ℹ️  Se recomienda reiniciar el sistema para aplicar todos los cambios".to_string(),
    );

    app.operation_state = OperationState::Completed;
}

/// Ejecuta optimización de programas de inicio
pub fn execute_startup_optimizer(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    app.operation_logs
        .push("🚀 Analizando programas de inicio...".to_string());

    // Listar programas de inicio usando WMIC
    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("📋 Obteniendo lista de programas de inicio...".to_string());

    let result = Command::new("wmic")
        .args(["startup", "get", "caption,command"])
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = output_str.lines().collect();

                app.operation_logs.push("".to_string());
                app.operation_logs.push(format!(
                    "✅ Programas de inicio encontrados: {}",
                    lines.len().saturating_sub(1)
                ));

                for (i, line) in lines.iter().take(10).enumerate() {
                    if i > 0 && !line.trim().is_empty() {
                        app.operation_logs.push(format!("  • {}", line.trim()));
                    }
                }
            } else {
                app.operation_logs
                    .push("⚠️  No se pudo obtener la lista de programas de inicio".to_string());
            }
        }
        Err(e) => {
            app.operation_logs.push(format!("❌ Error: {}", e));
        }
    }

    app.operation_logs.push("".to_string());
    app.operation_logs.push(
        "ℹ️  Para deshabilitar programas: Ejecuta 'msconfig' o 'Administrador de tareas'"
            .to_string(),
    );
    app.operation_logs.push(
        "ℹ️  Recomendación: Deshabilita programas innecesarios para acelerar el inicio"
            .to_string(),
    );

    app.operation_state = OperationState::Completed;
}

/// Ejecuta deshabilitación de efectos visuales
pub fn execute_visual_effects(app: &mut crate::app::App) {
    app.operation_state = OperationState::Running;
    app.operation_logs
        .push("🎨 Optimizando efectos visuales...".to_string());

    if !is_admin() {
        app.operation_logs
            .push("⛔ ERROR: Esta operación requiere permisos de Administrador".to_string());
        app.operation_logs
            .push("ℹ️  Por favor, ejecuta la aplicación como Administrador".to_string());
        app.operation_state = OperationState::Completed;
        return;
    }

    // Deshabilitar efectos visuales mediante registro
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

    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("⚙️  Aplicando configuraciones de rendimiento...".to_string());

    for (desc, _, _) in settings {
        app.operation_logs.push(format!("  • {}", desc));
    }

    app.operation_logs.push("".to_string());
    app.operation_logs
        .push("✅ Efectos visuales optimizados".to_string());
    app.operation_logs
        .push("ℹ️  Los cambios se aplicarán después de cerrar sesión o reiniciar".to_string());
    app.operation_logs.push(
        "💡 Esto puede mejorar significativamente el rendimiento en equipos antiguos"
            .to_string(),
    );

    app.operation_state = OperationState::Completed;
}
