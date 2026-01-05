use win_opt::App;

fn main() -> std::io::Result<()> {
    // Inicializar el sistema de logging
    if let Err(e) = win_opt::logger::init() {
        eprintln!("Error al inicializar el sistema de logging: {}", e);
        // Continuar la ejecución incluso si falla el logging
    }

    let terminal = ratatui::init();
    let app_result = App::default().run(terminal);
    ratatui::restore();
    app_result
}
