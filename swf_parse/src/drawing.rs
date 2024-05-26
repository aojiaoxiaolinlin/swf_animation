use std::cell::{Cell, RefCell};

use ruffle_render::{
    backend::ShapeHandle,
    bitmap::BitmapInfo,
    shape_utils::{DrawCommand, FillRule},
};
use swf::{FillStyle, LineStyle, Point, Rectangle, Twips};

pub struct Drawing {
    render_handler: RefCell<Option<ShapeHandle>>,
    shape_bounds: Rectangle<Twips>,
    edge_bounds: Rectangle<Twips>,
    dirty: Cell<bool>,
    paths: Vec<DrawingPath>,
    bitmaps: Vec<BitmapInfo>,
    current_fill: Option<DrawingFill>,
    current_line: Option<DrawingLine>,
    pending_lines: Vec<DrawingLine>,
    cursor: Point<Twips>,
    fill_start: Point<Twips>,
    winding_rule: FillRule,
}
#[derive(Debug, Clone)]
struct DrawingFill {
    style: FillStyle,
    commands: Vec<DrawCommand>,
}
#[derive(Debug, Clone)]
struct DrawingLine {
    style: LineStyle,
    commands: Vec<DrawCommand>,
    is_closed: bool,
}
#[derive(Debug, Clone)]
enum DrawingPath {
    Fill(DrawingFill),
    Line(DrawingLine),
}
