use std::sync::Arc;

use ruffle_render::backend::ShapeHandle;
use swf::{CharacterId, Rectangle, Shape, Twips};

use crate::tag_utils::SwfMovie;

#[derive(Debug)]
pub struct Graphic {
    static_data: GraphicData,
}
impl Graphic {
    pub fn from_swf_tag(swf_shape: Shape) -> Self {
        let static_data = GraphicData {
            id: swf_shape.id,
            render_handle: None,
            bounds: swf_shape.shape_bounds.clone(),
            shape: swf_shape,
        };
        Graphic { static_data }
    }
}
#[derive(Debug)]
struct GraphicData {
    id: CharacterId,
    shape: Shape,
    render_handle: Option<ShapeHandle>,
    bounds: Rectangle<Twips>,
}
