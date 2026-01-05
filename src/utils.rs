use std::process::Command;

/// Helper para pluralización correcta en español
fn pluralize(count: u64, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("{} {}", count, singular)
    } else {
        format!("{} {}", count, plural)
    }
}

/// Formatea el tiempo de actividad del sistema
pub fn format_uptime(uptime: u64) -> String {
    let seconds = uptime;
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!(
            "{}, {}, {}",
            pluralize(days, "día", "días"),
            pluralize(hours, "hora", "horas"),
            pluralize(minutes, "minuto", "minutos")
        )
    } else if hours > 0 {
        format!(
            "{}, {}",
            pluralize(hours, "hora", "horas"),
            pluralize(minutes, "minuto", "minutos")
        )
    } else if minutes > 0 {
        pluralize(minutes, "minuto", "minutos")
    } else {
        pluralize(seconds, "segundo", "segundos")
    }
}

/// Verifica si el proceso actual tiene permisos de administrador
pub fn is_admin() -> bool {
    Command::new("net")
        .args(["session"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime_seconds() {
        assert_eq!(format_uptime(0), "0 segundos");
        assert_eq!(format_uptime(1), "1 segundo");
        assert_eq!(format_uptime(30), "30 segundos");
        assert_eq!(format_uptime(59), "59 segundos");
    }

    #[test]
    fn test_format_uptime_minutes() {
        assert_eq!(format_uptime(60), "1 minuto");
        assert_eq!(format_uptime(120), "2 minutos");
        assert_eq!(format_uptime(3540), "59 minutos");
    }

    #[test]
    fn test_format_uptime_hours() {
        assert_eq!(format_uptime(3600), "1 hora, 0 minutos");
        assert_eq!(format_uptime(3661), "1 hora, 1 minuto");
        assert_eq!(format_uptime(7200), "2 horas, 0 minutos");
    }

    #[test]
    fn test_format_uptime_days() {
        assert_eq!(format_uptime(86400), "1 día, 0 horas, 0 minutos");
        assert_eq!(format_uptime(90061), "1 día, 1 hora, 1 minuto");
        assert_eq!(format_uptime(172800), "2 días, 0 horas, 0 minutos");
    }

    #[test]
    fn test_format_uptime_large_values() {
        // 7 días
        assert_eq!(format_uptime(604800), "7 días, 0 horas, 0 minutos");
        // 30 días
        assert_eq!(format_uptime(2592000), "30 días, 0 horas, 0 minutos");
    }

    #[test]
    fn test_is_admin_returns_bool() {
        // Solo verificar que no panic y retorna un booleano
        let result = is_admin();
        assert!(result == true || result == false);
    }

    #[test]
    fn test_is_admin_consistency() {
        // Verificar que múltiples llamadas retornan el mismo resultado
        let result1 = is_admin();
        let result2 = is_admin();
        assert_eq!(result1, result2);
    }
}
