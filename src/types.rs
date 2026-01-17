/// Vista actual de la aplicación
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    MainMenu,
    Clean,
    Network,
    Repair,
    Info,
    Optimize,
    WindowsUpdate,
    Privacy,
    BrowserCache,
    SystemLogs,
    RecycleBin,
    StartupOptimizer,
    VisualEffects,
}

/// Estado de ejecución de una operación
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationState {
    Idle,
    Starting,
    Running,
    Completed,
    Failed,
}

/// Estadísticas de limpieza
#[derive(Debug, Clone, Default)]
pub struct CleanStats {
    pub deleted_count: usize,
    pub failed_count: usize,
    pub size_freed: u64,
}

/// Mensajes enviados desde el worker thread al thread principal
#[derive(Debug)]
pub enum WorkerMessage {
    /// Log de una línea de texto
    Log(String),
    /// Cambio de estado de la operación
    StateChange(OperationState),
    /// Actualización de estadísticas de limpieza
    StatsUpdate(CleanStats),
    /// Error ocurrido durante la operación
    Error(String),
    /// Operación completada exitosamente
    Completed,
}

/// Handle para manejar un worker thread
pub struct WorkerHandle {
    /// Receptor de mensajes del worker
    pub receiver: std::sync::mpsc::Receiver<WorkerMessage>,
    /// Handle del thread (usado para join)
    pub thread_handle: Option<std::thread::JoinHandle<()>>,
    /// Flag atómico para cancelar la operación
    pub cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Drop for WorkerHandle {
    /// Asegura que el thread worker se una correctamente al ser descartado
    ///
    /// Esto previene fugas de recursos asegurando que el thread termine
    /// antes de que el handle sea destruido.
    fn drop(&mut self) {
        // Señalar cancelación al worker
        self.cancel_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);

        // Unir el thread si existe
        if let Some(handle) = self.thread_handle.take() {
            // Ignorar errores de join (el thread pudo haber entrado en pánico)
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_copy_and_equality() {
        let view1 = View::MainMenu;
        let view2 = view1;
        assert_eq!(view1, view2);
    }

    #[test]
    fn test_all_view_variants_unique() {
        // Verificar que todas las variantes son diferentes
        assert_ne!(View::MainMenu, View::Clean);
        assert_ne!(View::Clean, View::Network);
        assert_ne!(View::Network, View::Repair);
        assert_ne!(View::Repair, View::Info);
    }

    #[test]
    fn test_operation_state_transitions() {
        let idle = OperationState::Idle;
        let starting = OperationState::Starting;
        let running = OperationState::Running;
        let completed = OperationState::Completed;
        let failed = OperationState::Failed;

        assert_ne!(idle, starting);
        assert_ne!(starting, running);
        assert_ne!(running, completed);
        assert_ne!(completed, failed);
        assert_ne!(idle, completed);
    }

    #[test]
    fn test_clean_stats_default() {
        let stats = CleanStats::default();

        assert_eq!(stats.deleted_count, 0);
        assert_eq!(stats.failed_count, 0);
        assert_eq!(stats.size_freed, 0);
    }

    #[test]
    fn test_clean_stats_creation() {
        let stats = CleanStats {
            deleted_count: 42,
            failed_count: 3,
            size_freed: 1024 * 1024 * 50, // 50 MB
        };

        assert_eq!(stats.deleted_count, 42);
        assert_eq!(stats.failed_count, 3);
        assert_eq!(stats.size_freed, 52_428_800);
    }

    #[test]
    fn test_clean_stats_clone() {
        let stats1 = CleanStats {
            deleted_count: 10,
            failed_count: 2,
            size_freed: 1000,
        };

        let stats2 = stats1.clone();

        assert_eq!(stats1.deleted_count, stats2.deleted_count);
        assert_eq!(stats1.failed_count, stats2.failed_count);
        assert_eq!(stats1.size_freed, stats2.size_freed);
    }
}
