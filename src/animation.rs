//! Módulo de animaciones y efectos visuales para win_opt
//!
//! Proporciona funciones para crear progress bars animados, spinners
//! y otros efectos visuales para mejorar la experiencia de usuario.

use std::time::{Duration, Instant};

/// Estado de una animación de spinner
pub struct Spinner {
    start_time: Instant,
    frame_duration: Duration,
}

impl Spinner {
    /// Crea un nuevo spinner
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            frame_duration: Duration::from_millis(100),
        }
    }

    /// Obtiene el frame actual del spinner
    pub fn frame(&self) -> &'static str {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let elapsed = self.start_time.elapsed();
        let frame_index = (elapsed.as_millis() / self.frame_duration.as_millis()) as usize;
        frames[frame_index % frames.len()]
    }

    /// Obtiene el frame de un spinner de puntos
    pub fn dots_frame(&self) -> &'static str {
        let frames = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
        let elapsed = self.start_time.elapsed();
        let frame_index = (elapsed.as_millis() / 80) as usize;
        frames[frame_index % frames.len()]
    }

    /// Obtiene el frame de un spinner de bloques
    pub fn blocks_frame(&self) -> &'static str {
        let frames = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃", "▂"];
        let elapsed = self.start_time.elapsed();
        let frame_index = (elapsed.as_millis() / 60) as usize;
        frames[frame_index % frames.len()]
    }

    /// Obtiene el frame de un spinner circular
    pub fn circle_frame(&self) -> &'static str {
        let frames = ["◐", "◓", "◑", "◒"];
        let elapsed = self.start_time.elapsed();
        let frame_index = (elapsed.as_millis() / 120) as usize;
        frames[frame_index % frames.len()]
    }

    /// Obtiene el frame de animación de carga
    pub fn loading_frame(&self) -> &'static str {
        let frames = ["⡀", "⠄", "⠂", "⠁", "⠈", "⠐", "⠠", "⢀"];
        let elapsed = self.start_time.elapsed();
        let frame_index = (elapsed.as_millis() / 90) as usize;
        frames[frame_index % frames.len()]
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

/// Genera una barra de progreso con caracteres Unicode
pub fn progress_bar(percentage: u16, width: usize) -> String {
    let filled = (width as f32 * (percentage as f32 / 100.0)) as usize;
    let empty = width.saturating_sub(filled);

    let filled_char = "█";
    let partial_chars = ["▏", "▎", "▍", "▌", "▋", "▊", "▉"];
    let empty_char = "░";

    // Calcular si hay un carácter parcial
    let partial_progress = (width as f32 * (percentage as f32 / 100.0)) - filled as f32;
    let partial_index = (partial_progress * partial_chars.len() as f32) as usize;

    let mut bar = filled_char.repeat(filled);
    
    if partial_index > 0 && partial_index < partial_chars.len() && empty > 0 {
        bar.push_str(partial_chars[partial_index]);
        bar.push_str(&empty_char.repeat(empty.saturating_sub(1)));
    } else {
        bar.push_str(&empty_char.repeat(empty));
    }

    bar
}

/// Genera un gráfico de barras vertical
pub fn vertical_bar_chart(value: f32, max_value: f32, height: usize) -> Vec<String> {
    let percentage = (value / max_value).min(1.0);
    let filled_height = (height as f32 * percentage) as usize;
    
    let mut lines = Vec::new();
    
    for i in (0..height).rev() {
        if i < filled_height {
            lines.push("█".to_string());
        } else {
            lines.push("░".to_string());
        }
    }
    
    lines
}

/// Genera un gráfico de línea simple
pub fn sparkline(values: &[f32]) -> String {
    if values.is_empty() {
        return String::new();
    }

    let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let max_value = values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    let min_value = values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let range = max_value - min_value;

    if range == 0.0 {
        return chars[4].to_string().repeat(values.len());
    }

    values
        .iter()
        .map(|&v| {
            let normalized = (v - min_value) / range;
            let index = (normalized * (chars.len() - 1) as f32) as usize;
            chars[index.min(chars.len() - 1)]
        })
        .collect()
}

/// Efectos de pulso para elementos
pub struct Pulse {
    start_time: Instant,
    duration: Duration,
}

impl Pulse {
    /// Crea un nuevo efecto de pulso
    pub fn new(duration_ms: u64) -> Self {
        Self {
            start_time: Instant::now(),
            duration: Duration::from_millis(duration_ms),
        }
    }

    /// Obtiene la opacidad actual del pulso (0.0 a 1.0)
    pub fn opacity(&self) -> f32 {
        let elapsed = self.start_time.elapsed().as_millis() as f32;
        let cycle = self.duration.as_millis() as f32;
        let phase = (elapsed % cycle) / cycle;
        
        // Función sinusoidal para pulso suave
        ((phase * std::f32::consts::PI * 2.0).sin() + 1.0) / 2.0
    }

    /// Indica si el pulso está en fase "brillante"
    pub fn is_bright(&self) -> bool {
        self.opacity() > 0.5
    }
}

impl Default for Pulse {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar() {
        let bar = progress_bar(50, 10);
        assert_eq!(bar.len() >= 10, true);
    }

    #[test]
    fn test_progress_bar_full() {
        let bar = progress_bar(100, 10);
        assert!(bar.contains("█"));
    }

    #[test]
    fn test_progress_bar_empty() {
        let bar = progress_bar(0, 10);
        assert!(bar.contains("░"));
    }

    #[test]
    fn test_sparkline() {
        let values = vec![1.0, 2.0, 3.0, 2.0, 1.0];
        let line = sparkline(&values);
        assert_eq!(line.chars().count(), 5);
    }

    #[test]
    fn test_sparkline_empty() {
        let values = vec![];
        let line = sparkline(&values);
        assert_eq!(line, "");
    }

    #[test]
    fn test_vertical_bar_chart() {
        let chart = vertical_bar_chart(50.0, 100.0, 5);
        assert_eq!(chart.len(), 5);
    }

    #[test]
    fn test_spinner_frames() {
        let spinner = Spinner::new();
        let frame = spinner.frame();
        assert!(!frame.is_empty());
    }

    #[test]
    fn test_pulse_opacity() {
        let pulse = Pulse::new(1000);
        let opacity = pulse.opacity();
        assert!(opacity >= 0.0 && opacity <= 1.0);
    }
}
