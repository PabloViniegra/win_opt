use crate::theme::{ColorPalette, Theme};
use crate::types::{CleanStats, OperationState, View};
use crate::utils::format_uptime;
use crate::{cleanup, optimization};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
};
use sysinfo::{Disks, System};

/// Estructura principal de la aplicación
pub struct App {
    /// Vista actual
    pub current_view: View,
    /// Índice del item seleccionado en el menú
    pub selected_menu_item: usize,
    /// Logs de operaciones
    pub operation_logs: Vec<String>,
    /// Estado de la operación actual
    pub operation_state: OperationState,
    /// Estadísticas de la última limpieza
    pub clean_stats: CleanStats,
    /// Flag para salir de la aplicación
    pub should_quit: bool,
    /// Scroll vertical para logs
    pub scroll_offset: u16,
    /// Tema actual de la aplicación
    pub theme: Theme,
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
            theme: Theme::Dark, // Iniciar en modo oscuro
        }
    }
}

impl App {
    /// Obtiene la paleta de colores según el tema actual
    pub fn get_colors(&self) -> ColorPalette {
        ColorPalette::from_theme(self.theme)
    }

    /// Alterna entre tema claro y oscuro
    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
    }

    /// Ejecuta el loop principal de la aplicación
    pub fn run(mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
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
            View::BrowserCache => self.draw_browser_cache_view(frame),
            View::SystemLogs => self.draw_system_logs_view(frame),
            View::RecycleBin => self.draw_recycle_bin_view(frame),
            View::StartupOptimizer => self.draw_startup_optimizer_view(frame),
            View::VisualEffects => self.draw_visual_effects_view(frame),
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
                self.selected_menu_item = (self.selected_menu_item + 1).min(13);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_menu_item = self.selected_menu_item.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.operation_logs.clear();
                self.scroll_offset = 0;
                self.current_view = match self.selected_menu_item {
                    0 => {
                        cleanup::execute_clean(self);
                        View::Clean
                    }
                    1 => {
                        cleanup::execute_recycle_bin(self);
                        View::RecycleBin
                    }
                    2 => {
                        cleanup::execute_browser_cache(self);
                        View::BrowserCache
                    }
                    3 => {
                        cleanup::execute_system_logs(self);
                        View::SystemLogs
                    }
                    4 => {
                        optimization::execute_windows_update_cleanup(self);
                        View::WindowsUpdate
                    }
                    5 => {
                        optimization::execute_optimize(self);
                        View::Optimize
                    }
                    6 => {
                        optimization::execute_startup_optimizer(self);
                        View::StartupOptimizer
                    }
                    7 => {
                        optimization::execute_visual_effects(self);
                        View::VisualEffects
                    }
                    8 => {
                        optimization::execute_network(self);
                        View::Network
                    }
                    9 => {
                        optimization::execute_repair(self);
                        View::Repair
                    }
                    10 => {
                        optimization::execute_privacy(self);
                        View::Privacy
                    }
                    11 => View::Info,
                    12 => {
                        self.should_quit = true;
                        View::MainMenu
                    }
                    _ => View::MainMenu,
                };
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.toggle_theme();
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
            KeyCode::Tab => {
                self.toggle_theme();
            }
            _ => {}
        }
    }

    /// Dibuja el menú principal
    fn draw_main_menu(&mut self, frame: &mut Frame) {
        let colors = self.get_colors();
        let main_block = Block::default().style(Style::default().bg(colors.bg_main));
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

    /// Renderiza un banner suave con diseño pastel
    fn render_modern_banner(&self, frame: &mut Frame, area: Rect) {
        let colors = self.get_colors();
        let banner_lines = vec![
            Line::from(vec![
                Span::raw("  ╭───────────────────────────────────────────────────────╮  ")
                    .fg(colors.brand_primary),
            ]),
            Line::from(vec![
                Span::raw("  │  ").fg(colors.brand_primary),
                Span::raw("                                                     ")
                    .fg(colors.brand_secondary),
                Span::raw("  │  ").fg(colors.brand_primary),
            ]),
            Line::from(vec![
                Span::raw("  │  ").fg(colors.brand_primary),
                Span::raw("     ╭╮╮  ╭╮ ╭╮ ╭╮╮  ╮    ╭───╮ ╭───╮ ╭─────╮    ")
                    .fg(colors.brand_secondary)
                    .bold(),
                Span::raw("  │  ").fg(colors.brand_primary),
            ]),
            Line::from(vec![
                Span::raw("  │  ").fg(colors.brand_primary),
                Span::raw("     │││  ││ ││ │││╲ │    │   │ │   │   │      ")
                    .fg(colors.brand_secondary)
                    .bold(),
                Span::raw("  │  ").fg(colors.brand_primary),
            ]),
            Line::from(vec![
                Span::raw("  │  ").fg(colors.brand_primary),
                Span::raw("     │││╲╲││ ││ ││╲│││    │   │ ╰───╯   │      ")
                    .fg(colors.brand_accent)
                    .bold(),
                Span::raw("  │  ").fg(colors.brand_primary),
            ]),
            Line::from(vec![
                Span::raw("  │  ").fg(colors.brand_primary),
                Span::raw("     ╰──╯╰─ ╰╯ ╰╯ ╰─╯    ╰───╯  ╯       ╯      ")
                    .fg(colors.brand_accent)
                    .bold(),
                Span::raw("  │  ").fg(colors.brand_primary),
            ]),
            Line::from(vec![
                Span::raw("  │  ").fg(colors.brand_primary),
                Span::raw("                                                     ")
                    .fg(colors.brand_secondary),
                Span::raw("  │  ").fg(colors.brand_primary),
            ]),
            Line::from(vec![
                Span::raw("  │           ").fg(colors.brand_primary),
                Span::raw("✨ Windows 11 Optimizer ")
                    .fg(colors.text_primary)
                    .bold(),
                Span::raw("·").fg(colors.brand_accent),
                Span::raw(" v1.1.0 ").fg(colors.text_secondary),
                Span::raw("✨         │  ").fg(colors.brand_primary),
            ]),
            Line::from(vec![
                Span::raw("  │           ").fg(colors.brand_primary),
                Span::raw("🌸 Herramienta de optimización del sistema ")
                    .fg(colors.text_secondary)
                    .italic(),
                Span::raw("  🌸    │  ").fg(colors.brand_primary),
            ]),
            Line::from(vec![
                Span::raw("  │  ").fg(colors.brand_primary),
                Span::raw("                                                     ")
                    .fg(colors.brand_secondary),
                Span::raw("  │  ").fg(colors.brand_primary),
            ]),
            Line::from(vec![
                Span::raw("  ╰───────────────────────────────────────────────────────╯  ")
                    .fg(colors.brand_primary),
            ]),
        ];

        let banner_text = Text::from(banner_lines);
        let banner_widget = Paragraph::new(banner_text).alignment(Alignment::Center);
        frame.render_widget(banner_widget, area);
    }

    /// Renderiza el menú con diseño moderno
    fn render_modern_menu(&mut self, frame: &mut Frame, area: Rect) {
        let colors = self.get_colors();
        let menu_items = [
            // === LIBERACIÓN DE ESPACIO ===
            (
                "🧹",
                "Archivos Temporales",
                "Limpia archivos temp del sistema",
            ),
            (
                "🗑️",
                "Papelera de Reciclaje",
                "Vacía la papelera completamente",
            ),
            ("🌐", "Caché de Navegadores", "Limpia Chrome, Firefox, Edge"),
            ("📋", "Logs del Sistema", "Elimina archivos de registro"),
            ("🔄", "Windows Update", "Limpia archivos de actualización"),
            // === OPTIMIZACIÓN DE RENDIMIENTO ===
            (
                "⚡",
                "Optimización Avanzada",
                "Servicios, energía y prefetch",
            ),
            ("🚀", "Programas de Inicio", "Optimiza arranque de Windows"),
            ("🎨", "Efectos Visuales", "Deshabilita animaciones"),
            // === MANTENIMIENTO ===
            ("🌐", "Red", "DNS flush & Winsock reset"),
            ("🔧", "Reparación", "DISM & SFC scan"),
            ("🔒", "Privacidad", "Desactiva telemetría"),
            // === INFO Y SALIR ===
            ("💻", "Info del Sistema", "Detalles del hardware"),
            ("🚪", "Salir", "Cerrar aplicación"),
        ];

        let items: Vec<ListItem> = menu_items
            .iter()
            .enumerate()
            .map(|(i, (icon, title, desc))| {
                let is_selected = i == self.selected_menu_item;

                let (fg_color, icon_color, desc_color) = if is_selected {
                    (
                        colors.text_primary,
                        colors.brand_accent,
                        colors.text_primary,
                    )
                } else {
                    (
                        colors.text_primary,
                        colors.brand_primary,
                        colors.text_secondary,
                    )
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
                        .bg(colors.selection_bg) // Lavanda pastel claro para selección
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(content).style(style)
            })
            .collect();

        let menu_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.brand_primary))
            .border_set(symbols::border::ROUNDED)
            .title(Line::from(vec![
                Span::raw(" "),
                Span::raw("✨ ").fg(colors.brand_accent),
                Span::raw("Menú Principal ").fg(colors.text_primary).bold(),
            ]))
            .title_alignment(Alignment::Center);

        let menu_list = List::new(items).block(menu_block);
        frame.render_widget(menu_list, area);
    }

    /// Renderiza un footer moderno
    fn render_modern_footer(&self, frame: &mut Frame, area: Rect) {
        let colors = self.get_colors();
        let footer_text = Line::from(vec![
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("↑↓").fg(colors.brand_primary).bold(),
            Span::raw(" Navegar  ").fg(colors.text_secondary),
            Span::raw("•").fg(colors.brand_accent),
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("Enter").fg(colors.brand_primary).bold(),
            Span::raw(" Seleccionar  ").fg(colors.text_secondary),
            Span::raw("•").fg(colors.brand_accent),
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("Q/Esc").fg(colors.brand_primary).bold(),
            Span::raw(" Salir  ").fg(colors.text_secondary),
            Span::raw("•").fg(colors.brand_accent),
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("Tab").fg(colors.brand_primary).bold(),
            Span::raw(" Tema  ").fg(colors.text_secondary),
        ]);

        let footer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.brand_primary))
            .border_set(symbols::border::ROUNDED);

        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .block(footer_block);
        frame.render_widget(footer, area);
    }

    /// Dibuja la vista de limpieza con diseño mejorado
    fn draw_clean_view(&mut self, frame: &mut Frame) {
        let colors = self.get_colors();
        let main_block = Block::default().style(Style::default().bg(colors.bg_main));
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
            .border_style(Style::default().fg(colors.brand_primary))
            .border_set(symbols::border::ROUNDED);

        let title = Paragraph::new(Line::from(vec![
            Span::raw("🧹 ").fg(colors.brand_accent).bold(),
            Span::raw("Limpieza de Archivos Temporales")
                .fg(colors.text_primary)
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
        let colors = self.get_colors();
        let stats_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.success_color))
            .border_set(symbols::border::ROUNDED)
            .title(Line::from(vec![
                Span::raw(" "),
                Span::raw("📊 ").fg(colors.brand_accent),
                Span::raw("Estadísticas ").fg(colors.text_primary).bold(),
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
            Span::raw("  ✅  ").fg(colors.success_color).bold(),
            Span::raw("Elementos eliminados: ").fg(colors.text_secondary),
            Span::raw(self.clean_stats.deleted_count.to_string())
                .fg(colors.success_color)
                .bold(),
        ]);
        frame.render_widget(Paragraph::new(deleted_line), stats_chunks[0]);

        // Elementos omitidos
        let failed_line = Line::from(vec![
            Span::raw("  ⚠️  ").fg(colors.warning_color).bold(),
            Span::raw("Elementos omitidos: ").fg(colors.text_secondary),
            Span::raw(self.clean_stats.failed_count.to_string())
                .fg(colors.warning_color)
                .bold(),
        ]);
        frame.render_widget(Paragraph::new(failed_line), stats_chunks[1]);

        // Espacio liberado
        let size_mb = self.clean_stats.size_freed as f64 / 1024.0 / 1024.0;
        let freed_line = Line::from(vec![
            Span::raw("  💾  ").fg(colors.info_color).bold(),
            Span::raw("Espacio liberado: ").fg(colors.text_secondary),
            Span::raw(format!("{:.2} MB", size_mb))
                .fg(colors.brand_primary)
                .bold(),
        ]);
        frame.render_widget(Paragraph::new(freed_line), stats_chunks[2]);

        // Ayuda
        let help = Line::from(vec![
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("Q/Esc").fg(colors.brand_primary).bold(),
            Span::raw(" Volver  ").fg(colors.text_secondary),
            Span::raw("•").fg(colors.brand_accent),
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("↑↓").fg(colors.brand_primary).bold(),
            Span::raw(" Scroll  ").fg(colors.text_secondary),
            Span::raw("•").fg(colors.brand_accent),
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("Tab").fg(colors.brand_primary).bold(),
            Span::raw(" Tema  ").fg(colors.text_secondary),
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

    /// Dibuja la vista de limpieza de caché de navegadores
    fn draw_browser_cache_view(&mut self, frame: &mut Frame) {
        self.draw_generic_operation_view(frame, "🌐", "Caché de Navegadores");
    }

    /// Dibuja la vista de limpieza de logs del sistema
    fn draw_system_logs_view(&mut self, frame: &mut Frame) {
        self.draw_generic_operation_view(frame, "📋", "Logs del Sistema");
    }

    /// Dibuja la vista de vaciado de papelera de reciclaje
    fn draw_recycle_bin_view(&mut self, frame: &mut Frame) {
        self.draw_generic_operation_view(frame, "🗑️", "Papelera de Reciclaje");
    }

    /// Dibuja la vista de optimización de inicio
    fn draw_startup_optimizer_view(&mut self, frame: &mut Frame) {
        self.draw_generic_operation_view(frame, "🚀", "Programas de Inicio");
    }

    /// Dibuja la vista de efectos visuales
    fn draw_visual_effects_view(&mut self, frame: &mut Frame) {
        self.draw_generic_operation_view(frame, "🎨", "Efectos Visuales");
    }

    /// Dibuja una vista genérica de operación
    fn draw_generic_operation_view(&mut self, frame: &mut Frame, icon: &str, title: &str) {
        let colors = self.get_colors();
        let main_block = Block::default().style(Style::default().bg(colors.bg_main));
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
            .border_style(Style::default().fg(colors.brand_primary))
            .border_set(symbols::border::ROUNDED);

        let title_widget = Paragraph::new(Line::from(vec![
            Span::raw(format!("{} ", icon))
                .fg(colors.brand_accent)
                .bold(),
            Span::raw(title).fg(colors.text_primary).bold(),
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
        let colors = self.get_colors();
        let log_lines: Vec<Line> = self
            .operation_logs
            .iter()
            .map(|log| {
                // Colorear logs según contenido
                if log.contains("✅") {
                    Line::from(vec![Span::raw(log.as_str()).fg(colors.success_color)])
                } else if log.contains("⚠️") || log.contains("ℹ️") {
                    Line::from(vec![Span::raw(log.as_str()).fg(colors.warning_color)])
                } else if log.contains("❌") || log.contains("⛔") {
                    Line::from(vec![Span::raw(log.as_str()).fg(colors.error_color)])
                } else if log.contains("🧹")
                    || log.contains("🌐")
                    || log.contains("🔧")
                    || log.contains("⚡")
                    || log.contains("🔄")
                    || log.contains("🔒")
                {
                    Line::from(vec![
                        Span::raw(log.as_str()).fg(colors.brand_primary).bold(),
                    ])
                } else {
                    Line::from(vec![Span::raw(log.as_str()).fg(colors.text_primary)])
                }
            })
            .collect();

        let logs_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.brand_primary))
            .border_set(symbols::border::ROUNDED)
            .title(Line::from(vec![
                Span::raw(" "),
                Span::raw("📋 ").fg(colors.brand_accent),
                Span::raw(title).fg(colors.text_primary).bold(),
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
        let colors = self.get_colors();
        let footer_text = Line::from(vec![
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("Q/Esc").fg(colors.brand_primary).bold(),
            Span::raw(" Volver al menú  ").fg(colors.text_secondary),
            Span::raw("•").fg(colors.brand_accent),
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("↑↓").fg(colors.brand_primary).bold(),
            Span::raw(" Scroll  ").fg(colors.text_secondary),
            Span::raw("•").fg(colors.brand_accent),
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("Tab").fg(colors.brand_primary).bold(),
            Span::raw(" Tema  ").fg(colors.text_secondary),
        ]);

        let footer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.brand_primary))
            .border_set(symbols::border::ROUNDED);

        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .block(footer_block);
        frame.render_widget(footer, area);
    }

    /// Dibuja la vista de información del sistema con diseño mejorado
    fn draw_info_view(&mut self, frame: &mut Frame) {
        let colors = self.get_colors();
        let main_block = Block::default().style(Style::default().bg(colors.bg_main));
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
            .border_style(Style::default().fg(colors.brand_primary))
            .border_set(symbols::border::ROUNDED);

        let title = Paragraph::new(Line::from(vec![
            Span::raw("💻 ").fg(colors.brand_accent).bold(),
            Span::raw("Información del Sistema")
                .fg(colors.text_primary)
                .bold(),
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
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("Q/Esc").fg(colors.brand_primary).bold(),
            Span::raw(" Volver al menú  ").fg(colors.text_secondary),
        ]);

        let footer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.brand_primary))
            .border_set(symbols::border::ROUNDED);

        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .block(footer_block);
        frame.render_widget(footer, chunks[4]);
    }

    /// Renderiza información del OS
    fn render_os_info(&self, frame: &mut Frame, area: Rect, _sys: &System) {
        let colors = self.get_colors();
        let os_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.info_color))
            .border_set(symbols::border::ROUNDED)
            .title(Line::from(vec![
                Span::raw(" "),
                Span::raw("🖥️  ").fg(colors.brand_accent),
                Span::raw("Sistema Operativo ")
                    .fg(colors.text_primary)
                    .bold(),
            ]));

        let os_info = vec![
            Line::from(vec![
                Span::raw("  "),
                Span::raw("OS: ").fg(colors.brand_primary).bold(),
                Span::raw(System::name().unwrap_or_else(|| "Desconocido".to_string()))
                    .fg(colors.text_primary),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Versión: ").fg(colors.brand_primary).bold(),
                Span::raw(System::os_version().unwrap_or_else(|| "Desconocida".to_string()))
                    .fg(colors.text_primary),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Kernel: ").fg(colors.brand_primary).bold(),
                Span::raw(System::kernel_version().unwrap_or_else(|| "Desconocido".to_string()))
                    .fg(colors.text_primary),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Host: ").fg(colors.brand_primary).bold(),
                Span::raw(System::host_name().unwrap_or_else(|| "Desconocido".to_string()))
                    .fg(colors.text_primary),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Arquitectura: ").fg(colors.brand_primary).bold(),
                Span::raw(std::env::consts::ARCH).fg(colors.text_primary),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Tiempo activo: ").fg(colors.brand_primary).bold(),
                Span::raw(format_uptime(System::uptime())).fg(colors.success_color),
            ]),
        ];

        let os_widget = Paragraph::new(os_info).block(os_block);
        frame.render_widget(os_widget, area);
    }

    /// Renderiza información de CPU y memoria
    fn render_cpu_mem_info(&self, frame: &mut Frame, area: Rect, sys: &System) {
        let colors = self.get_colors();
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
            .border_style(Style::default().fg(colors.brand_secondary))
            .border_set(symbols::border::ROUNDED)
            .title(Line::from(vec![
                Span::raw(" "),
                Span::raw("⚡ ").fg(colors.brand_accent),
                Span::raw("CPU y Memoria ").fg(colors.text_primary).bold(),
            ]));

        let cpu_mem_info = vec![
            Line::from(vec![
                Span::raw("  "),
                Span::raw("CPU: ").fg(colors.brand_primary).bold(),
                Span::raw(cpu_brand).fg(colors.text_primary),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Núcleos: ").fg(colors.brand_primary).bold(),
                Span::raw(cpu_count.to_string()).fg(colors.text_primary),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Memoria Total: ").fg(colors.brand_primary).bold(),
                Span::raw(format!("{:.2} GB", total_memory)).fg(colors.text_primary),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw("Memoria Usada: ").fg(colors.brand_primary).bold(),
                Span::raw(format!("{:.2} GB", used_memory)).fg(colors.warning_color),
            ]),
        ];

        let cpu_mem_widget = Paragraph::new(cpu_mem_info).block(cpu_mem_block);
        frame.render_widget(cpu_mem_widget, area);
    }

    /// Renderiza información de almacenamiento
    fn render_storage_info(&self, frame: &mut Frame, area: Rect, sys: &System) {
        let colors = self.get_colors();
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
            colors.error_color
        } else if memory_percent > 70 {
            colors.warning_color
        } else {
            colors.success_color
        };

        let memory_gauge = Gauge::default()
            .block(
                Block::default()
                    .title(Line::from(vec![
                        Span::raw(" "),
                        Span::raw("💾 ").fg(colors.brand_accent),
                        Span::raw("Uso de Memoria ").fg(colors.text_primary).bold(),
                    ]))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(gauge_color))
                    .border_set(symbols::border::ROUNDED),
            )
            .gauge_style(Style::default().fg(gauge_color).bg(colors.bg_alt))
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
                    colors.error_color
                } else if usage_percent > 70.0 {
                    colors.warning_color
                } else {
                    colors.success_color
                };

                Line::from(vec![
                    Span::raw("  "),
                    Span::raw(format!("{}: ", disk.mount_point().to_string_lossy()))
                        .fg(colors.brand_primary)
                        .bold(),
                    Span::raw(format!("{:.1} GB / {:.1} GB ", used_space, total_space))
                        .fg(colors.text_primary),
                    Span::raw(format!("({:.1}%)", usage_percent))
                        .fg(color)
                        .bold(),
                ])
            })
            .collect();

        let disk_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.brand_accent))
            .border_set(symbols::border::ROUNDED)
            .title(Line::from(vec![
                Span::raw(" "),
                Span::raw("💿 ").fg(colors.brand_accent),
                Span::raw("Discos ").fg(colors.text_primary).bold(),
            ]));

        let disk_widget = Paragraph::new(disk_lines).block(disk_block);
        frame.render_widget(disk_widget, storage_chunks[1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_default() {
        let app = App::default();
        assert_eq!(app.current_view, View::MainMenu);
        assert_eq!(app.selected_menu_item, 0);
        assert!(!app.should_quit);
        assert_eq!(app.operation_logs.len(), 0);
    }
}
