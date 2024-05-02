use crate::display_object::DisplayObjectBase;
use crate::drawing::Drawing;
use crate::tag_utils::SwfMovie;
use swf::CharacterId;
use ruffle_render::backend::ShapeHandle;
use swf::Twips;
use swf::Rectangle;
use std::sync::Arc;

pub struct Graphic {
    base: DisplayObjectBase,
    static_data: GraphicStatic,
    drawing: Option<Drawing>,
}


struct GraphicStatic {
    id: CharacterId,
    shape: swf::Shape,
    render_handle: Option<ShapeHandle>,
    bounds: Rectangle<Twips>,
    movie: Arc<SwfMovie>,
}