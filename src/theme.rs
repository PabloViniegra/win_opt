use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Tema de la aplicación
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

/// Paleta de colores
#[derive(Debug, Clone, Copy)]
pub struct ColorPalette {
    pub brand_primary: Color,
    pub brand_secondary: Color,
    pub brand_accent: Color,
    pub success_color: Color,
    pub warning_color: Color,
    pub error_color: Color,
    pub info_color: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub bg_main: Color,
    pub bg_alt: Color,
    pub selection_bg: Color,
}

impl ColorPalette {
    /// Paleta de colores para modo claro (pasteles modernos y vibrantes)
    pub fn light() -> Self {
        Self {
            brand_primary: Color::Rgb(124, 58, 237), // Violeta vibrante (Purple 600)
            brand_secondary: Color::Rgb(168, 85, 247), // Púrpura brillante (Purple 500)
            brand_accent: Color::Rgb(236, 72, 153),  // Rosa fucsia (Pink 500)
            success_color: Color::Rgb(34, 197, 94),  // Verde esmeralda (Green 500)
            warning_color: Color::Rgb(245, 158, 11), // Ámbar dorado (Amber 500)
            error_color: Color::Rgb(239, 68, 68),    // Rojo coral (Red 500)
            info_color: Color::Rgb(59, 130, 246),    // Azul brillante (Blue 500)
            text_primary: Color::Rgb(30, 27, 75),    // Índigo oscuro profundo
            text_secondary: Color::Rgb(100, 116, 139), // Gris pizarra (Slate 500)
            bg_main: Color::Rgb(255, 255, 255),      // Blanco puro
            bg_alt: Color::Rgb(248, 250, 252),       // Gris casi blanco (Slate 50)
            selection_bg: Color::Rgb(243, 232, 255), // Violeta muy claro (Purple 100)
        }
    }

    /// Paleta de colores para modo oscuro (neón cyberpunk moderno)
    pub fn dark() -> Self {
        Self {
            brand_primary: Color::Rgb(139, 92, 246), // Violeta neón (Purple 500)
            brand_secondary: Color::Rgb(168, 85, 247), // Púrpura brillante (Purple 400)
            brand_accent: Color::Rgb(236, 72, 153),  // Rosa neón (Pink 500)
            success_color: Color::Rgb(52, 211, 153), // Verde agua neón (Emerald 400)
            warning_color: Color::Rgb(251, 191, 36), // Ámbar dorado (Amber 400)
            error_color: Color::Rgb(248, 113, 113),  // Rojo coral brillante (Red 400)
            info_color: Color::Rgb(96, 165, 250),    // Azul cielo neón (Blue 400)
            text_primary: Color::Rgb(241, 245, 249), // Casi blanco (Slate 100)
            text_secondary: Color::Rgb(148, 163, 184), // Gris pizarra (Slate 400)
            bg_main: Color::Rgb(15, 23, 42),         // Azul oscuro profundo (Slate 900)
            bg_alt: Color::Rgb(30, 41, 59),          // Azul oscuro medio (Slate 800)
            selection_bg: Color::Rgb(88, 28, 135),   // Púrpura oscuro intenso (Purple 900)
        }
    }

    /// Obtiene la paleta según el tema
    pub fn from_theme(theme: Theme) -> Self {
        match theme {
            Theme::Light => Self::light(),
            Theme::Dark => Self::dark(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_copy_and_equality() {
        let theme1 = Theme::Dark;
        let theme2 = theme1;
        assert_eq!(theme1, theme2);
    }

    #[test]
    fn test_light_and_dark_palettes_are_different() {
        let light = ColorPalette::light();
        let dark = ColorPalette::dark();

        // Verificar que los colores de fondo son diferentes
        assert_ne!(light.bg_main, dark.bg_main);
        assert_ne!(light.bg_alt, dark.bg_alt);

        // Verificar que los colores primarios son diferentes
        assert_ne!(light.brand_primary, dark.brand_primary);
        assert_ne!(light.text_primary, dark.text_primary);
    }

    #[test]
    fn test_from_theme_consistency() {
        // Verificar que from_theme(Light) == light()
        let light1 = ColorPalette::from_theme(Theme::Light);
        let light2 = ColorPalette::light();

        assert_eq!(light1.bg_main, light2.bg_main);
        assert_eq!(light1.brand_primary, light2.brand_primary);

        // Verificar que from_theme(Dark) == dark()
        let dark1 = ColorPalette::from_theme(Theme::Dark);
        let dark2 = ColorPalette::dark();

        assert_eq!(dark1.bg_main, dark2.bg_main);
        assert_eq!(dark1.brand_primary, dark2.brand_primary);
    }

    #[test]
    fn test_color_palette_has_all_required_colors() {
        let palette = ColorPalette::light();

        // Verificar que todos los campos están inicializados
        // (esto es más para documentación que funcionalidad)
        let _brand_primary = palette.brand_primary;
        let _brand_secondary = palette.brand_secondary;
        let _brand_accent = palette.brand_accent;
        let _success = palette.success_color;
        let _warning = palette.warning_color;
        let _error = palette.error_color;
        let _info = palette.info_color;
        let _text_primary = palette.text_primary;
        let _text_secondary = palette.text_secondary;
        let _bg_main = palette.bg_main;
        let _bg_alt = palette.bg_alt;
        let _selection = palette.selection_bg;
    }
}
