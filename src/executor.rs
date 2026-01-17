/// Sistema de workers para ejecutar comandos en threads separados
///
/// Este módulo proporciona funcionalidad para ejecutar comandos de Windows
/// en threads separados, manteniendo la UI responsiva y evitando que la
/// salida de los comandos corrompa la interfaz TUI.
use crate::types::{OperationState, WorkerHandle, WorkerMessage};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::thread;

/// Envía un mensaje de log al thread principal
///
/// # Returns
/// `true` si el mensaje fue enviado exitosamente, `false` si el receptor fue descartado
fn send_log(sender: &Sender<WorkerMessage>, message: String) -> bool {
    sender.send(WorkerMessage::Log(message)).is_ok()
}

/// Envía un cambio de estado al thread principal
///
/// # Returns
/// `true` si el mensaje fue enviado exitosamente, `false` si el receptor fue descartado
fn send_state(sender: &Sender<WorkerMessage>, state: OperationState) -> bool {
    sender.send(WorkerMessage::StateChange(state)).is_ok()
}

/// Envía un mensaje de error al thread principal
///
/// # Returns
/// `true` si el mensaje fue enviado exitosamente, `false` si el receptor fue descartado
fn send_error(sender: &Sender<WorkerMessage>, error: String) -> bool {
    sender.send(WorkerMessage::Error(error)).is_ok()
}

/// Ejecuta un comando y captura su salida sin mostrarla en pantalla
///
/// # Argumentos
/// * `sender` - Canal para enviar logs al thread principal
/// * `command` - Comando a ejecutar (ej: "DISM", "sfc")
/// * `args` - Argumentos del comando
///
/// # Returns
/// `true` si el comando se ejecutó exitosamente, `false` en caso contrario o si el canal está cerrado
fn execute_command(sender: &Sender<WorkerMessage>, command: &str, args: &[&str]) -> bool {
    if !send_log(
        sender,
        format!("Ejecutando: {} {}", command, args.join(" ")),
    ) {
        // Canal cerrado, terminar operación
        return false;
    }

    match Command::new(command).args(args).output() {
        Ok(output) => {
            // Convertir salida a UTF-8 (con reemplazo de caracteres inválidos)
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Enviar líneas de stdout como logs
            for line in stdout.lines() {
                if !line.trim().is_empty() && !send_log(sender, line.to_string()) {
                    // Canal cerrado, terminar operación
                    return false;
                }
            }

            // Enviar líneas de stderr como logs
            for line in stderr.lines() {
                if !line.trim().is_empty() && !send_log(sender, format!("ERROR: {}", line)) {
                    // Canal cerrado, terminar operación
                    return false;
                }
            }

            if output.status.success() {
                send_log(sender, "✓ Comando completado exitosamente".to_string());
                true
            } else {
                send_log(
                    sender,
                    format!("✗ Comando falló con código: {:?}", output.status.code()),
                );
                false
            }
        }
        Err(e) => {
            send_error(sender, format!("Error al ejecutar comando: {}", e));
            false
        }
    }
}

/// Spawn worker para operaciones de reparación del sistema (DISM + SFC)
///
/// Ejecuta DISM y SFC en secuencia, capturando toda la salida sin mostrarla
/// en la terminal, evitando corrupción visual de la TUI.
///
/// La operación puede ser cancelada en cualquier momento estableciendo el flag
/// de cancelación del `WorkerHandle` retornado.
///
/// # Returns
/// Un `WorkerHandle` que contiene:
/// - Un receptor de canal para mensajes de progreso
/// - Un handle del thread para join
/// - Un flag de cancelación atómico
///
/// # Example
/// ```no_run
/// use win_opt::executor::spawn_repair_worker;
///
/// let handle = spawn_repair_worker();
/// // Procesar mensajes del worker...
/// while let Ok(msg) = handle.receiver.recv() {
///     // Manejar mensaje...
/// }
/// ```
pub fn spawn_repair_worker() -> WorkerHandle {
    let (sender, receiver) = mpsc::channel();
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let cancel_flag_clone = cancel_flag.clone();

    let thread_handle = thread::spawn(move || {
        if !send_state(&sender, OperationState::Running) {
            return; // Canal cerrado
        }

        if !send_log(
            &sender,
            "=== Iniciando Reparación del Sistema ===".to_string(),
        ) {
            return; // Canal cerrado
        }

        // Verificar cancelación antes de DISM
        if cancel_flag_clone.load(Ordering::Relaxed) {
            send_log(&sender, "Operación cancelada por el usuario".to_string());
            send_state(&sender, OperationState::Failed);
            let _ = sender.send(WorkerMessage::Completed);
            return;
        }

        // Ejecutar DISM
        send_log(&sender, "Paso 1/2: Ejecutando DISM...".to_string());
        send_log(
            &sender,
            "Esto puede tomar entre 5-30 minutos dependiendo del sistema.".to_string(),
        );

        let dism_success = execute_command(
            &sender,
            "cmd",
            &["/C", "DISM /Online /Cleanup-Image /RestoreHealth"],
        );

        if !dism_success {
            send_error(
                &sender,
                "DISM falló. Continuando con SFC de todas formas...".to_string(),
            );
        }

        // Verificar cancelación antes de SFC
        if cancel_flag_clone.load(Ordering::Relaxed) {
            send_log(&sender, "Operación cancelada por el usuario".to_string());
            send_state(&sender, OperationState::Failed);
            let _ = sender.send(WorkerMessage::Completed);
            return;
        }

        // Ejecutar SFC
        send_log(&sender, "Paso 2/2: Ejecutando SFC...".to_string());
        send_log(
            &sender,
            "Verificando integridad de archivos del sistema...".to_string(),
        );

        let sfc_success = execute_command(&sender, "cmd", &["/C", "sfc /scannow"]);

        // Determinar resultado final
        if dism_success && sfc_success {
            send_log(
                &sender,
                "=== Reparación completada exitosamente ===".to_string(),
            );
            send_state(&sender, OperationState::Completed);
        } else {
            send_error(&sender, "Reparación completada con errores".to_string());
            send_state(&sender, OperationState::Failed);
        }

        let _ = sender.send(WorkerMessage::Completed);
    });

    WorkerHandle {
        receiver,
        thread_handle: Some(thread_handle),
        cancel_flag,
    }
}

/// Spawn worker para limpieza de Windows Update
///
/// Ejecuta DISM para limpiar archivos obsoletos de Windows Update en un thread separado.
///
/// La operación puede ser cancelada en cualquier momento estableciendo el flag
/// de cancelación del `WorkerHandle` retornado.
///
/// # Returns
/// Un `WorkerHandle` que contiene:
/// - Un receptor de canal para mensajes de progreso
/// - Un handle del thread para join
/// - Un flag de cancelación atómico
///
/// # Platform
/// Windows-only. Requiere privilegios de administrador.
///
/// # Example
/// ```no_run
/// use win_opt::executor::spawn_windows_update_worker;
///
/// let handle = spawn_windows_update_worker();
/// while let Ok(msg) = handle.receiver.recv() {
///     // Procesar mensaje...
/// }
/// ```
pub fn spawn_windows_update_worker() -> WorkerHandle {
    let (sender, receiver) = mpsc::channel();
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let cancel_flag_clone = cancel_flag.clone();

    let thread_handle = thread::spawn(move || {
        if !send_state(&sender, OperationState::Running) {
            return; // Canal cerrado
        }

        if !send_log(
            &sender,
            "=== Iniciando Limpieza de Windows Update ===".to_string(),
        ) {
            return; // Canal cerrado
        }

        // Verificar cancelación antes de ejecutar
        if cancel_flag_clone.load(Ordering::Relaxed) {
            send_log(&sender, "Operación cancelada por el usuario".to_string());
            send_state(&sender, OperationState::Failed);
            let _ = sender.send(WorkerMessage::Completed);
            return;
        }

        send_log(&sender, "Ejecutando DISM para limpiar caché...".to_string());
        send_log(
            &sender,
            "Esta operación puede tardar varios minutos...".to_string(),
        );

        let success = execute_command(
            &sender,
            "cmd",
            &[
                "/C",
                "DISM /Online /Cleanup-Image /StartComponentCleanup /ResetBase",
            ],
        );

        if success {
            send_log(
                &sender,
                "=== Limpieza completada exitosamente ===".to_string(),
            );
            send_state(&sender, OperationState::Completed);
        } else {
            send_error(&sender, "Limpieza falló".to_string());
            send_state(&sender, OperationState::Failed);
        }

        let _ = sender.send(WorkerMessage::Completed);
    });

    WorkerHandle {
        receiver,
        thread_handle: Some(thread_handle),
        cancel_flag,
    }
}

/// Spawn worker genérico para ejecutar un comando único
///
/// Útil para operaciones simples que requieren ejecutarse en background.
/// La operación puede ser cancelada estableciendo el flag de cancelación.
///
/// # Arguments
/// * `command` - Comando a ejecutar
/// * `args` - Argumentos del comando
/// * `description` - Descripción de la operación para logs
///
/// # Returns
/// Un `WorkerHandle` que contiene el receptor del canal, handle del thread y flag de cancelación
///
/// # Example
/// ```no_run
/// use win_opt::executor::spawn_command_worker;
///
/// let handle = spawn_command_worker(
///     "cmd".to_string(),
///     vec!["/C".to_string(), "dir".to_string()],
///     "Listar directorio".to_string(),
/// );
/// ```
pub fn spawn_command_worker(
    command: String,
    args: Vec<String>,
    description: String,
) -> WorkerHandle {
    let (sender, receiver) = mpsc::channel();
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let cancel_flag_clone = cancel_flag.clone();

    let thread_handle = thread::spawn(move || {
        if !send_state(&sender, OperationState::Running) {
            return; // Canal cerrado
        }

        if !send_log(&sender, format!("=== {} ===", description)) {
            return; // Canal cerrado
        }

        // Verificar cancelación antes de ejecutar
        if cancel_flag_clone.load(Ordering::Relaxed) {
            send_log(&sender, "Operación cancelada por el usuario".to_string());
            send_state(&sender, OperationState::Failed);
            let _ = sender.send(WorkerMessage::Completed);
            return;
        }

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let success = execute_command(&sender, &command, &args_str);

        if success {
            send_log(&sender, format!("=== {} completado ===", description));
            send_state(&sender, OperationState::Completed);
        } else {
            send_error(&sender, format!("{} falló", description));
            send_state(&sender, OperationState::Failed);
        }

        let _ = sender.send(WorkerMessage::Completed);
    });

    WorkerHandle {
        receiver,
        thread_handle: Some(thread_handle),
        cancel_flag,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_functions_dont_panic() {
        let (sender, receiver) = mpsc::channel();

        send_log(&sender, "Test log".to_string());
        send_state(&sender, OperationState::Running);
        send_error(&sender, "Test error".to_string());

        // Verificar que los mensajes fueron enviados
        let mut count = 0;
        while receiver.try_recv().is_ok() {
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn test_worker_handle_creation() {
        let (sender, receiver) = mpsc::channel();
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let handle = WorkerHandle {
            receiver,
            thread_handle: None,
            cancel_flag,
        };

        // Verificar que el handle se puede crear sin problemas
        assert!(handle.thread_handle.is_none());

        // Enviar un mensaje y verificar que se puede recibir
        sender.send(WorkerMessage::Log("test".to_string())).unwrap();
        assert!(handle.receiver.try_recv().is_ok());
    }

    #[test]
    fn test_worker_handle_cancellation() {
        let (_, receiver) = mpsc::channel();
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let handle = WorkerHandle {
            receiver,
            thread_handle: None,
            cancel_flag: cancel_flag.clone(),
        };

        // Verificar que el flag de cancelación se puede establecer
        assert!(!cancel_flag.load(Ordering::Relaxed));
        handle.cancel_flag.store(true, Ordering::Relaxed);
        assert!(cancel_flag.load(Ordering::Relaxed));
    }
}
