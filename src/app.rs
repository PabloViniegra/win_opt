use crate::animation::{Spinner, progress_bar};
use crate::config::Config;
use crate::i18n::{I18n, I18nKey};
use crate::theme::{ColorPalette, Theme};
use crate::types::{CleanStats, OperationState, View, WorkerHandle, WorkerMessage};
use crate::utils::format_uptime;
use crate::{cleanup, optimization};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
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
    /// Sistema de internacionalización
    pub i18n: I18n,
    /// Configuración de la aplicación
    pub config: Config,
    /// Spinner para animaciones
    pub spinner: Spinner,
    /// Handle del worker thread actual (si hay alguno ejecutándose)
    pub worker_handle: Option<WorkerHandle>,
}

impl Default for App {
    fn default() -> Self {
        // Cargar configuración
        let config = Config::load();

        // Inicializar i18n con el idioma de la configuración
        let i18n = I18n::new(config.language());

        // Obtener tema de la configuración
        let theme = config.theme();

        Self {
            current_view: View::MainMenu,
            selected_menu_item: 0,
            operation_logs: Vec::new(),
            operation_state: OperationState::Idle,
            clean_stats: CleanStats::default(),
            should_quit: false,
            scroll_offset: 0,
            theme,
            i18n,
            config,
            spinner: Spinner::new(),
            worker_handle: None,
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
        // Actualizar configuración
        self.config.set_theme(self.theme);
        // Guardar si está configurado para recordar
        let _ = self.config.save_if_remember();
    }

    /// Alterna entre idiomas disponibles
    pub fn toggle_language(&mut self) {
        self.i18n.toggle_language();
        // Actualizar configuración
        self.config.set_language(self.i18n.current_language());
        // Guardar si está configurado para recordar
        let _ = self.config.save_if_remember();
    }

    /// Obtiene una traducción
    pub fn t(&self, key: I18nKey) -> &str {
        self.i18n.t(key)
    }

    /// Ejecuta el loop principal de la aplicación
    pub fn run(mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        while !self.should_quit {
            // Procesar mensajes del worker si hay uno activo
            self.process_worker_messages();

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        // Guardar configuración al salir
        if let Err(e) = self.config.save() {
            tracing::warn!("No se pudo guardar la configuración al salir: {}", e);
        }

        Ok(())
    }

    /// Procesa mensajes del worker thread
    ///
    /// Este método lee todos los mensajes disponibles del canal del worker
    /// sin bloquear, actualizando el estado de la aplicación según corresponda.
    fn process_worker_messages(&mut self) {
        let mut should_clear_worker = false;

        if let Some(ref handle) = self.worker_handle {
            // Procesar todos los mensajes disponibles (non-blocking)
            while let Ok(message) = handle.receiver.try_recv() {
                match message {
                    WorkerMessage::Log(log) => {
                        self.operation_logs.push(log);
                    }
                    WorkerMessage::StateChange(state) => {
                        self.operation_state = state;
                    }
                    WorkerMessage::StatsUpdate(stats) => {
                        self.clean_stats = stats;
                    }
                    WorkerMessage::Error(error) => {
                        self.operation_logs.push(format!("❌ ERROR: {}", error));
                    }
                    WorkerMessage::Completed => {
                        // Marcar para limpiar handle después del loop
                        should_clear_worker = true;
                    }
                }
            }
        }

        // Limpiar worker handle si recibimos el mensaje de Completed
        if should_clear_worker {
            self.worker_handle = None;
        }
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
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.toggle_language();
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
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.toggle_language();
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

    /// Renderiza un banner moderno y profesional
    fn render_modern_banner(&self, frame: &mut Frame, area: Rect) {
        let colors = self.get_colors();

        // Banner con diseño moderno y limpio
        let banner_lines = vec![
            // Línea superior con gradiente simulado
            Line::from(vec![
                Span::raw("  ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀  ")
                    .fg(colors.brand_primary)
                    .bold(),
            ]),
            Line::from(""),
            // Logo ASCII moderno
            Line::from(vec![
                Span::raw("           ██╗    ██╗██╗███╗   ██╗               ")
                    .fg(colors.brand_primary)
                    .bold(),
            ]),
            Line::from(vec![
                Span::raw("           ██║    ██║██║████╗  ██║               ")
                    .fg(colors.brand_primary)
                    .bold(),
            ]),
            Line::from(vec![
                Span::raw("           ██║ █╗ ██║██║██╔██╗ ██║               ")
                    .fg(colors.brand_secondary)
                    .bold(),
            ]),
            Line::from(vec![
                Span::raw("           ██║███╗██║██║██║╚██╗██║               ")
                    .fg(colors.brand_accent)
                    .bold(),
            ]),
            Line::from(vec![
                Span::raw("           ╚███╔███╔╝██║██║ ╚████║               ")
                    .fg(colors.brand_accent)
                    .bold(),
            ]),
            Line::from(vec![
                Span::raw("            ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝               ")
                    .fg(colors.brand_accent)
                    .bold(),
            ]),
            Line::from(""),
            // Subtítulo con badge
            Line::from(vec![
                Span::raw("               ╔══════════════════════════════════════╗")
                    .fg(colors.brand_secondary),
            ]),
            Line::from(vec![
                Span::raw("               ║  ").fg(colors.brand_secondary),
                Span::raw("⚡ ").fg(colors.brand_accent).bold(),
                Span::raw(self.t(I18nKey::AppSubtitle))
                    .fg(colors.text_primary)
                    .bold(),
                Span::raw("  ").fg(colors.brand_secondary),
                Span::raw("│").fg(colors.text_secondary),
                Span::raw("  ").fg(colors.brand_secondary),
                Span::raw(self.t(I18nKey::AppVersion))
                    .fg(colors.info_color)
                    .bold(),
                Span::raw("  ║").fg(colors.brand_secondary),
            ]),
            Line::from(vec![
                Span::raw("               ╚══════════════════════════════════════╝")
                    .fg(colors.brand_secondary),
            ]),
            Line::from(""),
            // Footer decorativo
            Line::from(vec![
                Span::raw("  ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄  ")
                    .fg(colors.brand_primary)
                    .bold(),
            ]),
        ];

        let banner_text = Text::from(banner_lines);
        let banner_widget = Paragraph::new(banner_text).alignment(Alignment::Center);
        frame.render_widget(banner_widget, area);
    }

    /// Renderiza el menú con diseño moderno y categorías
    fn render_modern_menu(&mut self, frame: &mut Frame, area: Rect) {
        let colors = self.get_colors();

        // Definir categorías y sus items
        let cleanup_label = match self.i18n.current_language() {
            crate::Language::Spanish => "LIMPIEZA",
            crate::Language::English => "CLEANUP",
        };
        let optimize_label = match self.i18n.current_language() {
            crate::Language::Spanish => "OPTIMIZACIÓN",
            crate::Language::English => "OPTIMIZATION",
        };
        let system_label = match self.i18n.current_language() {
            crate::Language::Spanish => "SISTEMA",
            crate::Language::English => "SYSTEM",
        };

        // Items con categorías
        let menu_data = vec![
            // CLEANUP
            ("", cleanup_label, "", Some(colors.success_color)),
            (
                "🧹",
                self.t(I18nKey::MenuTempFiles),
                self.t(I18nKey::MenuTempFilesDesc),
                None,
            ),
            (
                "🗑️",
                self.t(I18nKey::MenuRecycleBin),
                self.t(I18nKey::MenuRecycleBinDesc),
                None,
            ),
            (
                "🌐",
                self.t(I18nKey::MenuBrowserCache),
                self.t(I18nKey::MenuBrowserCacheDesc),
                None,
            ),
            (
                "📋",
                self.t(I18nKey::MenuSystemLogs),
                self.t(I18nKey::MenuSystemLogsDesc),
                None,
            ),
            (
                "🔄",
                self.t(I18nKey::MenuWindowsUpdate),
                self.t(I18nKey::MenuWindowsUpdateDesc),
                None,
            ),
            // OPTIMIZATION
            ("", optimize_label, "", Some(colors.warning_color)),
            (
                "⚡",
                self.t(I18nKey::MenuOptimize),
                self.t(I18nKey::MenuOptimizeDesc),
                None,
            ),
            (
                "🚀",
                self.t(I18nKey::MenuStartup),
                self.t(I18nKey::MenuStartupDesc),
                None,
            ),
            (
                "🎨",
                self.t(I18nKey::MenuVisualEffects),
                self.t(I18nKey::MenuVisualEffectsDesc),
                None,
            ),
            // SYSTEM
            ("", system_label, "", Some(colors.info_color)),
            (
                "🌐",
                self.t(I18nKey::MenuNetwork),
                self.t(I18nKey::MenuNetworkDesc),
                None,
            ),
            (
                "🔧",
                self.t(I18nKey::MenuRepair),
                self.t(I18nKey::MenuRepairDesc),
                None,
            ),
            (
                "🔒",
                self.t(I18nKey::MenuPrivacy),
                self.t(I18nKey::MenuPrivacyDesc),
                None,
            ),
            (
                "💻",
                self.t(I18nKey::MenuInfo),
                self.t(I18nKey::MenuInfoDesc),
                None,
            ),
            (
                "🚪",
                self.t(I18nKey::MenuExit),
                self.t(I18nKey::MenuExitDesc),
                None,
            ),
        ];

        // Mapeo de índice visual a índice real (sin contar headers)
        let visual_to_actual: Vec<usize> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let actual_to_visual: Vec<usize> = vec![1, 2, 3, 4, 5, 7, 8, 9, 11, 12, 13, 14, 15];

        let items: Vec<ListItem> = menu_data
            .iter()
            .enumerate()
            .map(|(visual_idx, (icon, title, desc, cat_color))| {
                // Si es una categoría (header)
                if let Some(color) = cat_color {
                    let content = Line::from(vec![
                        Span::raw("  "),
                        Span::raw("▌").fg(*color).bold(),
                        Span::raw(" "),
                        Span::raw(*title).fg(*color).bold(),
                        Span::raw(" "),
                        Span::raw("━".repeat(45)).fg(*color),
                    ]);
                    return ListItem::new(content)
                        .style(Style::default().add_modifier(Modifier::DIM));
                }

                // Item normal
                let actual_idx = visual_to_actual
                    .iter()
                    .position(|&v| actual_to_visual.get(v).is_some_and(|&av| av == visual_idx))
                    .unwrap_or(0);

                let is_selected = actual_idx == self.selected_menu_item;

                let content = if is_selected {
                    Line::from(vec![
                        Span::raw(" ▶ ").fg(colors.brand_accent).bold(),
                        Span::raw(*icon).fg(colors.brand_accent).bold(),
                        Span::raw("  "),
                        Span::raw(*title).fg(colors.text_primary).bold(),
                        Span::raw("  "),
                        Span::raw(format!("│ {}", desc))
                            .fg(colors.text_primary)
                            .italic(),
                    ])
                } else {
                    Line::from(vec![
                        Span::raw("   "),
                        Span::raw(*icon).fg(colors.brand_primary),
                        Span::raw("  "),
                        Span::raw(*title).fg(colors.text_primary),
                        Span::raw("  "),
                        Span::raw(format!("│ {}", desc))
                            .fg(colors.text_secondary)
                            .italic(),
                    ])
                };

                let style = if is_selected {
                    Style::default()
                        .bg(colors.selection_bg)
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
                Span::raw("◆ ").fg(colors.brand_accent).bold(),
                Span::raw(format!("{} ", self.t(I18nKey::MainMenu)))
                    .fg(colors.text_primary)
                    .bold(),
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
            Span::raw(format!(" {}  ", self.t(I18nKey::FooterNavigate))).fg(colors.text_secondary),
            Span::raw("•").fg(colors.brand_accent),
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("Enter").fg(colors.brand_primary).bold(),
            Span::raw(format!(" {}  ", self.t(I18nKey::FooterSelect))).fg(colors.text_secondary),
            Span::raw("•").fg(colors.brand_accent),
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("Q/Esc").fg(colors.brand_primary).bold(),
            Span::raw(format!(" {}  ", self.t(I18nKey::FooterExit))).fg(colors.text_secondary),
            Span::raw("•").fg(colors.brand_accent),
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("Tab").fg(colors.brand_primary).bold(),
            Span::raw(format!(" {}  ", self.t(I18nKey::FooterTheme))).fg(colors.text_secondary),
            Span::raw("•").fg(colors.brand_accent),
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("L").fg(colors.brand_primary).bold(),
            Span::raw(format!(" {}  ", self.t(I18nKey::FooterLanguage))).fg(colors.text_secondary),
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
            Span::raw(self.t(I18nKey::CleanTitle))
                .fg(colors.text_primary)
                .bold(),
        ]))
        .alignment(Alignment::Center)
        .block(title_block);
        frame.render_widget(title, chunks[0]);

        // Logs con diseño moderno
        self.render_styled_logs(frame, chunks[1], self.t(I18nKey::OperationsLog));

        // Estadísticas elegantes
        self.render_clean_stats(frame, chunks[2]);
    }

    /// Renderiza estadísticas de limpieza con diseño moderno
    fn render_clean_stats(&self, frame: &mut Frame, area: Rect) {
        let colors = self.get_colors();

        // Layout horizontal para las 3 estadísticas principales
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(area);

        // Card 1: Elementos eliminados
        let deleted_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.success_color).bold())
            .border_set(symbols::border::ROUNDED)
            .style(Style::default().bg(colors.bg_alt));

        let deleted_content = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("     "),
                Span::raw("✅").fg(colors.success_color).bold(),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("   "),
                Span::raw(self.clean_stats.deleted_count.to_string())
                    .fg(colors.success_color)
                    .bold()
                    .add_modifier(Modifier::UNDERLINED),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw(" "),
                Span::raw(self.t(I18nKey::StatsDeleted))
                    .fg(colors.text_secondary)
                    .italic(),
            ]),
        ];

        let deleted_widget = Paragraph::new(deleted_content)
            .block(deleted_block)
            .alignment(Alignment::Center);
        frame.render_widget(deleted_widget, main_layout[0]);

        // Card 2: Elementos omitidos
        let failed_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.warning_color).bold())
            .border_set(symbols::border::ROUNDED)
            .style(Style::default().bg(colors.bg_alt));

        let failed_content = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("     "),
                Span::raw("⚠️").fg(colors.warning_color).bold(),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("   "),
                Span::raw(self.clean_stats.failed_count.to_string())
                    .fg(colors.warning_color)
                    .bold()
                    .add_modifier(Modifier::UNDERLINED),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw(" "),
                Span::raw(self.t(I18nKey::StatsSkipped))
                    .fg(colors.text_secondary)
                    .italic(),
            ]),
        ];

        let failed_widget = Paragraph::new(failed_content)
            .block(failed_block)
            .alignment(Alignment::Center);
        frame.render_widget(failed_widget, main_layout[1]);

        // Card 3: Espacio liberado
        let size_mb = self.clean_stats.size_freed as f64 / 1024.0 / 1024.0;
        let freed_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.info_color).bold())
            .border_set(symbols::border::ROUNDED)
            .style(Style::default().bg(colors.bg_alt));

        let freed_content = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("     "),
                Span::raw("💾").fg(colors.info_color).bold(),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw(" "),
                Span::raw(format!("{:.2} MB", size_mb))
                    .fg(colors.info_color)
                    .bold()
                    .add_modifier(Modifier::UNDERLINED),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw(" "),
                Span::raw(self.t(I18nKey::StatsFreed))
                    .fg(colors.text_secondary)
                    .italic(),
            ]),
        ];

        let freed_widget = Paragraph::new(freed_content)
            .block(freed_block)
            .alignment(Alignment::Center);
        frame.render_widget(freed_widget, main_layout[2]);
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

        // Ajustar layout según si hay spinner o no
        let show_spinner = self.operation_state == OperationState::Running
            || self.operation_state == OperationState::Starting;

        let chunks = if show_spinner {
            Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3), // Título
                    Constraint::Length(3), // Spinner
                    Constraint::Min(7),    // Logs
                    Constraint::Length(3), // Footer
                ])
                .split(frame.area())
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3), // Título
                    Constraint::Min(10),   // Logs
                    Constraint::Length(3), // Footer
                ])
                .split(frame.area())
        };

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

        if show_spinner {
            // Spinner
            self.render_spinner(frame, chunks[1]);

            // Logs
            self.render_styled_logs(frame, chunks[2], "Registro de Operaciones");

            // Footer
            self.render_operation_footer(frame, chunks[3]);
        } else {
            // Logs
            self.render_styled_logs(frame, chunks[1], "Registro de Operaciones");

            // Footer
            self.render_operation_footer(frame, chunks[2]);
        }
    }

    /// Renderiza logs con estilo mejorado
    fn render_styled_logs(&self, frame: &mut Frame, area: Rect, title: &str) {
        let colors = self.get_colors();
        let log_lines: Vec<Line> = self
            .operation_logs
            .iter()
            .map(|log| {
                // Colorear logs según contenido (optimizado para reducir allocaciones)
                let span = if log.contains("✅") {
                    Span::raw(log.as_str()).fg(colors.success_color)
                } else if log.contains("⚠️") || log.contains("ℹ️") {
                    Span::raw(log.as_str()).fg(colors.warning_color)
                } else if log.contains("❌") || log.contains("⛔") {
                    Span::raw(log.as_str()).fg(colors.error_color)
                } else if log.contains("🧹")
                    || log.contains("🌐")
                    || log.contains("🔧")
                    || log.contains("⚡")
                    || log.contains("🔄")
                    || log.contains("🔒")
                {
                    Span::raw(log.as_str()).fg(colors.brand_primary).bold()
                } else {
                    Span::raw(log.as_str()).fg(colors.text_primary)
                };
                Line::from(span)
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
            Span::raw(format!(" {}  ", self.t(I18nKey::FooterBack))).fg(colors.text_secondary),
            Span::raw("•").fg(colors.brand_accent),
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("↑↓").fg(colors.brand_primary).bold(),
            Span::raw(format!(" {}  ", self.t(I18nKey::FooterScroll))).fg(colors.text_secondary),
            Span::raw("•").fg(colors.brand_accent),
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("Tab").fg(colors.brand_primary).bold(),
            Span::raw(format!(" {}  ", self.t(I18nKey::FooterTheme))).fg(colors.text_secondary),
            Span::raw("•").fg(colors.brand_accent),
            Span::raw("  ").fg(colors.brand_accent),
            Span::raw("L").fg(colors.brand_primary).bold(),
            Span::raw(format!(" {}  ", self.t(I18nKey::FooterLanguage))).fg(colors.text_secondary),
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

    /// Renderiza el spinner animado durante operaciones en curso
    ///
    /// Muestra un spinner animado con el mensaje "Operación en progreso..."
    /// cuando hay una operación ejecutándose en un worker thread.
    fn render_spinner(&self, frame: &mut Frame, area: Rect) {
        let colors = self.get_colors();

        // El spinner calcula automáticamente su frame basado en el tiempo transcurrido
        let spinner_text = Line::from(vec![
            Span::raw(self.spinner.frame())
                .fg(colors.brand_accent)
                .bold(),
            Span::raw(" Operación en progreso...").fg(colors.text_primary),
        ]);

        let spinner_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.warning_color))
            .border_set(symbols::border::ROUNDED);

        let spinner_widget = Paragraph::new(spinner_text)
            .alignment(Alignment::Center)
            .block(spinner_block);

        frame.render_widget(spinner_widget, area);
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
            Span::raw(self.t(I18nKey::InfoTitle))
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
            Span::raw(format!(" {}  ", self.t(I18nKey::FooterBack))).fg(colors.text_secondary),
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
                Span::raw(format!("{} ", self.t(I18nKey::InfoOs)))
                    .fg(colors.text_primary)
                    .bold(),
            ]));

        let unknown = match self.i18n.current_language() {
            crate::Language::Spanish => "Desconocido",
            crate::Language::English => "Unknown",
        };

        let os_info = vec![
            Line::from(vec![
                Span::raw("  "),
                Span::raw(format!("{} ", self.t(I18nKey::InfoOs)))
                    .fg(colors.brand_primary)
                    .bold(),
                Span::raw(System::name().unwrap_or_else(|| unknown.to_string()))
                    .fg(colors.text_primary),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw(format!("{} ", self.t(I18nKey::InfoVersion)))
                    .fg(colors.brand_primary)
                    .bold(),
                Span::raw(System::os_version().unwrap_or_else(|| unknown.to_string()))
                    .fg(colors.text_primary),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw(format!("{} ", self.t(I18nKey::InfoKernel)))
                    .fg(colors.brand_primary)
                    .bold(),
                Span::raw(System::kernel_version().unwrap_or_else(|| unknown.to_string()))
                    .fg(colors.text_primary),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw(format!("{} ", self.t(I18nKey::InfoHost)))
                    .fg(colors.brand_primary)
                    .bold(),
                Span::raw(System::host_name().unwrap_or_else(|| unknown.to_string()))
                    .fg(colors.text_primary),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw(format!("{} ", self.t(I18nKey::InfoArch)))
                    .fg(colors.brand_primary)
                    .bold(),
                Span::raw(std::env::consts::ARCH).fg(colors.text_primary),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw(format!("{} ", self.t(I18nKey::InfoUptime)))
                    .fg(colors.brand_primary)
                    .bold(),
                Span::raw(format_uptime(System::uptime())).fg(colors.success_color),
            ]),
        ];

        let os_widget = Paragraph::new(os_info).block(os_block);
        frame.render_widget(os_widget, area);
    }

    /// Renderiza información de CPU y memoria
    fn render_cpu_mem_info(&self, frame: &mut Frame, area: Rect, sys: &System) {
        let colors = self.get_colors();

        let unknown = match self.i18n.current_language() {
            crate::Language::Spanish => "Desconocido",
            crate::Language::English => "Unknown",
        };

        let cpu_count = sys.cpus().len();
        let cpu_brand = sys.cpus().first().map(|cpu| cpu.brand()).unwrap_or(unknown);

        let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_memory = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

        let cpu_mem_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.brand_secondary))
            .border_set(symbols::border::ROUNDED)
            .title(Line::from(vec![
                Span::raw(" "),
                Span::raw("⚡ ").fg(colors.brand_accent),
                Span::raw(format!("{} ", self.t(I18nKey::InfoCpu)))
                    .fg(colors.text_primary)
                    .bold(),
            ]));

        let cpu_mem_info = vec![
            Line::from(vec![
                Span::raw("  "),
                Span::raw(format!("{} ", self.t(I18nKey::InfoCpu)))
                    .fg(colors.brand_primary)
                    .bold(),
                Span::raw(cpu_brand).fg(colors.text_primary),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw(format!("{} ", self.t(I18nKey::InfoCores)))
                    .fg(colors.brand_primary)
                    .bold(),
                Span::raw(cpu_count.to_string()).fg(colors.text_primary),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::raw(format!("{} ", self.t(I18nKey::InfoMemTotal)))
                    .fg(colors.brand_primary)
                    .bold(),
                Span::raw(format!("{:.2} GB", total_memory)).fg(colors.text_primary),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw(format!("{} ", self.t(I18nKey::InfoMemUsed)))
                    .fg(colors.brand_primary)
                    .bold(),
                Span::raw(format!("{:.2} GB", used_memory)).fg(colors.warning_color),
            ]),
        ];

        let cpu_mem_widget = Paragraph::new(cpu_mem_info).block(cpu_mem_block);
        frame.render_widget(cpu_mem_widget, area);
    }

    /// Renderiza información de almacenamiento con gráficos visuales
    fn render_storage_info(&self, frame: &mut Frame, area: Rect, sys: &System) {
        let colors = self.get_colors();
        let storage_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(area);

        // Gauge de memoria con barra de progreso custom
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

        // Crear barra de progreso visual
        let bar = progress_bar(memory_percent, 40);

        let memory_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(gauge_color).bold())
            .border_set(symbols::border::ROUNDED)
            .title(Line::from(vec![
                Span::raw(" "),
                Span::raw("💾 ").fg(colors.brand_accent).bold(),
                Span::raw(format!("{} ", self.t(I18nKey::InfoMemUsage)))
                    .fg(colors.text_primary)
                    .bold(),
            ]));

        let memory_content = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::raw(bar).fg(gauge_color),
                Span::raw(format!("  {}%", memory_percent))
                    .fg(gauge_color)
                    .bold(),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::raw(format!("{:.1} GB / {:.1} GB", used_memory, total_memory))
                    .fg(colors.text_secondary),
            ]),
        ];

        let memory_widget = Paragraph::new(memory_content)
            .block(memory_block)
            .alignment(Alignment::Left);
        frame.render_widget(memory_widget, storage_chunks[0]);

        // Discos con barras de progreso
        let disks = Disks::new_with_refreshed_list();
        let mut disk_lines: Vec<Line> = vec![];

        for disk in disks.list() {
            let total_space = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
            let available_space = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0;
            let used_space = total_space - available_space;
            let usage_percent = if total_space > 0.0 {
                ((used_space / total_space) * 100.0) as u16
            } else {
                0
            };

            let color = if usage_percent > 90 {
                colors.error_color
            } else if usage_percent > 70 {
                colors.warning_color
            } else {
                colors.success_color
            };

            // Título del disco
            disk_lines.push(Line::from(""));
            disk_lines.push(Line::from(vec![
                Span::raw("  "),
                Span::raw(format!("💿 {}", disk.mount_point().to_string_lossy()))
                    .fg(colors.brand_primary)
                    .bold(),
            ]));

            // Barra de progreso del disco
            let disk_bar = progress_bar(usage_percent, 30);
            disk_lines.push(Line::from(vec![
                Span::raw("     "),
                Span::raw(disk_bar).fg(color),
                Span::raw(format!(" {}%", usage_percent)).fg(color).bold(),
            ]));

            // Info de espacio
            disk_lines.push(Line::from(vec![
                Span::raw("     "),
                Span::raw(format!("{:.1} GB / {:.1} GB", used_space, total_space))
                    .fg(colors.text_secondary)
                    .italic(),
            ]));
        }

        let disk_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.brand_accent))
            .border_set(symbols::border::ROUNDED)
            .title(Line::from(vec![
                Span::raw(" "),
                Span::raw("📊 ").fg(colors.brand_accent),
                Span::raw(format!("{} ", self.t(I18nKey::InfoDisks)))
                    .fg(colors.text_primary)
                    .bold(),
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
