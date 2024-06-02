use swf::{CharacterId, Rectangle, Shape, Twips};

use crate::display_object::{DisplayObjectBase, TDisplayObject};

#[derive(Clone)]
pub struct Graphic {
    id: CharacterId,
    shape: Shape,
    bounds: Rectangle<Twips>,
    base: DisplayObjectBase,
}

impl Graphic {
    pub fn from_swf_tag(shape: Shape) -> Self {
        Self {
            id: shape.id,
            bounds: shape.shape_bounds.clone(),
            shape,
            base: DisplayObjectBase::default(),
        }
    }
}

impl TDisplayObject for Graphic{
    fn base_mut(&mut self) -> &mut DisplayObjectBase {
        &mut self.base
    }
}