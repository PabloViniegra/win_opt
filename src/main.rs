use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
};
use std::fs;
use std::process::Command;
use sysinfo::{Disks, System};

// Paleta de colores moderna
const BRAND_PRIMARY: Color = Color::Rgb(99, 102, 241); // Indigo vibrante
const BRAND_SECONDARY: Color = Color::Rgb(139, 92, 246); // Purple
const BRAND_ACCENT: Color = Color::Rgb(236, 72, 153); // Pink
const SUCCESS_COLOR: Color = Color::Rgb(34, 197, 94); // Green
const WARNING_COLOR: Color = Color::Rgb(251, 191, 36); // Amber
const ERROR_COLOR: Color = Color::Rgb(239, 68, 68); // Red
const INFO_COLOR: Color = Color::Rgb(59, 130, 246); // Blue
const TEXT_PRIMARY: Color = Color::Rgb(248, 250, 252); // Slate 50
const TEXT_SECONDARY: Color = Color::Rgb(148, 163, 184); // Slate 400
const BG_DARKER: Color = Color::Rgb(15, 23, 42); // Slate 900
const BG_DARK: Color = Color::Rgb(30, 41, 59); // Slate 800

/// Vista actual de la aplicación
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    MainMenu,
    Clean,
    Network,
    Repair,
    Info,
    Optimize,
    WindowsUpdate,
    Privacy,
}

/// Estado de ejecución de una operación
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperationState {
    Idle,
    Running,
    Completed,
}

/// Estructura principal de la aplicación
struct App {
    /// Vista actual
    current_view: View,
    /// Índice del item seleccionado en el menú
    selected_menu_item: usize,
    /// Logs de operaciones
    operation_logs: Vec<String>,
    /// Estado de la operación actual
    operation_state: OperationState,
    /// Estadísticas de la última limpieza
    clean_stats: CleanStats,
    /// Flag para salir de la aplicación
    should_quit: bool,
    /// Scroll vertical para logs
    scroll_offset: u16,
}

/// Estadísticas de limpieza
#[derive(Debug, Clone, Default)]
struct CleanStats {
    deleted_count: usize,
    failed_count: usize,
    size_freed: u64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_view: View::MainMenu,
            selected_menu_item: 0,
            operation_logs: Vec::new(),
            operation_state: OperationState::Idle,
            clean_stats: CleanStats::default(),
            should_quit: false,
            scroll_offset: 0,
        }
    }
}

impl App {
    /// Ejecuta el loop principal de la aplicación
    fn run(mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// Dibuja la interfaz según la vista actual
    fn draw(&mut self, frame: &mut Frame) {
        match self.current_view {
            View::MainMenu => self.draw_main_menu(frame),
            View::Clean => self.draw_clean_view(frame),
            View::Network => self.draw_network_view(frame),
            View::Repair => self.draw_repair_view(frame),
            View::Info => self.draw_info_view(frame),
            View::Optimize => self.draw_optimize_view(frame),
            View::WindowsUpdate => self.draw_windows_update_view(frame),
            View::Privacy => self.draw_privacy_view(frame),
        }
    }

    /// Maneja los eventos de teclado
    fn handle_events(&mut self) -> std::io::Result<()> {
        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match self.current_view {
                View::MainMenu => self.handle_menu_input(key.code),
                _ => self.handle_operation_input(key.code),
            }
        }
        Ok(())
    }

    /// Maneja input en el menú principal
    fn handle_menu_input(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected_menu_item = (self.selected_menu_item + 1).min(8);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_menu_item = self.selected_menu_item.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.operation_logs.clear();
                self.scroll_offset = 0;
                self.current_view = match self.selected_menu_item {
                    0 => {
                        self.execute_clean();
                        View::Clean
                    }
                    1 => {
                        self.execute_windows_update_cleanup();
                        View::WindowsUpdate
                    }
                    2 => {
                        self.execute_network();
                        View::Network
                    }
                    3 => {
                        self.execute_repair();
                        View::Repair
                    }
                    4 => {
                        self.execute_optimize();
                        View::Optimize
                    }
                    5 => {
                        self.execute_privacy();
                        View::Privacy
                    }
                    6 => View::Info,
                    7 => {
                        self.should_quit = true;
                        View::MainMenu
                    }
                    _ => View::MainMenu,
                };
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    /// Maneja input en las vistas de operaciones
    fn handle_operation_input(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.current_view = View::MainMenu;
                self.operation_state = OperationState::Idle;
            }
            KeyCode::Down => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }
            KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Dibuja el menú principal
    fn draw_main_menu(&mut self, frame: &mut Frame) {
        let main_block = Block::default().style(Style::default().bg(BG_DARKER));
        frame.render_widget(main_block, frame.area());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(13),
                Constraint::Min(8),
                Constraint::Length(4),
            ])
            .split(frame.area());

        // Banner moderno con degradado simulado
        self.render_modern_banner(frame, chunks[0]);

        // Menú con diseño moderno
        self.render_modern_menu(frame, chunks[1]);

        // Footer elegante
        self.render_modern_footer(frame, chunks[2]);
    }

    /// Renderiza un banner moderno con efectos visuales
    fn render_modern_banner(&self, frame: &mut Frame, area: Rect) {
        let banner_lines = vec![
            Line::from(vec![
                Span::raw("  ╔═══════════════════════════════════════════════════════╗  ")
                    .fg(BRAND_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  ║  ").fg(BRAND_PRIMARY),
                Span::raw("██╗    ██╗██╗███╗   ██╗     ██████╗ ██████╗ ████████╗")
                    .fg(BRAND_SECONDARY)
                    .bold(),
                Span::raw("  ║  ").fg(BRAND_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  ║  ").fg(BRAND_PRIMARY),
                Span::raw("██║    ██║██║████╗  ██║    ██╔═══██╗██╔══██╗╚══██╔══╝")
                    .fg(BRAND_SECONDARY)
                    .bold(),
                Span::raw("  ║  ").fg(BRAND_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  ║  ").fg(BRAND_PRIMARY),
                Span::raw("██║ █╗ ██║██║██╔██╗ ██║    ██║   ██║██████╔╝   ██║   ")
                    .fg(BRAND_ACCENT)
                    .bold(),
                Span::raw("  ║  ").fg(BRAND_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  ║  ").fg(BRAND_PRIMARY),
                Span::raw("██║███╗██║██║██║╚██╗██║    ██║   ██║██╔═══╝    ██║   ")
                    .fg(BRAND_ACCENT)
                    .bold(),
                Span::raw("  ║  ").fg(BRAND_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  ║  ").fg(BRAND_PRIMARY),
                Span::raw("╚███╔███╔╝██║██║ ╚████║    ╚██████╔╝██║        ██║   ")
                    .fg(Color::Rgb(168, 85, 247))
                    .bold(),
                Span::raw("  ║  ").fg(BRAND_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  ║  ").fg(BRAND_PRIMARY),
                Span::raw(" ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝     ╚═════╝ ╚═╝        ╚═╝   ")
                    .fg(Color::Rgb(168, 85, 247))
                    .bold(),
                Span::raw("  ║  ").fg(BRAND_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  ║                                                       ║  ")
                    .fg(BRAND_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  ║           ").fg(BRAND_PRIMARY),
                Span::raw("Windows 11 Optimizer ").fg(TEXT_PRIMARY).bold(),
                Span::raw("•").fg(BRAND_ACCENT),
                Span::raw(" v1.0.0              ").fg(TEXT_SECONDARY),
                Span::raw("║  ").fg(BRAND_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  ║           ").fg(BRAND_PRIMARY),
                Span::raw("⚡ Herramienta de optimización del sistema  ")
                    .fg(TEXT_SECONDARY)
                    .italic(),
                Span::raw("      ║  ").fg(BRAND_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  ╚═══════════════════════════════════════════════════════╝  ")
                    .fg(BRAND_PRIMARY),
            ]),
        ];

        let banner_text = Text::from(banner_lines);
        let banner_widget = Paragraph::new(banner_text).alignment(Alignment::Center);
        frame.render_widget(banner_widget, area);
    }

    /// Renderiza el menú con diseño moderno
    fn render_modern_menu(&mut self, frame: &mut Frame, area: Rect) {
        let menu_items = [
            (
                "🧹",
                "Limpieza de Archivos Temporales",
                "Libera espacio en disco",
            ),
            (
                "🔄",
                "Limpieza de Windows Update",
                "Limpia archivos de actualización",
            ),
            ("🌐", "Limpieza de Red", "DNS flush & Winsock reset"),
            ("🔧", "Reparación del Sistema", "DISM & SFC scan"),
            (
                "⚡",
                "Optimización Avanzada",
                "Servicios, energía y rendimiento",
            ),
            (
                "🔒",
                "Privacidad y Telemetría",
                "Desactiva recolección de datos",
            ),
            ("💻", "Información del Sistema", "Detalles del hardware"),
            ("🚪", "Salir", "Cerrar la aplicación"),
        ];

        let items: Vec<ListItem> = menu_items
            .iter()
            .enumerate()
            .map(|(i, (icon, title, desc))| {
                let is_selected = i == self.selected_menu_item;

                let (fg_color, icon_color, desc_color) = if is_selected {
                    (TEXT_PRIMARY, BRAND_ACCENT, TEXT_PRIMARY)
                } else {
                    (TEXT_PRIMARY, BRAND_PRIMARY, TEXT_SECONDARY)
                };

                let content = Line::from(vec![
                    Span::raw("  "),
                    Span::raw(*icon).fg(icon_color).bold(),
                    Span::raw("  "),
                    Span::raw(*title).fg(fg_color).bold(),
                    Span::raw("  "),
                    Span::raw(format!("— {}", desc)).fg(desc_color).italic(),
                ]);

                let style = if is_selected {
                    Style::default()
                        .bg(Color::Rgb(51, 65, 85))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(content).style(style)
            })
            .collect();

        let menu_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BRAND_PRIMARY))
            .border_set(symbols::border::ROUNDED)
            .title(Line::from(vec![
                Span::raw(" "),
                Span::raw("✨ ").fg(BRAND_ACCENT),
                Span::raw("Menú Principal ").fg(TEXT_PRIMARY).bold(),
            ]))
            .title_alignment(Alignment::Center);

        let menu_list = List::new(items).block(menu_block);
        frame.render_widget(menu_list, area);
    }

    /// Renderiza un footer moderno
    fn render_modern_footer(&self, frame: &mut Frame, area: Rect) {
        let footer_text = Line::from(vec![
            Span::raw("  ").fg(BRAND_ACCENT),
            Span::raw("↑↓").fg(BRAND_PRIMARY).bold(),
            Span::raw(" Navegar  ").fg(TEXT_SECONDARY),
            Span::raw("•").fg(BRAND_ACCENT),
            Span::raw("  ").fg(BRAND_ACCENT),
            Span::raw("Enter").fg(BRAND_PRIMARY).bold(),
            Span::raw(" Seleccionar  ").fg(TEXT_SECONDARY),
            Span::raw("•").fg(BRAND_ACCENT),
            Span::raw("  ").fg(BRAND_ACCENT),
            Span::raw("Q/Esc").fg(BRAND_PRIMARY).bold(),
            Span::raw(" Salir  ").fg(TEXT_SECONDARY),
        ]);

        let footer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BRAND_PRIMARY))
            .border_set(symbols::border::ROUNDED);

        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .block(footer_block);
        frame.render_widget(footer, area);
    }

    /// Dibuja la vista de limpieza con diseño mejorado
    fn draw_clean_view(&mut self, frame: &mut Frame) {
        let main_block = Block::default().style(Style::default().bg(BG_DARKER));
        frame.render_widget(main_block, frame.area());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(10),
            ])
            .split(frame.area());

        // Título elegante
        let title_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BRAND_PRIMARY))
            .border_set(symbols::border::ROUNDED);

        let title = Paragraph::new(Line::from(vec![
            Span::raw("🧹 ").fg(BRAND_ACCENT).bold(),
            Span::raw("Limpieza de Archivos Temporales")
                .fg(TEXT_PRIMARY)
                .bold(),
        ]))
        .alignment(Alignment::Center)
        .block(title_block);
        frame.render_widget(title, chunks[0]);

        // Logs con diseño moderno
        self.render_styled_logs(frame, chunks[1], "Registro de Operaciones");

        // Estadísticas elegantes
        self.render_clean_stats(frame, chunks[2]);
    }

    /// Renderiza estadísticas de limpieza con diseño moderno
    fn render_clean_stats(&self, frame: &mut Frame, area: Rect) {
        let stats_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(SUCCESS_COLOR))
            .border_set(symbols::border::ROUNDED)
            .title(Line::from(vec![
                Span::raw(" "),
                Span::raw("📊 ").fg(BRAND_ACCENT),
                Span::raw("Estadísticas ").fg(TEXT_PRIMARY).bold(),
            ]));

        let inner = stats_block.inner(area);
        frame.render_widget(stats_block, area);

        let stats_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Min(1),
            ])
            .split(inner);

        // Elementos eliminados
        let deleted_line = Line::from(vec![
            Span::raw("  ✅  ").fg(SUCCESS_COLOR).bold(),
            Span::raw("Elementos eliminados: ").fg(TEXT_SECONDARY),
            Span::raw(self.clean_stats.deleted_count.to_string())
                .fg(SUCCESS_COLOR)
                .bold(),
        ]);
        frame.render_widget(Paragraph::new(deleted_line), stats_chunks[0]);

        // Elementos omitidos
        let failed_line = Line::from(vec![
            Span::raw("  ⚠️  ").fg(WARNING_COLOR).bold(),
            Span::raw("Elementos omitidos: ").fg(TEXT_SECONDARY),
            Span::raw(self.clean_stats.failed_count.to_string())
                .fg(WARNING_COLOR)
                .bold(),
        ]);
        frame.render_widget(Paragraph::new(failed_line), stats_chunks[1]);

        // Espacio liberado
        let size_mb = self.clean_stats.size_freed as f64 / 1024.0 / 1024.0;
        let freed_line = Line::from(vec![
            Span::raw("  💾  ").fg(INFO_COLOR).bold(),
            Span::raw("Espacio liberado: ").fg(TEXT_SECONDARY),
            Span::raw(format!("{:.2} MB", size_mb))
                .fg(BRAND_PRIMARY)
                .bold(),
        ]);
        frame.render_widget(Paragraph::new(freed_line), stats_chunks[2]);

        // Ayuda
        let help = Line::from(vec![
            Span::raw("  ").fg(BRAND_ACCENT),
            Span::raw("Q/Esc").fg(BRAND_PRIMARY).bold(),
            Span::raw(" Volver  ").fg(TEXT_SECONDARY),
            Span::raw("•").fg(BRAND_ACCENT),
            Span::raw("  ").fg(BRAND_ACCENT),
            Span::raw("↑↓").fg(BRAND_PRIMARY).bold(),
            Span::raw(" Scroll  ").fg(TEXT_SECONDARY),
        ]);
        frame.render_widget(
            Paragraph::new(help).alignment(Alignment::Center),
            stats_chunks[3],
        );
    }

    /// Dibuja la vista de red con diseño mejorado
    fn draw_network_view(&mut self, frame: &mut Frame) {
        self.draw_generic_operation_view(frame, "🌐", "Limpieza de Red");
    }

    /// Dibuja la vista de reparación con diseño mejorado
    fn draw_repair_view(&mut self, frame: &mut Frame) {
        self.draw_generic_operation_view(frame, "🔧", "Reparación del Sistema");
    }

    /// Dibuja la vista de optimización
    fn draw_optimize_view(&mut self, frame: &mut Frame) {
        self.draw_generic_operation_view(frame, "⚡", "Optimización Avanzada");
    }

    /// Dibuja la vista de Windows Update cleanup
    fn draw_windows_update_view(&mut self, frame: &mut Frame) {
        self.draw_generic_operation_view(frame, "🔄", "Limpieza de Windows Update");
    }

    /// Dibuja la vista de privacidad
    fn draw_privacy_view(&mut self, frame: &mut Frame) {
        self.draw_generic_operation_view(frame, "🔒", "Privacidad y Telemetría");
    }

    /// Dibuja una vista genérica de operación
    fn draw_generic_operation_view(&mut self, frame: &mut Frame, icon: &str, title: &str) {
        let main_block = Block::default().style(Style::default().bg(BG_DARKER));
        frame.render_widget(main_block, frame.area());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(frame.area());

        // Título
        let title_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BRAND_PRIMARY))
            .border_set(symbols::border::ROUNDED);

        let title_widget = Paragraph::new(Line::from(vec![
            Span::raw(format!("{} ", icon)).fg(BRAND_ACCENT).bold(),
            Span::raw(title).fg(TEXT_PRIMARY).bold(),
        ]))
        .alignment(Alignment::Center)
        .block(title_block);
        frame.render_widget(title_widget, chunks[0]);

        // Logs
        self.render_styled_logs(frame, chunks[1], "Registro de Operaciones");

        // Footer
        self.render_operation_footer(frame, chunks[2]);
    }

    /// Renderiza logs con estilo mejorado
    fn render_styled_logs(&self, frame: &mut Frame, area: Rect, title: &str) {
        let log_lines: Vec<Line> = self
            .operation_logs
            .iter()
            .map(|log| {
                // Colorear logs según contenido
                if log.contains("✅") {
                    Line::from(vec![Span::raw(log.as_str()).fg(SUCCESS_COLOR)])
                } else if log.contains("⚠️") || log.contains("ℹ️") {
                    Line::from(vec![Span::raw(log.as_str()).fg(WARNING_COLOR)])
                } else if log.contains("❌") || log.contains("⛔") {
                    Line::from(vec![Span::raw(log.as_str()).fg(ERROR_COLOR)])
                } else if log.contains("🧹")
                    || log.contains("🌐")
                    || log.contains("🔧")
                    || log.contains("⚡")
                    || log.contains("🔄")
                    || log.contains("🔒")
                {
                    Line::from(vec![Span::raw(log.as_str()).fg(BRAND_PRIMARY).bold()])
                } else {
                    Line::from(vec![Span::raw(log.as_str()).fg(TEXT_PRIMARY)])
                }
            })
            .collect();

        let logs_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BRAND_PRIMARY))
            .border_set(symbols::border::ROUNDED)
            .title(Line::from(vec![
                Span::raw(" "),
                Span::raw("📋 ").fg(BRAND_ACCENT),
                Span::raw(title).fg(TEXT_PRIMARY).bold(),
                Span::raw(" "),
            ]));

        let logs = Paragraph::new(log_lines)
            .block(logs_block)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll_offset, 0));
        frame.render_widget(logs, area);
    }

    /// Renderiza footer para vistas de operación
    fn render_operation_footer(&self, frame: &mut Frame, area: Rect) {
        let footer_text = Line::from(vec![
            Span::raw("  ").fg(BRAND_ACCENT),
            Span::raw("Q/Esc").fg(BRAND_PRIMARY).bold(),
            Span::raw(" Volver al menú  ").fg(TEXT_SECONDARY),
            Span::raw("•").fg(BRAND_ACCENT),
            Span::raw("  ").fg(BRAND_ACCENT),
            Span::raw("↑↓").fg(BRAND_PRIMARY).bold(),
            Span::raw(" Scroll  ").fg(TEXT_SECONDARY),
        ]);

        let footer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BRAND_PRIMARY))
            .border_set(symbols::border::ROUNDED);

        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .block(footer_block);
        frame.render_widget(footer, area);
    }

    /// Dibuja la vista de información del sistema con diseño mejorado
    fn draw_info_view(&mut self, frame: &mut Frame) {
        let main_block = Block::default().style(Style::default().bg(BG_DARKER));
        frame.render_widget(main_block, frame.area());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(9),
                Constraint::Length(8),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(frame.area());

        // Título
        let title_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BRAND_PRIMARY))
            .border_set(symbols::border::ROUNDED);

        let title = Paragraph::new(Line::from(vec![
            Span::raw("💻 ").fg(BRAND_ACCENT).bold(),
            Span::raw("Información del Sistema").fg(TEXT_PRIMARY).bold(),
        ]))
        .alignment(Alignment::Center)
        .block(title_block);
        frame.render_widget(title, chunks[0]);

        let mut sys = System::new_all();
        sys.refresh_all();

        // Información del SO
        self.render_os_info(frame, chunks[1], &sys);

        // CPU y Memoria
        self.render_cpu_mem_info(frame, chunks[2], &sys);

        // Discos y gauge de memoria
        self.render_storage_info(frame, chunks[3], &sys);

        // Footer
        let footer_text = Line::from(vec![
            Span::raw("  ").fg(BRAND_ACCENT),
            Span::raw("Q/Esc").fg(BRAND_PRIMARY).bold(),
            Span::raw(" Volver al menú  ").fg(TEXT_SECONDARY),
        ]);

        let footer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BRAND_PRIMARY))
            .border_set(symbols::border::ROUNDED);

        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .block(footer_block);
        frame.render_widget(footer, chunks[4]);
    }

    /// Renderiza información del OS
    fn render_os_info(&self, frame: &mut Frame, area: Rect, _sys: &System) {
        let os_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(INFO_COLOR))
            .border_set(symbols::border::ROUNDED)
            .title(Line::from(vec![
                Span::raw(" "),
                Span::raw("🖥️  ").fg(BRAND_ACCENT),
                Span::raw("Sistema Operativo ").fg(TEXT_PRIMARY).bold(),
            ]));

        let os_info = vec![
            Line::from(vec![
                Span::raw("  "),
                Span::raw("OS: ").fg(BRAND_PRIMARY).bold(),
                Span::raw(System::name().unwrap_or_else(|| "Desconocido".to_string()))
                    .fg(TEXT_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Versión: ").fg(BRAND_PRIMARY).bold(),
                Span::raw(System::os_version().unwrap_or_else(|| "Desconocida".to_string()))
                    .fg(TEXT_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Kernel: ").fg(BRAND_PRIMARY).bold(),
                Span::raw(System::kernel_version().unwrap_or_else(|| "Desconocido".to_string()))
                    .fg(TEXT_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Host: ").fg(BRAND_PRIMARY).bold(),
                Span::raw(System::host_name().unwrap_or_else(|| "Desconocido".to_string()))
                    .fg(TEXT_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Arquitectura: ").fg(BRAND_PRIMARY).bold(),
                Span::raw(std::env::consts::ARCH).fg(TEXT_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Tiempo activo: ").fg(BRAND_PRIMARY).bold(),
                Span::raw(format_uptime(System::uptime())).fg(SUCCESS_COLOR),
            ]),
        ];

        let os_widget = Paragraph::new(os_info).block(os_block);
        frame.render_widget(os_widget, area);
    }

    /// Renderiza información de CPU y memoria
    fn render_cpu_mem_info(&self, frame: &mut Frame, area: Rect, sys: &System) {
        let cpu_count = sys.cpus().len();
        let cpu_brand = sys
            .cpus()
            .first()
            .map(|cpu| cpu.brand())
            .unwrap_or("Desconocido");

        let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_memory = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

        let cpu_mem_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BRAND_SECONDARY))
            .border_set(symbols::border::ROUNDED)
            .title(Line::from(vec![
                Span::raw(" "),
                Span::raw("⚡ ").fg(BRAND_ACCENT),
                Span::raw("CPU y Memoria ").fg(TEXT_PRIMARY).bold(),
            ]));

        let cpu_mem_info = vec![
            Line::from(vec![
                Span::raw("  "),
                Span::raw("CPU: ").fg(BRAND_PRIMARY).bold(),
                Span::raw(cpu_brand).fg(TEXT_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Núcleos: ").fg(BRAND_PRIMARY).bold(),
                Span::raw(cpu_count.to_string()).fg(TEXT_PRIMARY),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Memoria Total: ").fg(BRAND_PRIMARY).bold(),
                Span::raw(format!("{:.2} GB", total_memory)).fg(TEXT_PRIMARY),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Memoria Usada: ").fg(BRAND_PRIMARY).bold(),
                Span::raw(format!("{:.2} GB", used_memory)).fg(WARNING_COLOR),
            ]),
        ];

        let cpu_mem_widget = Paragraph::new(cpu_mem_info).block(cpu_mem_block);
        frame.render_widget(cpu_mem_widget, area);
    }

    /// Renderiza información de almacenamiento
    fn render_storage_info(&self, frame: &mut Frame, area: Rect, sys: &System) {
        let storage_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Gauge de memoria
        let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_memory = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let memory_percent = if total_memory > 0.0 {
            (used_memory / total_memory * 100.0) as u16
        } else {
            0
        };

        let gauge_color = if memory_percent > 90 {
            ERROR_COLOR
        } else if memory_percent > 70 {
            WARNING_COLOR
        } else {
            SUCCESS_COLOR
        };

        let memory_gauge = Gauge::default()
            .block(
                Block::default()
                    .title(Line::from(vec![
                        Span::raw(" "),
                        Span::raw("💾 ").fg(BRAND_ACCENT),
                        Span::raw("Uso de Memoria ").fg(TEXT_PRIMARY).bold(),
                    ]))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(gauge_color))
                    .border_set(symbols::border::ROUNDED),
            )
            .gauge_style(Style::default().fg(gauge_color).bg(BG_DARK))
            .percent(memory_percent)
            .label(format!("{}%", memory_percent));
        frame.render_widget(memory_gauge, storage_chunks[0]);

        // Discos
        let disks = Disks::new_with_refreshed_list();
        let disk_lines: Vec<Line> = disks
            .list()
            .iter()
            .map(|disk| {
                let total_space = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
                let available_space = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0;
                let used_space = total_space - available_space;
                let usage_percent = if total_space > 0.0 {
                    (used_space / total_space) * 100.0
                } else {
                    0.0
                };

                let color = if usage_percent > 90.0 {
                    ERROR_COLOR
                } else if usage_percent > 70.0 {
                    WARNING_COLOR
                } else {
                    SUCCESS_COLOR
                };

                Line::from(vec![
                    Span::raw("  "),
                    Span::raw(format!("{}: ", disk.mount_point().to_string_lossy()))
                        .fg(BRAND_PRIMARY)
                        .bold(),
                    Span::raw(format!("{:.1} GB / {:.1} GB ", used_space, total_space))
                        .fg(TEXT_PRIMARY),
                    Span::raw(format!("({:.1}%)", usage_percent))
                        .fg(color)
                        .bold(),
                ])
            })
            .collect();

        let disk_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BRAND_ACCENT))
            .border_set(symbols::border::ROUNDED)
            .title(Line::from(vec![
                Span::raw(" "),
                Span::raw("💿 ").fg(BRAND_ACCENT),
                Span::raw("Discos ").fg(TEXT_PRIMARY).bold(),
            ]));

        let disk_widget = Paragraph::new(disk_lines).block(disk_block);
        frame.render_widget(disk_widget, storage_chunks[1]);
    }

    /// Ejecuta la operación de limpieza
    fn execute_clean(&mut self) {
        self.operation_state = OperationState::Running;
        self.operation_logs
            .push("🧹 Iniciando limpieza de archivos temporales...".to_string());

        let temp_dir = std::env::temp_dir();
        self.operation_logs
            .push(format!("📁 Directorio: {}", temp_dir.to_string_lossy()));

        let mut deleted_count = 0;
        let mut size_freed: u64 = 0;
        let mut failed_count = 0;

        match fs::read_dir(&temp_dir) {
            Ok(entries) => {
                let entries_vec: Vec<_> = entries.flatten().collect();
                let total = entries_vec.len();

                self.operation_logs
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
                        self.operation_logs
                            .push(format!("Procesando... {}/{}", idx + 1, total));
                    }
                }

                self.clean_stats = CleanStats {
                    deleted_count,
                    failed_count,
                    size_freed,
                };

                self.operation_logs
                    .push("✅ Limpieza completada".to_string());
            }
            Err(_) => {
                self.operation_logs
                    .push("❌ Error al leer el directorio temporal".to_string());
            }
        }

        self.operation_state = OperationState::Completed;
    }

    /// Ejecuta las operaciones de red
    fn execute_network(&mut self) {
        self.operation_state = OperationState::Running;
        self.operation_logs
            .push("🌐 Iniciando operaciones de red...".to_string());

        // DNS Flush
        self.operation_logs
            .push("Ejecutando: ipconfig /flushdns".to_string());
        let output = Command::new("cmd")
            .args(["/C", "ipconfig /flushdns"])
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    self.operation_logs
                        .push("✅ Caché DNS limpiada exitosamente".to_string());
                } else {
                    self.operation_logs
                        .push("❌ Error al limpiar la caché DNS".to_string());
                }
            }
            Err(e) => self.operation_logs.push(format!("❌ Error: {}", e)),
        }

        // Winsock Reset
        self.operation_logs.push("".to_string());
        self.operation_logs
            .push("Ejecutando: netsh winsock reset".to_string());
        let output_winsock = Command::new("cmd")
            .args(["/C", "netsh winsock reset"])
            .output();

        match output_winsock {
            Ok(result) => {
                if result.status.success() {
                    self.operation_logs
                        .push("✅ Winsock reiniciado exitosamente".to_string());
                    self.operation_logs.push(
                        "ℹ️  Se recomienda reiniciar el sistema para aplicar los cambios"
                            .to_string(),
                    );
                } else {
                    self.operation_logs.push(
                        "⚠️  Falló el reinicio de Winsock (se requieren permisos de administrador)"
                            .to_string(),
                    );
                }
            }
            Err(_) => {
                self.operation_logs.push(
                    "❌ Falló el reinicio de Winsock (se requieren permisos de administrador)"
                        .to_string(),
                );
            }
        }

        self.operation_state = OperationState::Completed;
    }

    /// Ejecuta las operaciones de reparación
    fn execute_repair(&mut self) {
        self.operation_state = OperationState::Running;
        self.operation_logs
            .push("🔧 Iniciando reparación del sistema...".to_string());

        if !is_admin() {
            self.operation_logs
                .push("⛔ ERROR: Esta operación requiere permisos de Administrador".to_string());
            self.operation_logs
                .push("ℹ️  Por favor, ejecuta la aplicación como Administrador".to_string());
            self.operation_state = OperationState::Completed;
            return;
        }

        // DISM
        self.operation_logs.push("".to_string());
        self.operation_logs
            .push("🔧 Ejecutando DISM (Deployment Image Servicing and Management)...".to_string());
        self.operation_logs
            .push("ℹ️  Esto puede tardar varios minutos...".to_string());

        let status_dism = Command::new("cmd")
            .args(["/C", "DISM /Online /Cleanup-Image /RestoreHealth"])
            .status();

        match status_dism {
            Ok(s) => {
                if s.success() {
                    self.operation_logs
                        .push("✅ DISM finalizado correctamente".to_string());
                } else {
                    self.operation_logs
                        .push("❌ DISM finalizó con errores".to_string());
                }
            }
            Err(_) => {
                self.operation_logs
                    .push("❌ Error al ejecutar DISM".to_string());
            }
        }

        // SFC
        self.operation_logs.push("".to_string());
        self.operation_logs
            .push("🔧 Ejecutando SFC (System File Checker)...".to_string());
        self.operation_logs
            .push("ℹ️  Esto puede tardar varios minutos...".to_string());

        let status_sfc = Command::new("cmd").args(["/C", "sfc /scannow"]).status();

        match status_sfc {
            Ok(s) => {
                if s.success() {
                    self.operation_logs
                        .push("✅ Escaneo de archivos finalizado".to_string());
                } else {
                    self.operation_logs
                        .push("⚠️  Escaneo finalizado con advertencias".to_string());
                }
            }
            Err(e) => self.operation_logs.push(format!("❌ Error crítico: {}", e)),
        }

        self.operation_state = OperationState::Completed;
    }

    /// Ejecuta optimización avanzada del sistema
    fn execute_optimize(&mut self) {
        self.operation_state = OperationState::Running;
        self.operation_logs
            .push("⚡ Iniciando optimización avanzada del sistema...".to_string());

        if !is_admin() {
            self.operation_logs
                .push("⛔ ERROR: Esta operación requiere permisos de Administrador".to_string());
            self.operation_logs
                .push("ℹ️  Por favor, ejecuta la aplicación como Administrador".to_string());
            self.operation_state = OperationState::Completed;
            return;
        }

        // Limpiar Prefetch
        self.operation_logs.push("".to_string());
        self.operation_logs
            .push("🗑️  Limpiando archivos Prefetch...".to_string());

        let prefetch_path = "C:\\Windows\\Prefetch\\*";
        let prefetch_result = Command::new("cmd")
            .args(["/C", &format!("del /f /q {}", prefetch_path)])
            .output();

        match prefetch_result {
            Ok(result) => {
                if result.status.success() {
                    self.operation_logs
                        .push("✅ Archivos Prefetch limpiados".to_string());
                } else {
                    self.operation_logs
                        .push("⚠️  Algunos archivos Prefetch no pudieron eliminarse".to_string());
                }
            }
            Err(e) => self
                .operation_logs
                .push(format!("❌ Error limpiando Prefetch: {}", e)),
        }

        // Configurar plan de energía de alto rendimiento
        self.operation_logs.push("".to_string());
        self.operation_logs
            .push("⚡ Configurando plan de energía de alto rendimiento...".to_string());

        let power_result = Command::new("powercfg")
            .args(["/setactive", "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"])
            .output();

        match power_result {
            Ok(result) => {
                if result.status.success() {
                    self.operation_logs
                        .push("✅ Plan de energía configurado a Alto Rendimiento".to_string());
                } else {
                    self.operation_logs
                        .push("⚠️  No se pudo cambiar el plan de energía".to_string());
                }
            }
            Err(e) => self
                .operation_logs
                .push(format!("❌ Error configurando energía: {}", e)),
        }

        // Deshabilitar servicios innecesarios (con precaución)
        self.operation_logs.push("".to_string());
        self.operation_logs
            .push("🔧 Optimizando servicios del sistema...".to_string());

        let services_to_disable = [
            ("DiagTrack", "Servicio de telemetría"),
            ("SysMain", "SuperFetch (en SSDs)"),
        ];

        for (service, description) in services_to_disable {
            let service_result = Command::new("sc")
                .args(["config", service, "start=", "disabled"])
                .output();

            match service_result {
                Ok(result) => {
                    if result.status.success() {
                        self.operation_logs.push(format!(
                            "✅ Servicio deshabilitado: {} ({})",
                            service, description
                        ));
                    } else {
                        self.operation_logs
                            .push(format!("⚠️  No se pudo deshabilitar: {}", service));
                    }
                }
                Err(_) => {
                    self.operation_logs
                        .push(format!("❌ Error con servicio: {}", service));
                }
            }
        }

        self.operation_logs.push("".to_string());
        self.operation_logs
            .push("✅ Optimización avanzada completada".to_string());
        self.operation_logs
            .push("ℹ️  Se recomienda reiniciar el sistema".to_string());

        self.operation_state = OperationState::Completed;
    }

    /// Ejecuta limpieza de archivos de Windows Update
    fn execute_windows_update_cleanup(&mut self) {
        self.operation_state = OperationState::Running;
        self.operation_logs
            .push("🔄 Iniciando limpieza de Windows Update...".to_string());

        if !is_admin() {
            self.operation_logs
                .push("⛔ ERROR: Esta operación requiere permisos de Administrador".to_string());
            self.operation_logs
                .push("ℹ️  Por favor, ejecuta la aplicación como Administrador".to_string());
            self.operation_state = OperationState::Completed;
            return;
        }

        // Limpiar archivos de Windows Update
        self.operation_logs.push("".to_string());
        self.operation_logs
            .push("🗑️  Eliminando archivos de actualización antiguos...".to_string());

        let cleanup_result = Command::new("cmd")
            .args(["/C", "cleanmgr /sageset:1 & cleanmgr /sagerun:1"])
            .output();

        match cleanup_result {
            Ok(result) => {
                if result.status.success() {
                    self.operation_logs
                        .push("✅ Limpieza de disco iniciada".to_string());
                } else {
                    self.operation_logs
                        .push("⚠️  Error al iniciar limpieza de disco".to_string());
                }
            }
            Err(e) => self.operation_logs.push(format!("❌ Error: {}", e)),
        }

        // Limpiar componentes de Windows Update
        self.operation_logs.push("".to_string());
        self.operation_logs
            .push("🔧 Ejecutando limpieza de componentes...".to_string());

        let dism_cleanup = Command::new("cmd")
            .args(["/C", "DISM /Online /Cleanup-Image /StartComponentCleanup"])
            .status();

        match dism_cleanup {
            Ok(s) => {
                if s.success() {
                    self.operation_logs
                        .push("✅ Componentes limpiados exitosamente".to_string());
                } else {
                    self.operation_logs
                        .push("⚠️  Limpieza de componentes con advertencias".to_string());
                }
            }
            Err(e) => self
                .operation_logs
                .push(format!("❌ Error en limpieza: {}", e)),
        }

        self.operation_logs.push("".to_string());
        self.operation_logs
            .push("✅ Limpieza de Windows Update completada".to_string());

        self.operation_state = OperationState::Completed;
    }

    /// Ejecuta desactivación de telemetría y mejoras de privacidad
    fn execute_privacy(&mut self) {
        self.operation_state = OperationState::Running;
        self.operation_logs
            .push("🔒 Iniciando configuración de privacidad...".to_string());

        if !is_admin() {
            self.operation_logs
                .push("⛔ ERROR: Esta operación requiere permisos de Administrador".to_string());
            self.operation_logs
                .push("ℹ️  Por favor, ejecuta la aplicación como Administrador".to_string());
            self.operation_state = OperationState::Completed;
            return;
        }

        // Deshabilitar telemetría de Windows
        self.operation_logs.push("".to_string());
        self.operation_logs
            .push("🛡️  Deshabilitando telemetría de Windows...".to_string());

        let telemetry_services = ["DiagTrack", "dmwappushservice", "WerSvc"];

        for service in telemetry_services {
            let result = Command::new("sc")
                .args(["config", service, "start=", "disabled"])
                .output();

            match result {
                Ok(output) => {
                    if output.status.success() {
                        self.operation_logs
                            .push(format!("✅ Servicio {} deshabilitado", service));
                    } else {
                        self.operation_logs
                            .push(format!("⚠️  No se pudo deshabilitar {}", service));
                    }
                }
                Err(_) => {
                    self.operation_logs
                        .push(format!("❌ Error con servicio {}", service));
                }
            }
        }

        // Deshabilitar tareas programadas de telemetría
        self.operation_logs.push("".to_string());
        self.operation_logs
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
                self.operation_logs
                    .push("✅ Tarea deshabilitada".to_string());
            }
        }

        self.operation_logs.push("".to_string());
        self.operation_logs
            .push("✅ Configuración de privacidad completada".to_string());
        self.operation_logs.push(
            "ℹ️  Se recomienda reiniciar el sistema para aplicar todos los cambios".to_string(),
        );

        self.operation_state = OperationState::Completed;
    }
}

/// Formatea el tiempo de actividad del sistema
fn format_uptime(uptime: u64) -> String {
    let seconds = uptime;
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{} días, {} horas, {} minutos", days, hours, minutes)
    } else if hours > 0 {
        format!("{} horas, {} minutos", hours, minutes)
    } else if minutes > 0 {
        format!("{} minutos", minutes)
    } else {
        format!("{} segundos", seconds)
    }
}

/// Verifica si el proceso actual tiene permisos de administrador
fn is_admin() -> bool {
    Command::new("net")
        .args(["session"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let app_result = App::default().run(terminal);
    ratatui::restore();
    app_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_admin_returns_bool() {
        let _result = is_admin();
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(30), "30 segundos");
        assert_eq!(format_uptime(59), "59 segundos");
        assert_eq!(format_uptime(60), "1 minutos");
        assert_eq!(format_uptime(120), "2 minutos");
        assert_eq!(format_uptime(3540), "59 minutos");
        assert_eq!(format_uptime(3600), "1 horas, 0 minutos");
        assert_eq!(format_uptime(3661), "1 horas, 1 minutos");
        assert_eq!(format_uptime(86400), "1 días, 0 horas, 0 minutos");
        assert_eq!(format_uptime(90061), "1 días, 1 horas, 1 minutos");
    }

    #[test]
    fn test_app_default() {
        let app = App::default();
        assert_eq!(app.current_view, View::MainMenu);
        assert_eq!(app.selected_menu_item, 0);
        assert!(!app.should_quit);
        assert_eq!(app.operation_logs.len(), 0);
    }

    #[test]
    fn test_clean_stats_default() {
        let stats = CleanStats::default();
        assert_eq!(stats.deleted_count, 0);
        assert_eq!(stats.failed_count, 0);
        assert_eq!(stats.size_freed, 0);
    }
}
