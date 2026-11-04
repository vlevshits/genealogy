#[allow(unused_imports)]
use super::*;

pub(crate) const PT_TO_MM: f64 = 0.352778;

pub(crate) fn pt_to_mm(pt: f64) -> f64 {
    pt * PT_TO_MM
}

pub(crate) fn mm(v: f64) -> Mm {
    Mm(v as f32)
}

pub(crate) fn pt(v: f64) -> f32 {
    v as f32
}

pub(crate) fn to_printpdf_color(c: &PdfColor) -> Color {
    Color::Rgb(Rgb::new(c.r, c.g, c.b, None))
}

pub(crate) fn parse_hex_color(hex: &str) -> Option<PdfColor> {
    let s = hex.strip_prefix('#').unwrap_or(hex);
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&s[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&s[4..6], 16).ok()? as f32 / 255.0;
        Some(PdfColor { r, g, b })
    } else {
        None
    }
}

/// Assembles and renders the complete vector PDF.
pub fn render_pdf(
    layout: &CanvasLayout,
    tiling: &TilingPlan,
    theme: &PdfTheme,
) -> Result<Vec<u8>, AppError> {
    let sheet_w = tiling.target_sheet.width_mm;
    let sheet_h = tiling.target_sheet.height_mm;

    let (doc, first_page, first_layer) =
        PdfDocument::new(&layout.title, mm(sheet_w), mm(sheet_h), "Background");

    let reg_font = doc
        .add_external_font(std::io::Cursor::new(theme.regular_font_bytes))
        .map_err(|e| AppError::Internal(format!("Failed to embed regular font: {e}")))?;
    let bold_font = doc
        .add_external_font(std::io::Cursor::new(theme.bold_font_bytes))
        .map_err(|e| AppError::Internal(format!("Failed to embed bold font: {e}")))?;

    let mut current_page_idx = first_page;
    let mut current_layer_idx = first_layer;
    let mut is_first_page = true;

    // Optional Sheet 1 Assembly Guide (only in tiled mode if requested)
    if tiling.include_guide && !tiling.is_single_poster {
        let layer = doc.get_page(current_page_idx).get_layer(current_layer_idx);
        render_assembly_guide(
            &layer, layout, tiling, theme, sheet_w, sheet_h, &reg_font, &bold_font,
        );
        is_first_page = false;
    }

    if tiling.is_single_poster {
        let layer = if is_first_page {
            doc.get_page(current_page_idx).get_layer(current_layer_idx)
        } else {
            let (p, l) = doc.add_page(mm(sheet_w), mm(sheet_h), "Poster");
            doc.get_page(p).get_layer(l)
        };

        render_single_poster(
            &layer, layout, tiling, theme, sheet_w, sheet_h, &reg_font, &bold_font,
        );
    } else {
        // Tiled pages
        for tile in &tiling.tiles {
            let layer = if is_first_page {
                is_first_page = false;
                doc.get_page(current_page_idx).get_layer(current_layer_idx)
            } else {
                let (p, l) = doc.add_page(
                    mm(sheet_w),
                    mm(sheet_h),
                    format!("Sheet-{}", tile.sheet_index),
                );
                current_page_idx = p;
                current_layer_idx = l;
                doc.get_page(p).get_layer(l)
            };

            render_tile_sheet(
                &layer, tile, layout, tiling, theme, sheet_w, sheet_h, &reg_font, &bold_font,
            );
        }
    }

    doc.save_to_bytes()
        .map_err(|e| AppError::Internal(format!("Failed to serialize PDF: {e}")))
}

/// Renders Sheet 1 Assembly Guide for multi-page tiled posters.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_assembly_guide(
    layer: &printpdf::PdfLayerReference,
    layout: &CanvasLayout,
    tiling: &TilingPlan,
    theme: &PdfTheme,
    sheet_w: f64,
    sheet_h: f64,
    reg_font: &printpdf::IndirectFontRef,
    bold_font: &printpdf::IndirectFontRef,
) {
    // Background fill
    fill_rect(
        layer,
        0.0,
        0.0,
        sheet_w,
        sheet_h,
        sheet_h,
        &theme.bg_color,
        None,
    );

    // Decorative frame
    let frame_m = 12.0;
    stroke_rect(
        layer,
        frame_m,
        frame_m,
        sheet_w - 2.0 * frame_m,
        sheet_h - 2.0 * frame_m,
        sheet_h,
        &theme.card_border,
        1.0,
    );

    // Header banner
    let title_text = format!("{} — Assembly Guide", layout.title);
    layer.set_fill_color(to_printpdf_color(&theme.title_color));
    let y_title = sheet_h - 24.0 - pt_to_mm(18.0);
    layer.use_text(
        title_text,
        pt(18.0),
        mm(frame_m + 8.0),
        mm(y_title),
        bold_font,
    );

    let desc_text = format!(
        "This family tree spans across {} sheets ({} × {} grid) of {} paper. Total canvas: {} × {} mm.",
        tiling.tiles.len(),
        tiling.grid_cols,
        tiling.grid_rows,
        tiling.target_sheet.name,
        layout.width_mm as u32,
        layout.height_mm as u32
    );
    layer.set_fill_color(to_printpdf_color(&theme.text_secondary));
    let y_desc = sheet_h - 32.0 - pt_to_mm(10.0);
    layer.use_text(desc_text, pt(10.0), mm(frame_m + 8.0), mm(y_desc), reg_font);

    // Instructions Box
    let inst_x = frame_m + 8.0;
    let inst_y = 44.0;
    let inst_w = sheet_w - 2.0 * (frame_m + 8.0);
    let inst_h = 36.0;

    fill_rect(
        layer,
        inst_x,
        inst_y,
        inst_w,
        inst_h,
        sheet_h,
        &theme.card_bg,
        Some(&theme.card_border),
    );

    layer.set_fill_color(to_printpdf_color(&theme.text_primary));
    let step1_y = sheet_h - inst_y - 8.0 - pt_to_mm(9.0);
    layer.use_text(
        "1. Print all pages at 100% scale ('Actual Size', no scaling or fit-to-page).",
        pt(9.0),
        mm(inst_x + 6.0),
        mm(step1_y),
        reg_font,
    );

    let step2_y = sheet_h - inst_y - 16.0 - pt_to_mm(9.0);
    layer.use_text(
        "2. Trim along dashed cut lines using scissors or a hobby knife & metal ruler.",
        pt(9.0),
        mm(inst_x + 6.0),
        mm(step2_y),
        reg_font,
    );

    let step3_y = sheet_h - inst_y - 24.0 - pt_to_mm(9.0);
    let overlap_txt = format!(
        "3. Match corner crosshairs (+) and glue overlapping seams ({} mm bleed) from Row 1 down to Row {}.",
        tiling.overlap_mm as u32,
        tiling.grid_rows
    );
    layer.use_text(
        overlap_txt,
        pt(9.0),
        mm(inst_x + 6.0),
        mm(step3_y),
        reg_font,
    );

    // Mini bird's-eye layout grid diagram
    let grid_origin_x = inst_x + 12.0;
    let grid_origin_y = 90.0;
    let max_avail_w = inst_w - 24.0;
    let max_avail_h = sheet_h - grid_origin_y - 20.0;

    let cell_w = (max_avail_w / (tiling.grid_cols as f64)).min(48.0);
    let cell_h = (max_avail_h / (tiling.grid_rows as f64)).min(32.0);

    for r in 0..tiling.grid_rows {
        for c in 0..tiling.grid_cols {
            let sheet_num = r * tiling.grid_cols + c + 1;
            let cx = grid_origin_x + (c as f64) * (cell_w + 3.0);
            let cy = grid_origin_y + (r as f64) * (cell_h + 3.0);

            fill_rect(
                layer,
                cx,
                cy,
                cell_w,
                cell_h,
                sheet_h,
                &theme.banner_bg,
                Some(&theme.accent_color),
            );

            layer.set_fill_color(to_printpdf_color(&theme.title_color));
            let num_str = format!("Sheet {sheet_num}");
            let num_y = sheet_h - cy - (cell_h / 2.0) - pt_to_mm(9.0) / 2.0;
            layer.use_text(num_str, pt(9.0), mm(cx + 6.0), mm(num_y), bold_font);

            let coord_str = format!("R{}, C{}", r + 1, c + 1);
            let coord_y = sheet_h - cy - (cell_h / 2.0) + pt_to_mm(8.0);
            layer.set_fill_color(to_printpdf_color(&theme.text_secondary));
            layer.use_text(coord_str, pt(7.5), mm(cx + 6.0), mm(coord_y), reg_font);
        }
    }
}

/// Renders a single wide-format poster (A0, A1, A2, A3).
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_single_poster(
    layer: &printpdf::PdfLayerReference,
    layout: &CanvasLayout,
    _tiling: &TilingPlan,
    theme: &PdfTheme,
    sheet_w: f64,
    sheet_h: f64,
    reg_font: &printpdf::IndirectFontRef,
    bold_font: &printpdf::IndirectFontRef,
) {
    // Fill background
    fill_rect(
        layer,
        0.0,
        0.0,
        sheet_w,
        sheet_h,
        sheet_h,
        &theme.bg_color,
        None,
    );

    // Decorative frame
    let frame_m = 10.0;
    stroke_rect(
        layer,
        frame_m,
        frame_m,
        sheet_w - 2.0 * frame_m,
        sheet_h - 2.0 * frame_m,
        sheet_h,
        &theme.card_border,
        1.5,
    );

    // Scale canvas to fit inside poster
    let avail_w = sheet_w - 2.0 * frame_m - 10.0;
    let avail_h = sheet_h - 2.0 * frame_m - 10.0;
    let scale_x = avail_w / layout.width_mm;
    let scale_y = avail_h / layout.height_mm;
    let scale = scale_x.min(scale_y).min(1.2);

    let offset_x = frame_m + 5.0 + (avail_w - layout.width_mm * scale) / 2.0;
    let offset_y = frame_m + 5.0 + (avail_h - layout.height_mm * scale) / 2.0;

    // Render title banner
    let title_y = offset_y + 4.0;
    layer.set_fill_color(to_printpdf_color(&theme.title_color));
    let py_title = sheet_h - title_y - pt_to_mm(20.0 * scale);
    layer.use_text(
        &layout.title,
        pt(20.0 * scale),
        mm(offset_x + 10.0),
        mm(py_title),
        bold_font,
    );

    let sub_y = title_y + 10.0 * scale;
    layer.set_fill_color(to_printpdf_color(&theme.text_secondary));
    let py_sub = sheet_h - sub_y - pt_to_mm(10.0 * scale);
    layer.use_text(
        &layout.subtitle,
        pt(10.0 * scale),
        mm(offset_x + 10.0),
        mm(py_sub),
        reg_font,
    );

    // Render connectors
    for conn in &layout.connectors {
        render_connector_scaled(layer, conn, theme, offset_x, offset_y, scale, sheet_h);
    }

    // Render cards
    for card in &layout.cards {
        render_card_scaled(
            layer, card, theme, offset_x, offset_y, scale, sheet_h, reg_font, bold_font,
        );
    }
}

/// Renders a single tiled sheet with cut marks, overlap margins, and coordinates.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_tile_sheet(
    layer: &printpdf::PdfLayerReference,
    tile: &super::tiling::TileCell,
    layout: &CanvasLayout,
    tiling: &TilingPlan,
    theme: &PdfTheme,
    sheet_w: f64,
    sheet_h: f64,
    reg_font: &printpdf::IndirectFontRef,
    bold_font: &printpdf::IndirectFontRef,
) {
    let margin = tiling.margin_mm;

    // Background fill
    fill_rect(
        layer,
        0.0,
        0.0,
        sheet_w,
        sheet_h,
        sheet_h,
        &theme.bg_color,
        None,
    );

    let win_x = tile.canvas_window_x;
    let win_y = tile.canvas_window_y;
    let win_w = tile.window_width;
    let win_h = tile.window_height;

    // Content mapping helper
    let map_x = |cx: f64| margin + (cx - win_x);
    let map_y = |cy: f64| margin + (cy - win_y);

    // Title banner if in top row
    if tile.row == 0 && win_y <= BANNER_HEIGHT_MM {
        let title_y = map_y(8.0);
        if title_y < margin + win_h && title_y >= margin - 20.0 {
            layer.set_fill_color(to_printpdf_color(&theme.title_color));
            let py_title = sheet_h - title_y - pt_to_mm(16.0);
            layer.use_text(
                &layout.title,
                pt(16.0),
                mm(map_x(20.0)),
                mm(py_title),
                bold_font,
            );

            layer.set_fill_color(to_printpdf_color(&theme.text_secondary));
            let py_sub = sheet_h - title_y - 8.0 - pt_to_mm(9.0);
            layer.use_text(
                &layout.subtitle,
                pt(9.0),
                mm(map_x(20.0)),
                mm(py_sub),
                reg_font,
            );
        }
    }

    // Connectors intersecting this tile
    for conn in &layout.connectors {
        let intersects = conn.points.iter().any(|(x, y)| {
            *x >= win_x - 10.0
                && *x <= win_x + win_w + 10.0
                && *y >= win_y - 10.0
                && *y <= win_y + win_h + 10.0
        });

        if intersects && conn.points.len() >= 2 {
            let (outline_color, thickness) = if conn.color.as_deref() == Some("#d97706") {
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
            layer.set_outline_thickness(thickness as f32);

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
                let sx = map_x(*cx);
                let sy = map_y(*cy);
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
    }

    // Cards intersecting this tile
    for card in &layout.cards {
        let card_right = card.x + card.width;
        let card_bottom = card.y + card.height;

        let intersects = card_right >= win_x
            && card.x <= win_x + win_w
            && card_bottom >= win_y
            && card.y <= win_y + win_h;

        if intersects {
            let sx = map_x(card.x);
            let sy = map_y(card.y);
            render_card(layer, card, sx, sy, theme, sheet_h, reg_font, bold_font);
        }
    }

    // Assembly Crop Marks, Dashed Cut Lines, and Crosshairs
    if tiling.show_crop_marks {
        render_assembly_markers(layer, tile, margin, win_w, win_h, sheet_w, sheet_h, theme);
    }

    // Coordinate footer text outside printable margin
    layer.set_fill_color(to_printpdf_color(&theme.text_secondary));
    let footer_y = sheet_h - (margin / 2.0) - pt_to_mm(8.0) / 2.0;
    layer.use_text(
        &tile.footer_text,
        pt(8.0),
        mm(margin),
        mm(footer_y),
        reg_font,
    );
}
