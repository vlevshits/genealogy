#[allow(unused_imports)]
use super::*;

/// Renders assembly crop marks, dashed cut lines, and corner crosshairs.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_assembly_markers(
    layer: &printpdf::PdfLayerReference,
    tile: &super::tiling::TileCell,
    margin: f64,
    win_w: f64,
    win_h: f64,
    _sheet_w: f64,
    sheet_h: f64,
    theme: &PdfTheme,
) {
    let x0 = margin;
    let y0 = margin;
    let x1 = margin + win_w;
    let y1 = margin + win_h;

    // Corner crosshairs (+) at the 4 bounds
    render_crosshair(layer, x0, y0, sheet_h, theme);
    render_crosshair(layer, x1, y0, sheet_h, theme);
    render_crosshair(layer, x0, y1, sheet_h, theme);
    render_crosshair(layer, x1, y1, sheet_h, theme);

    // Dashed cut lines on overlapping seams
    let dash = LineDashPattern {
        dash_1: Some(4),
        gap_1: Some(2),
        ..Default::default()
    };
    layer.set_line_dash_pattern(dash);
    layer.set_outline_color(to_printpdf_color(&theme.crop_mark_color));
    layer.set_outline_thickness(0.5);

    if tile.overlap_left {
        let px0 = x0;
        let py0 = sheet_h - y0;
        let py1 = sheet_h - y1;
        layer.add_line(Line {
            points: vec![
                (Point::new(mm(px0), mm(py0)), false),
                (Point::new(mm(px0), mm(py1)), false),
            ],
            is_closed: false,
        });
    }

    if tile.overlap_top {
        let px0 = x0;
        let px1 = x1;
        let py0 = sheet_h - y0;
        layer.add_line(Line {
            points: vec![
                (Point::new(mm(px0), mm(py0)), false),
                (Point::new(mm(px1), mm(py0)), false),
            ],
            is_closed: false,
        });
    }

    // Reset dash pattern to solid
    layer.set_line_dash_pattern(LineDashPattern::default());
}

pub(crate) fn render_crosshair(
    layer: &printpdf::PdfLayerReference,
    cx: f64,
    cy: f64,
    sheet_h: f64,
    theme: &PdfTheme,
) {
    layer.set_outline_color(to_printpdf_color(&theme.crop_mark_color));
    layer.set_outline_thickness(0.5);

    let py = sheet_h - cy;
    let arm = 2.5;

    // Horizontal arm
    layer.add_line(Line {
        points: vec![
            (Point::new(mm(cx - arm), mm(py)), false),
            (Point::new(mm(cx + arm), mm(py)), false),
        ],
        is_closed: false,
    });

    // Vertical arm
    layer.add_line(Line {
        points: vec![
            (Point::new(mm(cx), mm(py - arm)), false),
            (Point::new(mm(cx), mm(py + arm)), false),
        ],
        is_closed: false,
    });
}

/// Renders a single person card with avatar photo, name, and lifespan.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_card(
    layer: &printpdf::PdfLayerReference,
    card: &RenderPersonCard,
    x: f64,
    y: f64,
    theme: &PdfTheme,
    sheet_h: f64,
    reg_font: &printpdf::IndirectFontRef,
    bold_font: &printpdf::IndirectFontRef,
) {
    let (bg, border) = if card.is_focal {
        (&theme.card_focal_bg, &theme.card_focal_border)
    } else {
        (&theme.card_bg, &theme.card_border)
    };

    // Card background & border
    fill_rect(
        layer,
        x,
        y,
        card.width,
        card.height,
        sheet_h,
        bg,
        Some(border),
    );

    let mut text_x = x + 4.0;

    // Avatar portrait photo if available
    if let Some(ref bytes) = card.avatar_bytes {
        if let Ok(dyn_img) = image::load_from_memory(bytes) {
            let thumb = dyn_img
                .resize_to_fill(100, 100, image::imageops::FilterType::Triangle)
                .to_rgb8();
            let (tw, th) = (thumb.width(), thumb.height());
            let photo_size_mm = 16.0;

            let xobj = ImageXObject {
                width: Px(tw as usize),
                height: Px(th as usize),
                color_space: ColorSpace::Rgb,
                bits_per_component: ColorBits::Bit8,
                image_data: thumb.into_raw(),
                image_filter: None,
                interpolate: true,
                smask: None,
                clipping_bbox: None,
            };

            let pdf_image = Image::from(xobj);
            let mut transform = ImageTransform::default();

            // 1 unit in PDF = 1 pt = 0.352778 mm
            let scale_factor = photo_size_mm / ((tw as f64) * PT_TO_MM);
            transform.scale_x = Some(scale_factor as f32);
            transform.scale_y = Some(scale_factor as f32);

            let photo_x = x + 3.0;
            let photo_y = y + 5.0;
            let photo_y_pdf = sheet_h - photo_y - photo_size_mm;

            transform.translate_x = Some(mm(photo_x));
            transform.translate_y = Some(mm(photo_y_pdf));

            pdf_image.add_to_layer(layer.clone(), transform);
            text_x = photo_x + photo_size_mm + 3.0;
        }
    }

    // Full Name
    layer.set_fill_color(to_printpdf_color(&theme.text_primary));
    let name_font_size = if card.full_name.len() > 22 { 7.5 } else { 8.5 };
    let py_name = sheet_h - y - 6.0 - pt_to_mm(name_font_size);
    layer.use_text(
        &card.full_name,
        pt(name_font_size),
        mm(text_x),
        mm(py_name),
        bold_font,
    );

    // Lifespan / Living text
    if !card.lifespan.is_empty() {
        layer.set_fill_color(to_printpdf_color(&theme.text_secondary));
        let py_life = sheet_h - y - 14.5 - pt_to_mm(7.0);
        layer.use_text(&card.lifespan, pt(7.0), mm(text_x), mm(py_life), reg_font);
    }
}

/// Renders a scaled card for single-poster mode.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_card_scaled(
    layer: &printpdf::PdfLayerReference,
    card: &RenderPersonCard,
    theme: &PdfTheme,
    offset_x: f64,
    offset_y: f64,
    scale: f64,
    sheet_h: f64,
    reg_font: &printpdf::IndirectFontRef,
    bold_font: &printpdf::IndirectFontRef,
) {
    let sx = offset_x + card.x * scale;
    let sy = offset_y + card.y * scale;
    let sw = card.width * scale;
    let sh = card.height * scale;

    let (bg, border) = if card.is_focal {
        (&theme.card_focal_bg, &theme.card_focal_border)
    } else {
        (&theme.card_bg, &theme.card_border)
    };

    fill_rect(layer, sx, sy, sw, sh, sheet_h, bg, Some(border));

    let mut text_x = sx + 3.0 * scale;

    // Avatar portrait photo if available in single-poster mode
    if let Some(ref bytes) = card.avatar_bytes {
        if let Ok(dyn_img) = image::load_from_memory(bytes) {
            let thumb = dyn_img
                .resize_to_fill(100, 100, image::imageops::FilterType::Triangle)
                .to_rgb8();
            let (tw, th) = (thumb.width(), thumb.height());
            let photo_size_mm = 16.0 * scale;

            let xobj = ImageXObject {
                width: Px(tw as usize),
                height: Px(th as usize),
                color_space: ColorSpace::Rgb,
                bits_per_component: ColorBits::Bit8,
                image_data: thumb.into_raw(),
                image_filter: None,
                interpolate: true,
                smask: None,
                clipping_bbox: None,
            };

            let pdf_image = Image::from(xobj);
            let mut transform = ImageTransform::default();

            let scale_factor = photo_size_mm / ((tw as f64) * PT_TO_MM);
            transform.scale_x = Some(scale_factor as f32);
            transform.scale_y = Some(scale_factor as f32);

            let photo_x = sx + 3.0 * scale;
            let photo_y = sy + 5.0 * scale;
            let photo_y_pdf = sheet_h - photo_y - photo_size_mm;

            transform.translate_x = Some(mm(photo_x));
            transform.translate_y = Some(mm(photo_y_pdf));

            pdf_image.add_to_layer(layer.clone(), transform);
            text_x = photo_x + photo_size_mm + 3.0 * scale;
        }
    }

    layer.set_fill_color(to_printpdf_color(&theme.text_primary));
    let font_size = 8.0 * scale;
    let py_name = sheet_h - sy - 5.0 * scale - pt_to_mm(font_size);
    layer.use_text(
        &card.full_name,
        pt(font_size),
        mm(text_x),
        mm(py_name),
        bold_font,
    );

    if !card.lifespan.is_empty() {
        layer.set_fill_color(to_printpdf_color(&theme.text_secondary));
        let life_size = 6.5 * scale;
        let py_life = sheet_h - sy - 13.0 * scale - pt_to_mm(life_size);
        layer.use_text(
            &card.lifespan,
            pt(life_size),
            mm(text_x),
            mm(py_life),
            reg_font,
        );
    }
}

pub(crate) fn render_connector_scaled(
    layer: &printpdf::PdfLayerReference,
    conn: &RenderConnector,
    theme: &PdfTheme,
    offset_x: f64,
    offset_y: f64,
    scale: f64,
    sheet_h: f64,
) {
    if conn.points.len() < 2 {
        return;
    }

    let (outline_color, base_thickness) = if conn.color.as_deref() == Some("#d97706") {
        (
            PdfColor {
                r: 217.0 / 255.0,
                g: 119.0 / 255.0,
                b: 6.0 / 255.0,
            },
            1.5,
        )
    } else if let Some(ref hex) = conn.color {
        (parse_hex_color(hex).unwrap_or(theme.connector_color), 1.5)
    } else {
        (theme.connector_color, 0.75)
    };

    layer.set_outline_color(to_printpdf_color(&outline_color));
    layer.set_outline_thickness(pt(base_thickness * scale));
    layer.set_line_cap_style(LineCapStyle::Round);
    layer.set_line_join_style(LineJoinStyle::Round);

    if conn.is_dashed {
        layer.set_line_dash_pattern(LineDashPattern {
            dash_1: Some(4),
            gap_1: Some(2),
            ..Default::default()
        });
    } else {
        layer.set_line_dash_pattern(LineDashPattern::default());
    }

    let mut pdf_points = Vec::with_capacity(conn.points.len());
    for (cx, cy) in &conn.points {
        let sx = offset_x + *cx * scale;
        let sy = offset_y + *cy * scale;
        let py = sheet_h - sy;
        pdf_points.push((Point::new(mm(sx), mm(py)), false));
    }

    layer.add_line(Line {
        points: pdf_points,
        is_closed: false,
    });

    if conn.is_dashed {
        layer.set_line_dash_pattern(LineDashPattern::default());
    }
}

/// Helper: fills a rectangle with optional border outline.
#[allow(clippy::too_many_arguments)]
pub(crate) fn fill_rect(
    layer: &printpdf::PdfLayerReference,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    sheet_h: f64,
    fill_color: &PdfColor,
    border_color: Option<&PdfColor>,
) {
    let x0 = x;
    let x1 = x + w;
    let y0 = sheet_h - y - h;
    let y1 = sheet_h - y;

    layer.set_fill_color(to_printpdf_color(fill_color));

    let mode = if let Some(bc) = border_color {
        layer.set_outline_color(to_printpdf_color(bc));
        layer.set_outline_thickness(0.75);
        printpdf::path::PaintMode::FillStroke
    } else {
        printpdf::path::PaintMode::Fill
    };

    let points = vec![
        (Point::new(mm(x0), mm(y0)), false),
        (Point::new(mm(x1), mm(y0)), false),
        (Point::new(mm(x1), mm(y1)), false),
        (Point::new(mm(x0), mm(y1)), false),
    ];

    layer.add_polygon(Polygon {
        rings: vec![points],
        mode,
        winding_order: printpdf::path::WindingOrder::NonZero,
    });
}

/// Helper: strokes a rectangle outline without fill.
#[allow(clippy::too_many_arguments)]
pub(crate) fn stroke_rect(
    layer: &printpdf::PdfLayerReference,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    sheet_h: f64,
    border_color: &PdfColor,
    thickness: f64,
) {
    let x0 = x;
    let x1 = x + w;
    let y0 = sheet_h - y - h;
    let y1 = sheet_h - y;

    layer.set_outline_color(to_printpdf_color(border_color));
    layer.set_outline_thickness(pt(thickness));

    let points = vec![
        (Point::new(mm(x0), mm(y0)), false),
        (Point::new(mm(x1), mm(y0)), false),
        (Point::new(mm(x1), mm(y1)), false),
        (Point::new(mm(x0), mm(y1)), false),
    ];

    layer.add_polygon(Polygon {
        rings: vec![points],
        mode: printpdf::path::PaintMode::Stroke,
        winding_order: printpdf::path::WindingOrder::NonZero,
    });
}
