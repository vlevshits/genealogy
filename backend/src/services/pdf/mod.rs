pub mod layout;
pub mod render;
pub mod themes;
pub mod tiling;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use self::layout::compute_tree_layout;
use self::render::render_pdf;
use self::themes::PdfTheme;
use self::tiling::plan_tiling;
use crate::error::AppError;
use crate::services::privacy::GenealogyPrivacyStore;

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct PdfExportRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focal_person_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generations: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_sheet_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlap_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_crop_marks: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_photos: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redact_living: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_partner: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_siblings: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collateral: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_assembly_guide: Option<bool>,
}

impl PdfExportRequest {
    pub fn is_empty(&self) -> bool {
        self.focal_person_id.is_none()
            && self.view_mode.is_none()
            && self.generations.is_none()
            && self.theme.is_none()
            && self.format.is_none()
            && self.target_sheet_size.is_none()
            && self.overlap_mm.is_none()
            && self.show_crop_marks.is_none()
            && self.show_photos.is_none()
            && self.redact_living.is_none()
            && self.title.is_none()
            && self.subtitle.is_none()
            && self.include_partner.is_none()
            && self.include_siblings.is_none()
            && self.collateral.is_none()
            && self.include_assembly_guide.is_none()
    }

    pub fn view_mode(&self) -> &str {
        self.view_mode.as_deref().unwrap_or("pedigree")
    }

    pub fn generations(&self) -> u32 {
        self.generations.unwrap_or(5)
    }

    pub fn theme(&self) -> &str {
        self.theme.as_deref().unwrap_or("modern")
    }

    pub fn format(&self) -> &str {
        self.format.as_deref().unwrap_or("tiled")
    }

    pub fn target_sheet_size(&self) -> &str {
        self.target_sheet_size.as_deref().unwrap_or("A4")
    }

    pub fn overlap_mm(&self) -> f64 {
        self.overlap_mm.unwrap_or(10.0)
    }

    pub fn show_crop_marks(&self) -> bool {
        self.show_crop_marks.unwrap_or(true)
    }

    pub fn show_photos(&self) -> bool {
        self.show_photos.unwrap_or(true)
    }

    pub fn redact_living(&self) -> bool {
        self.redact_living.unwrap_or(false)
    }

    pub fn include_partner(&self) -> bool {
        self.include_partner.unwrap_or(true)
    }

    pub fn include_siblings(&self) -> bool {
        self.include_siblings.unwrap_or(false)
    }

    pub fn collateral(&self) -> bool {
        self.collateral.unwrap_or(true)
    }

    pub fn include_assembly_guide(&self) -> bool {
        self.include_assembly_guide.unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PdfPreviewCardDto {
    pub id: String,
    pub gid: String,
    pub full_name: String,
    pub lifespan: String,
    pub is_living: bool,
    pub is_focal: bool,
    pub sex: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub generation: i32,
    pub has_avatar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PdfPreviewConnectorDto {
    pub points: Vec<[f64; 2]>,
    pub is_dashed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PdfPosterLayoutDto {
    pub sheet_width_mm: f64,
    pub sheet_height_mm: f64,
    pub scale: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub frame_margin: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PdfTileSheetDto {
    pub sheet_index: u32,
    pub row: u32,
    pub col: u32,
    pub canvas_window_x: f64,
    pub canvas_window_y: f64,
    pub window_width: f64,
    pub window_height: f64,
    pub has_cut_right: bool,
    pub has_cut_bottom: bool,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PdfExportPreviewInfo {
    pub total_sheets: u32,
    pub grid_rows: u32,
    pub grid_cols: u32,
    pub canvas_width_mm: f64,
    pub canvas_height_mm: f64,
    pub sheet_width_mm: f64,
    pub sheet_height_mm: f64,
    pub overlap_mm: f64,
    pub format: String,
    pub target_sheet_size: String,
    pub total_persons: usize,
    pub summary_text: String,
    pub title: String,
    pub subtitle: String,
    pub cards: Vec<PdfPreviewCardDto>,
    pub connectors: Vec<PdfPreviewConnectorDto>,
    pub poster_layout: Option<PdfPosterLayoutDto>,
    pub tiles: Vec<PdfTileSheetDto>,
}

fn resolve_focal_id<'a>(
    store: &'a GenealogyPrivacyStore,
    requested: Option<&'a str>,
    home_person_id: Option<&'a str>,
) -> Result<&'a str, AppError> {
    if let Some(req_id) = requested.map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(p) = store
            .persons
            .iter()
            .find(|p| p.id == req_id || p.gid.eq_ignore_ascii_case(req_id))
        {
            return Ok(&p.id);
        }
        return Err(AppError::PersonNotFound);
    }

    if let Some(hid) = home_person_id.map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(p) = store.persons.iter().find(|p| p.id == hid) {
            return Ok(&p.id);
        }
    }

    store
        .persons
        .first()
        .map(|p| p.id.as_str())
        .ok_or(AppError::PersonNotFound)
}

/// Calculates layout preview information including sheet counts and grid dimensions.
pub fn calculate_preview_info(
    store: &GenealogyPrivacyStore,
    req: &PdfExportRequest,
    instance_tree_title: &str,
    home_person_id: Option<&str>,
) -> Result<PdfExportPreviewInfo, AppError> {
    let focal_id = resolve_focal_id(store, req.focal_person_id.as_deref(), home_person_id)?;

    let layout = compute_tree_layout(
        store,
        focal_id,
        req.view_mode(),
        req.generations(),
        req.redact_living(),
        req.show_photos(),
        req.title.as_deref(),
        req.subtitle.as_deref(),
        req.include_partner(),
        req.include_siblings(),
        req.collateral(),
        instance_tree_title,
        None,
    )
    .ok_or(AppError::PersonNotFound)?;

    let is_single_poster = req.format().trim().eq_ignore_ascii_case("poster")
        || req.format().trim().eq_ignore_ascii_case("single");

    let tiling = plan_tiling(
        layout.width_mm,
        layout.height_mm,
        req.target_sheet_size(),
        req.overlap_mm(),
        is_single_poster,
        req.include_assembly_guide(),
        req.show_crop_marks(),
    );

    let summary_text = if is_single_poster {
        format!(
            "1 Poster Sheet ({} × {} mm) — {} Paper",
            tiling.target_sheet.width_mm as u32,
            tiling.target_sheet.height_mm as u32,
            tiling.target_sheet.name
        )
    } else {
        format!(
            "{} Sheets ({}×{} Grid) of {} Paper ({} × {} mm canvas)",
            tiling.total_sheets,
            tiling.grid_cols,
            tiling.grid_rows,
            tiling.target_sheet.name,
            layout.width_mm as u32,
            layout.height_mm as u32
        )
    };

    let preview_cards: Vec<PdfPreviewCardDto> = layout
        .cards
        .iter()
        .map(|c| PdfPreviewCardDto {
            id: c.id.clone(),
            gid: c.gid.clone(),
            full_name: c.full_name.clone(),
            lifespan: c.lifespan.clone(),
            is_living: c.is_living,
            is_focal: c.is_focal,
            sex: c.sex.clone(),
            x: c.x,
            y: c.y,
            width: c.width,
            height: c.height,
            generation: c.generation,
            has_avatar: c.avatar_bytes.is_some(),
        })
        .collect();

    let preview_connectors: Vec<PdfPreviewConnectorDto> = layout
        .connectors
        .iter()
        .map(|c| PdfPreviewConnectorDto {
            points: c.points.iter().map(|(x, y)| [*x, *y]).collect(),
            is_dashed: c.is_dashed,
            color: c.color.clone(),
        })
        .collect();

    let poster_layout = if is_single_poster {
        let sheet_w = tiling.target_sheet.width_mm;
        let sheet_h = tiling.target_sheet.height_mm;
        let frame_m = 10.0;
        let avail_w = sheet_w - 2.0 * frame_m - 10.0;
        let avail_h = sheet_h - 2.0 * frame_m - 10.0;
        let scale_x = avail_w / layout.width_mm;
        let scale_y = avail_h / layout.height_mm;
        let scale = scale_x.min(scale_y).min(1.2);
        let offset_x = frame_m + 5.0 + (avail_w - layout.width_mm * scale) / 2.0;
        let offset_y = frame_m + 5.0 + (avail_h - layout.height_mm * scale) / 2.0;

        Some(PdfPosterLayoutDto {
            sheet_width_mm: sheet_w,
            sheet_height_mm: sheet_h,
            scale,
            offset_x,
            offset_y,
            frame_margin: frame_m,
        })
    } else {
        None
    };

    let tiles: Vec<PdfTileSheetDto> = tiling
        .tiles
        .iter()
        .map(|t| PdfTileSheetDto {
            sheet_index: t.sheet_index,
            row: t.row,
            col: t.col,
            canvas_window_x: t.canvas_window_x,
            canvas_window_y: t.canvas_window_y,
            window_width: t.window_width,
            window_height: t.window_height,
            has_cut_right: t.overlap_right,
            has_cut_bottom: t.overlap_bottom,
            label: format!(
                "Sheet {} of {} [Row {}, Col {}]",
                t.sheet_index,
                tiling.total_sheets,
                t.row + 1,
                t.col + 1
            ),
        })
        .collect();

    Ok(PdfExportPreviewInfo {
        total_sheets: tiling.total_sheets,
        grid_rows: tiling.grid_rows,
        grid_cols: tiling.grid_cols,
        canvas_width_mm: layout.width_mm,
        canvas_height_mm: layout.height_mm,
        sheet_width_mm: tiling.target_sheet.width_mm,
        sheet_height_mm: tiling.target_sheet.height_mm,
        overlap_mm: tiling.overlap_mm,
        format: req.format().to_string(),
        target_sheet_size: tiling.target_sheet.name.to_string(),
        total_persons: layout.total_persons,
        summary_text,
        title: layout.title,
        subtitle: layout.subtitle,
        cards: preview_cards,
        connectors: preview_connectors,
        poster_layout,
        tiles,
    })
}

/// Generates the printable vector PDF bytes.
pub fn generate_printable_pdf(
    store: &GenealogyPrivacyStore,
    req: &PdfExportRequest,
    instance_tree_title: &str,
    home_person_id: Option<&str>,
    media_storage_path: Option<&str>,
) -> Result<Vec<u8>, AppError> {
    let focal_id = resolve_focal_id(store, req.focal_person_id.as_deref(), home_person_id)?;

    let layout = compute_tree_layout(
        store,
        focal_id,
        req.view_mode(),
        req.generations(),
        req.redact_living(),
        req.show_photos(),
        req.title.as_deref(),
        req.subtitle.as_deref(),
        req.include_partner(),
        req.include_siblings(),
        req.collateral(),
        instance_tree_title,
        media_storage_path,
    )
    .ok_or(AppError::PersonNotFound)?;

    let is_single_poster = req.format().trim().eq_ignore_ascii_case("poster")
        || req.format().trim().eq_ignore_ascii_case("single");

    let tiling = plan_tiling(
        layout.width_mm,
        layout.height_mm,
        req.target_sheet_size(),
        req.overlap_mm(),
        is_single_poster,
        req.include_assembly_guide(),
        req.show_crop_marks(),
    );

    let theme = PdfTheme::from_name(req.theme());

    render_pdf(&layout, &tiling, &theme)
}
