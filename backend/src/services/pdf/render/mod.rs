#![allow(unused_imports)]
pub(crate) use printpdf::{
    Color, ColorBits, ColorSpace, Image, ImageTransform, ImageXObject, Line, LineCapStyle,
    LineDashPattern, LineJoinStyle, Mm, PdfDocument, Point, Polygon, Px, Rgb,
};

pub(crate) use super::layout::{CanvasLayout, RenderConnector, RenderPersonCard, BANNER_HEIGHT_MM};
pub(crate) use super::themes::{PdfColor, PdfTheme};
pub(crate) use super::tiling::{self, TilingPlan};
pub(crate) use crate::error::AppError;

pub mod posters;
pub use posters::*;
pub mod primitives;
pub use primitives::*;
