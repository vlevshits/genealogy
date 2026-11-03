#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SheetDimension {
    pub name: &'static str,
    pub width_mm: f64,
    pub height_mm: f64,
}

pub const SHEET_A4: SheetDimension = SheetDimension {
    name: "A4",
    width_mm: 297.0,
    height_mm: 210.0,
};
pub const SHEET_LETTER: SheetDimension = SheetDimension {
    name: "Letter",
    width_mm: 279.4,
    height_mm: 215.9,
};
pub const SHEET_A3: SheetDimension = SheetDimension {
    name: "A3",
    width_mm: 420.0,
    height_mm: 297.0,
};
pub const SHEET_A2: SheetDimension = SheetDimension {
    name: "A2",
    width_mm: 594.0,
    height_mm: 420.0,
};
pub const SHEET_A1: SheetDimension = SheetDimension {
    name: "A1",
    width_mm: 841.0,
    height_mm: 594.0,
};
pub const SHEET_A0: SheetDimension = SheetDimension {
    name: "A0",
    width_mm: 1189.0,
    height_mm: 841.0,
};

pub fn resolve_sheet_dimension(target: &str) -> SheetDimension {
    match target.trim().to_ascii_uppercase().as_str() {
        "LETTER" => SHEET_LETTER,
        "A3" => SHEET_A3,
        "A2" => SHEET_A2,
        "A1" => SHEET_A1,
        "A0" => SHEET_A0,
        _ => SHEET_A4,
    }
}

#[derive(Debug, Clone)]
pub struct TileCell {
    pub sheet_index: u32,
    pub row: u32,
    pub col: u32,
    pub total_rows: u32,
    pub total_cols: u32,
    pub canvas_window_x: f64,
    pub canvas_window_y: f64,
    pub window_width: f64,
    pub window_height: f64,
    pub overlap_left: bool,
    pub overlap_right: bool,
    pub overlap_top: bool,
    pub overlap_bottom: bool,
    pub footer_text: String,
}

#[derive(Debug, Clone)]
pub struct TilingPlan {
    pub target_sheet: SheetDimension,
    pub canvas_width_mm: f64,
    pub canvas_height_mm: f64,
    pub margin_mm: f64,
    pub overlap_mm: f64,
    pub grid_rows: u32,
    pub grid_cols: u32,
    pub total_sheets: u32,
    pub include_guide: bool,
    pub show_crop_marks: bool,
    pub is_single_poster: bool,
    pub tiles: Vec<TileCell>,
}

pub const DEFAULT_MARGIN_MM: f64 = 10.0;
pub const DEFAULT_OVERLAP_MM: f64 = 10.0;

/// Plans the tiling grid across the requested target sheets.
pub fn plan_tiling(
    canvas_w: f64,
    canvas_h: f64,
    target_sheet_name: &str,
    overlap_mm: f64,
    is_single_poster: bool,
    include_guide: bool,
    show_crop_marks: bool,
) -> TilingPlan {
    let target_sheet = resolve_sheet_dimension(target_sheet_name);
    let margin = DEFAULT_MARGIN_MM;

    if is_single_poster {
        let single_tile = TileCell {
            sheet_index: 1,
            row: 0,
            col: 0,
            total_rows: 1,
            total_cols: 1,
            canvas_window_x: 0.0,
            canvas_window_y: 0.0,
            window_width: canvas_w,
            window_height: canvas_h,
            overlap_left: false,
            overlap_right: false,
            overlap_top: false,
            overlap_bottom: false,
            footer_text: format!(
                "Single Poster ({} × {} mm) — {} Format",
                target_sheet.width_mm as u32, target_sheet.height_mm as u32, target_sheet.name
            ),
        };

        return TilingPlan {
            target_sheet,
            canvas_width_mm: canvas_w,
            canvas_height_mm: canvas_h,
            margin_mm: margin,
            overlap_mm: 0.0,
            grid_rows: 1,
            grid_cols: 1,
            total_sheets: 1,
            include_guide: false,
            show_crop_marks: false,
            is_single_poster: true,
            tiles: vec![single_tile],
        };
    }

    let overlap = overlap_mm.clamp(5.0, 25.0);
    let usable_w = (target_sheet.width_mm - 2.0 * margin).max(50.0);
    let usable_h = (target_sheet.height_mm - 2.0 * margin).max(50.0);

    let step_w = (usable_w - overlap).max(10.0);
    let step_h = (usable_h - overlap).max(10.0);

    let grid_cols = if canvas_w <= usable_w {
        1
    } else {
        ((canvas_w - usable_w) / step_w).ceil() as u32 + 1
    };

    let grid_rows = if canvas_h <= usable_h {
        1
    } else {
        ((canvas_h - usable_h) / step_h).ceil() as u32 + 1
    };

    let total_tiles = grid_rows * grid_cols;
    let mut tiles = Vec::with_capacity(total_tiles as usize);

    for r in 0..grid_rows {
        for c in 0..grid_cols {
            let sheet_index = r * grid_cols + c + 1;
            let canvas_window_x = (c as f64) * step_w;
            let canvas_window_y = (r as f64) * step_h;

            let overlap_left = c > 0;
            let overlap_right = c + 1 < grid_cols;
            let overlap_top = r > 0;
            let overlap_bottom = r + 1 < grid_rows;

            let paste_help = if overlap_left && overlap_top {
                format!(
                    " — Paste Right of Sheet {}, Below Sheet {}",
                    sheet_index - 1,
                    sheet_index - grid_cols
                )
            } else if overlap_left {
                format!(" — Paste Right of Sheet {}", sheet_index - 1)
            } else if overlap_top {
                format!(" — Paste Below Sheet {}", sheet_index - grid_cols)
            } else {
                String::new()
            };

            let footer_text = format!(
                "Sheet {} of {} [Row {}, Column {}]{}",
                sheet_index,
                total_tiles,
                r + 1,
                c + 1,
                paste_help
            );

            tiles.push(TileCell {
                sheet_index,
                row: r,
                col: c,
                total_rows: grid_rows,
                total_cols: grid_cols,
                canvas_window_x,
                canvas_window_y,
                window_width: usable_w,
                window_height: usable_h,
                overlap_left,
                overlap_right,
                overlap_top,
                overlap_bottom,
                footer_text,
            });
        }
    }

    let total_sheets = if include_guide {
        total_tiles + 1
    } else {
        total_tiles
    };

    TilingPlan {
        target_sheet,
        canvas_width_mm: canvas_w,
        canvas_height_mm: canvas_h,
        margin_mm: margin,
        overlap_mm: overlap,
        grid_rows,
        grid_cols,
        total_sheets,
        include_guide,
        show_crop_marks,
        is_single_poster: false,
        tiles,
    }
}
