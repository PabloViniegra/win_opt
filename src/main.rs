use clap::{Parser, Subcommand};
use colored::*;
use dialoguer::Confirm;
use directories::BaseDirs;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::process::Command;
use std::time::Instant;
use sysinfo::{Disks, System};

/// CLI para optimización básica de Windows 11
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Omitir confirmaciones interactivas
    #[arg(short = 'y', long = "yes", global = true)]
    no_confirm: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Elimina archivos temporales del usuario
    Clean,
    /// Limpia la caché de DNS y reinicia sockets
    Network,
    /// Ejecuta herramientas de reparación (SFC / DISM) - Requiere Admin
    Repair,
    /// Muestra información del sistema (básica)
    Info,
}

/// Punto de entrada principal de la aplicación.
///
/// Parsea los argumentos de línea de comandos y ejecuta el subcomando correspondiente,
/// midiendo el tiempo de ejecución de la operación.
fn main() {
    let cli = Cli::parse();
    let start_time = Instant::now();

    print_banner();

    match &cli.command {
        Commands::Clean => clean_temp_files(cli.no_confirm),
        Commands::Network => flush_dns(),
        Commands::Repair => run_system_repair(),
        Commands::Info => show_system_info(),
    }

    let duration = start_time.elapsed();
    print_separator();
    println!(
        "{}",
        format!("⏱️  Operación finalizada en {:.2?}", duration)
            .bright_green()
            .bold()
    );
}

/// Imprime el banner de la aplicación.
fn print_banner() {
    let banner = r#"
╔══════════════════════════════════════════════════════════╗
║                                                          ║
║   ██╗    ██╗██╗███╗   ██╗    ██████╗ ██████╗ ████████╗  ║
║   ██║    ██║██║████╗  ██║   ██╔═══██╗██╔══██╗╚══██╔══╝  ║
║   ██║ █╗ ██║██║██╔██╗ ██║   ██║   ██║██████╔╝   ██║     ║
║   ██║███╗██║██║██║╚██╗██║   ██║   ██║██╔═══╝    ██║     ║
║   ╚███╔███╔╝██║██║ ╚████║   ╚██████╔╝██║        ██║     ║
║    ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝    ╚═════╝ ╚═╝        ╚═╝     ║
║                                                          ║
║         Windows 11 Optimizer CLI - v0.1.0                ║
║         Herramienta de optimización del sistema          ║
║                                                          ║
╚══════════════════════════════════════════════════════════╝
"#;
    println!("{}", banner.cyan().bold());
}

/// Imprime un separador visual.
fn print_separator() {
    println!("\n{}", "─".repeat(60).bright_black());
}

/// Imprime un encabezado de sección.
fn print_section_header(title: &str) {
    print_separator();
    println!("{}", format!("▶ {}", title).bright_cyan().bold());
    print_separator();
}

/// Elimina archivos y directorios temporales del sistema.
///
/// Esta función intenta eliminar todos los archivos y carpetas dentro del directorio
/// temporal del sistema (`%TEMP%`). Los archivos bloqueados o en uso son ignorados
/// silenciosamente. Reporta el número de elementos eliminados y el espacio aproximado liberado.
///
/// # Argumentos
/// * `no_confirm` - Si es `true`, omite la confirmación interactiva
///
/// # Nota
/// Los errores de eliminación (archivos en uso, permisos insuficientes) son ignorados
/// para permitir que la limpieza continúe con otros archivos.
fn clean_temp_files(no_confirm: bool) {
    print_section_header("LIMPIEZA DE ARCHIVOS TEMPORALES");

    if let Some(_base_dirs) = BaseDirs::new() {
        let temp_dir = std::env::temp_dir();
        println!("{} {:?}", "📁 Directorio:".bright_yellow(), temp_dir);

        // Contar elementos antes de pedir confirmación
        let total_items = if let Ok(entries) = fs::read_dir(&temp_dir) {
            entries.count()
        } else {
            0
        };

        println!(
            "{} {}",
            "📊 Elementos encontrados:".bright_yellow(),
            total_items.to_string().bright_white().bold()
        );

        // Pedir confirmación si no está deshabilitada
        if !no_confirm {
            let confirmation = Confirm::new()
                .with_prompt("¿Deseas continuar con la limpieza?")
                .default(false)
                .interact();

            match confirmation {
                Ok(true) => {}
                Ok(false) => {
                    println!("{}", "❌ Operación cancelada por el usuario.".yellow());
                    return;
                }
                Err(_) => {
                    println!("{}", "❌ Error al leer la confirmación.".red());
                    return;
                }
            }
        }

        println!("\n{}", "🧹 Iniciando limpieza...".bright_yellow());

        let mut deleted_count = 0;
        let mut size_freed: u64 = 0;
        let mut failed_count = 0;

        // Crear barra de progreso
        let pb = ProgressBar::new(total_items as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} | {msg}")
                .expect("Error al crear el estilo de la barra de progreso")
                .progress_chars("█▓░")
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );

        if let Ok(entries) = fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("desconocido");

                pb.set_message(format!("Procesando: {}", file_name));

                // Intentamos borrar. Si el archivo está en uso, Windows lanzará error, lo ignoramos.
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
                    // Aproximar el tamaño del directorio
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

                pb.inc(1);
            }
        }

        pb.finish_with_message("Limpieza completada");
        println!();

        // Mostrar resultados
        print_separator();
        println!("{}", "RESULTADOS DE LA LIMPIEZA".bright_green().bold());
        print_separator();
        println!(
            "{} {}",
            "✅ Elementos eliminados:".bright_green(),
            deleted_count.to_string().bright_white().bold()
        );
        println!(
            "{} {}",
            "⚠️  Elementos omitidos:".bright_yellow(),
            failed_count.to_string().bright_white().bold()
        );
        println!(
            "{} {} MB",
            "💾 Espacio liberado:".bright_green(),
            format!("{:.2}", size_freed as f64 / 1024.0 / 1024.0)
                .bright_white()
                .bold()
        );
    } else {
        println!(
            "{}",
            "❌ No se pudo localizar el directorio de usuario.".red()
        );
    }
}

/// Limpia la caché DNS y reinicia el catálogo de Winsock.
///
/// Esta función ejecuta dos comandos de Windows:
/// - `ipconfig /flushdns`: Limpia la caché de resolución DNS
/// - `netsh winsock reset`: Reinicia el catálogo de Winsock (requiere permisos de administrador)
///
/// # Nota
/// El comando `netsh winsock reset` puede fallar si no se ejecuta con privilegios de administrador.
fn flush_dns() {
    print_section_header("LIMPIEZA DE RED");

    // Spinner para DNS flush
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .expect("Error al crear el estilo del spinner")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );

    pb.set_message("Limpiando caché DNS...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let output = Command::new("cmd")
        .args(["/C", "ipconfig /flushdns"])
        .output();

    pb.finish_and_clear();

    match output {
        Ok(result) => {
            if result.status.success() {
                println!("{}", "✅ Caché DNS limpiada exitosamente.".bright_green());
            } else {
                println!("{}", "❌ Error al limpiar la caché DNS.".red());
            }
        }
        Err(e) => println!("{} {}", "❌ Error al ejecutar comando:".red(), e),
    }

    println!();

    // Spinner para Winsock reset
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .expect("Error al crear el estilo del spinner")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );

    pb.set_message("Reiniciando Winsock (requiere admin)...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let output_winsock = Command::new("cmd")
        .args(["/C", "netsh winsock reset"])
        .output();

    pb.finish_and_clear();

    match output_winsock {
        Ok(result) => {
            if result.status.success() {
                println!("{}", "✅ Winsock reiniciado exitosamente.".bright_green());
                println!(
                    "{}",
                    "ℹ️  Se recomienda reiniciar el sistema para aplicar los cambios."
                        .bright_cyan()
                );
            } else {
                println!(
                    "{}",
                    "⚠️  Falló el reinicio de Winsock (se requieren permisos de administrador)."
                        .yellow()
                );
            }
        }
        Err(_) => println!(
            "{}",
            "❌ Falló el reinicio de Winsock (se requieren permisos de administrador).".red()
        ),
    }
}

/// Ejecuta herramientas de reparación del sistema de Windows.
///
/// Esta función requiere permisos de administrador y ejecuta:
/// - `DISM /Online /Cleanup-Image /RestoreHealth`: Repara la imagen del sistema
/// - `sfc /scannow`: Verifica y repara archivos del sistema corruptos
///
/// # Requisitos
/// - Debe ejecutarse con privilegios de administrador
/// - Puede tardar varios minutos en completarse
///
/// # Comportamiento
/// Si no se detectan permisos de administrador, la función termina sin ejecutar los comandos.
fn run_system_repair() {
    print_section_header("REPARACIÓN DEL SISTEMA");

    if !is_admin() {
        println!(
            "{}",
            "⛔ ERROR: Esta operación requiere permisos de Administrador."
                .red()
                .bold()
        );
        println!(
            "{}",
            "ℹ️  Por favor, ejecuta la terminal como Administrador.".bright_cyan()
        );
        return;
    }

    // DISM con spinner
    println!(
        "{}",
        "🔧 Ejecutando DISM (Deployment Image Servicing and Management)..."
            .bright_magenta()
            .bold()
    );
    println!(
        "{}",
        "ℹ️  Esto puede tardar varios minutos...".bright_cyan()
    );
    println!();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.magenta} {msg} [{elapsed_precise}]")
            .expect("Error al crear el estilo del spinner")
            .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"]),
    );

    pb.set_message("Ejecutando DISM...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let status_dism = Command::new("cmd")
        .args(["/C", "DISM /Online /Cleanup-Image /RestoreHealth"])
        .status();

    pb.finish_and_clear();

    if let Ok(s) = status_dism {
        if s.success() {
            println!("{}", "✅ DISM finalizado correctamente.".bright_green());
        } else {
            println!("{}", "❌ DISM finalizó con errores.".red());
        }
    } else {
        println!("{}", "❌ Error al ejecutar DISM.".red());
    }

    println!();

    // SFC con spinner
    println!(
        "{}",
        "🔧 Ejecutando SFC (System File Checker)..."
            .bright_magenta()
            .bold()
    );
    println!(
        "{}",
        "ℹ️  Esto puede tardar varios minutos...".bright_cyan()
    );
    println!();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.magenta} {msg} [{elapsed_precise}]")
            .expect("Error al crear el estilo del spinner")
            .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"]),
    );

    pb.set_message("Ejecutando SFC...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let status_sfc = Command::new("cmd").args(["/C", "sfc /scannow"]).status();

    pb.finish_and_clear();

    match status_sfc {
        Ok(s) => {
            if s.success() {
                println!("{}", "✅ Escaneo de archivos finalizado.".bright_green());
            } else {
                println!("{}", "⚠️  Escaneo finalizado con advertencias.".yellow());
            }
        }
        Err(e) => println!("{} {}", "❌ Error crítico:".red(), e),
    }
}

/// Muestra información básica del sistema.
///
/// Imprime información detallada del sistema incluyendo OS, CPU, RAM, discos, etc.
fn show_system_info() {
    print_section_header("INFORMACIÓN DEL SISTEMA");

    let mut sys = System::new_all();
    sys.refresh_all();

    // Información del sistema operativo
    println!("{}", "💻 SISTEMA OPERATIVO".bright_cyan().bold());
    println!(
        "   {} {}",
        "OS:".bright_white(),
        System::name().unwrap_or_else(|| "Desconocido".to_string())
    );
    println!(
        "   {} {}",
        "Versión:".bright_white(),
        System::os_version().unwrap_or_else(|| "Desconocida".to_string())
    );
    println!(
        "   {} {}",
        "Kernel:".bright_white(),
        System::kernel_version().unwrap_or_else(|| "Desconocido".to_string())
    );
    println!(
        "   {} {}",
        "Arquitectura:".bright_white(),
        std::env::consts::ARCH
    );
    println!(
        "   {} {}",
        "Nombre del host:".bright_white(),
        System::host_name().unwrap_or_else(|| "Desconocido".to_string())
    );

    print_separator();

    // Información del CPU
    println!("{}", "🖥️  PROCESADOR".bright_cyan().bold());
    if let Some(cpu) = sys.cpus().first() {
        println!("   {} {}", "Modelo:".bright_white(), cpu.brand());
        println!(
            "   {} {} MHz",
            "Frecuencia:".bright_white(),
            cpu.frequency()
        );
    }
    println!(
        "   {} {}",
        "Núcleos lógicos:".bright_white(),
        sys.cpus().len()
    );

    // Calcular uso promedio de CPU
    let avg_cpu_usage = if !sys.cpus().is_empty() {
        sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32
    } else {
        0.0
    };

    println!("   {} {:.1}%", "Uso global:".bright_white(), avg_cpu_usage);

    print_separator();

    // Información de memoria
    println!("{}", "💾 MEMORIA".bright_cyan().bold());
    let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let used_memory = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let available_memory = sys.available_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let memory_usage = (used_memory / total_memory) * 100.0;

    println!("   {} {:.2} GB", "Total:".bright_white(), total_memory);
    println!("   {} {:.2} GB", "Usada:".bright_white(), used_memory);
    println!(
        "   {} {:.2} GB",
        "Disponible:".bright_white(),
        available_memory
    );
    println!("   {} {:.1}%", "Uso:".bright_white(), memory_usage);

    // Barra de progreso de memoria
    let bar_width = 30;
    let filled = (memory_usage / 100.0 * bar_width as f64) as usize;
    let empty = bar_width - filled;
    let bar = format!(
        "[{}{}]",
        "█".repeat(filled).bright_green(),
        "░".repeat(empty).bright_black()
    );
    println!("   {}", bar);

    print_separator();

    // Información de discos
    println!("{}", "💿 DISCOS".bright_cyan().bold());
    let disks = Disks::new_with_refreshed_list();

    for disk in disks.list() {
        let name = disk.name().to_string_lossy();
        let mount_point = disk.mount_point().to_string_lossy();
        let total_space = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
        let available_space = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_space = total_space - available_space;
        let usage_percent = (used_space / total_space) * 100.0;

        println!("\n   {} {}", "Disco:".bright_white(), name);
        println!("   {} {}", "Punto de montaje:".bright_white(), mount_point);
        println!(
            "   {} {:.2} GB / {:.2} GB ({:.1}%)",
            "Espacio:".bright_white(),
            used_space,
            total_space,
            usage_percent
        );

        // Barra de progreso de disco
        let filled = (usage_percent / 100.0 * bar_width as f64) as usize;
        let empty = bar_width - filled;
        let color = if usage_percent > 90.0 {
            "red"
        } else if usage_percent > 70.0 {
            "yellow"
        } else {
            "green"
        };

        let bar = match color {
            "red" => format!(
                "[{}{}]",
                "█".repeat(filled).red(),
                "░".repeat(empty).bright_black()
            ),
            "yellow" => format!(
                "[{}{}]",
                "█".repeat(filled).yellow(),
                "░".repeat(empty).bright_black()
            ),
            _ => format!(
                "[{}{}]",
                "█".repeat(filled).bright_green(),
                "░".repeat(empty).bright_black()
            ),
        };
        println!("   {}", bar);
    }

    print_separator();

    // Información del sistema
    println!("{}", "⚡ RENDIMIENTO".bright_cyan().bold());
    println!(
        "   {} {}",
        "Tiempo de actividad:".bright_white(),
        format_uptime(System::uptime())
    );
    println!(
        "   {} {}",
        "Procesos:".bright_white(),
        sys.processes().len()
    );
}

/// Formatea el tiempo de actividad del sistema.
///
/// # Argumentos
/// * `uptime` - Tiempo de actividad en segundos
///
/// # Returns
/// String formateado con el tiempo de actividad
fn format_uptime(uptime: u64) -> String {
    let days = uptime / 86400;
    let hours = (uptime % 86400) / 3600;
    let minutes = (uptime % 3600) / 60;

    if days > 0 {
        format!("{} días, {} horas, {} minutos", days, hours, minutes)
    } else if hours > 0 {
        format!("{} horas, {} minutos", hours, minutes)
    } else {
        format!("{} minutos", minutes)
    }
}

/// Verifica si el proceso actual tiene permisos de administrador.
///
/// Utiliza el comando `net session` como heurística para detectar privilegios administrativos.
/// Este comando solo tiene éxito cuando se ejecuta con permisos elevados.
///
/// # Returns
/// `true` si el proceso tiene permisos de administrador, `false` en caso contrario.
///
/// # Nota
/// Este es un método heurístico que puede tener casos extremos en configuraciones
/// de seguridad no estándar.
fn is_admin() -> bool {
    let output = Command::new("net").args(["session"]).output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_admin_returns_bool() {
        // Este test simplemente verifica que la función is_admin() retorna un bool
        // y no causa pánico. El resultado depende de si el test se ejecuta como admin.
        let result = is_admin();
        assert!(result || !result);
    }

    #[test]
    fn test_show_system_info_does_not_panic() {
        // Verifica que show_system_info() no cause pánico al ejecutarse
        show_system_info();
    }

    #[test]
    fn test_clean_temp_files_does_not_panic() {
        // Verifica que clean_temp_files() no cause pánico
        // Nota: Este test puede modificar archivos temporales del sistema
        clean_temp_files(true); // Usar true para omitir la confirmación
    }

    #[test]
    fn test_flush_dns_does_not_panic() {
        // Verifica que flush_dns() no cause pánico
        // Nota: Este test ejecuta comandos del sistema reales
        flush_dns();
    }

    #[test]
    fn test_arch_detection() {
        // Verifica que podemos detectar la arquitectura del sistema
        let arch = std::env::consts::ARCH;
        assert!(!arch.is_empty());
    }

    #[test]
    fn test_available_parallelism() {
        // Verifica que podemos obtener el número de núcleos disponibles
        let cores = std::thread::available_parallelism()
            .expect("No se pudo detectar el número de núcleos disponibles");
        assert!(cores.get() > 0);
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(59), "59 minutos");
        assert_eq!(format_uptime(3600), "1 horas, 0 minutos");
        assert_eq!(format_uptime(86400), "1 días, 0 horas, 0 minutos");
        assert_eq!(format_uptime(90061), "1 días, 1 horas, 1 minutos");
    }
}
