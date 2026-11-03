#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PdfColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl PdfColor {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }
}

pub static ROBOTO_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/Roboto-Regular.ttf");
pub static ROBOTO_BOLD: &[u8] = include_bytes!("../../../assets/fonts/Roboto-Bold.ttf");
pub static PTSERIF_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/PTSerif-Regular.ttf");
pub static PTSERIF_BOLD: &[u8] = include_bytes!("../../../assets/fonts/PTSerif-Bold.ttf");

#[derive(Debug, Clone)]
pub struct PdfTheme {
    pub name: String,
    pub regular_font_bytes: &'static [u8],
    pub bold_font_bytes: &'static [u8],
    pub bg_color: PdfColor,
    pub card_bg: PdfColor,
    pub card_border: PdfColor,
    pub card_focal_border: PdfColor,
    pub card_focal_bg: PdfColor,
    pub text_primary: PdfColor,
    pub text_secondary: PdfColor,
    pub connector_color: PdfColor,
    pub accent_color: PdfColor,
    pub crop_mark_color: PdfColor,
    pub title_color: PdfColor,
    pub banner_bg: PdfColor,
}

impl PdfTheme {
    pub fn modern() -> Self {
        Self {
            name: "modern".to_string(),
            regular_font_bytes: ROBOTO_REGULAR,
            bold_font_bytes: ROBOTO_BOLD,
            bg_color: PdfColor::new(0.97, 0.98, 0.99), // Subtle Slate White
            card_bg: PdfColor::new(1.0, 1.0, 1.0),     // Pure White
            card_border: PdfColor::new(0.80, 0.84, 0.88), // Sleek Slate border
            card_focal_border: PdfColor::new(0.15, 0.39, 0.92), // Vibrant Accent
            card_focal_bg: PdfColor::new(0.94, 0.97, 1.0), // Light Indigo Tint
            text_primary: PdfColor::new(0.06, 0.09, 0.16), // Dark Slate
            text_secondary: PdfColor::new(0.39, 0.45, 0.55), // Muted Slate
            connector_color: PdfColor::new(0.50, 0.58, 0.68), // 1px Slate
            accent_color: PdfColor::new(0.15, 0.39, 0.92), // Blue/Indigo
            crop_mark_color: PdfColor::new(0.60, 0.65, 0.70),
            title_color: PdfColor::new(0.06, 0.09, 0.16),
            banner_bg: PdfColor::new(0.92, 0.95, 0.98),
        }
    }

    pub fn heritage() -> Self {
        Self {
            name: "heritage".to_string(),
            regular_font_bytes: PTSERIF_REGULAR,
            bold_font_bytes: PTSERIF_BOLD,
            bg_color: PdfColor::new(0.98, 0.97, 0.93), // Aged Parchment / Warm Ivory
            card_bg: PdfColor::new(1.0, 0.99, 0.96),   // Antique Cream
            card_border: PdfColor::new(0.75, 0.62, 0.48), // Antique Gold / Sepia border
            card_focal_border: PdfColor::new(0.55, 0.15, 0.24), // Antique Burgundy
            card_focal_bg: PdfColor::new(0.97, 0.94, 0.90), // Warm Parchment Tint
            text_primary: PdfColor::new(0.18, 0.14, 0.12), // Deep Sepia / Charcoal
            text_secondary: PdfColor::new(0.48, 0.39, 0.33), // Warm Bronze
            connector_color: PdfColor::new(0.55, 0.37, 0.24), // Warm Sepia
            accent_color: PdfColor::new(0.55, 0.15, 0.24), // Antique Burgundy / Crimson
            crop_mark_color: PdfColor::new(0.65, 0.55, 0.48),
            title_color: PdfColor::new(0.18, 0.14, 0.12),
            banner_bg: PdfColor::new(0.94, 0.91, 0.84),
        }
    }

    pub fn from_name(name: &str) -> Self {
        if name.trim().eq_ignore_ascii_case("heritage") {
            Self::heritage()
        } else {
            Self::modern()
        }
    }
}
