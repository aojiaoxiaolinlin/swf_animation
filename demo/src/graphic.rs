use std::error;

use swf::{CharacterId, Rectangle, Shape, Twips};

use crate::{display_object::{DisplayObject, DisplayObjectBase, TDisplayObject}, library::MovieLibrary};

#[derive(Clone)]
pub struct Graphic {
    pub id: CharacterId,
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
    
    fn base(&self) -> &DisplayObjectBase {
        &self.base
    }
    
    fn character_id(&self) -> CharacterId {
        self.id
    }
    fn replace_with(&mut self, id: CharacterId, library: &mut MovieLibrary) {
        if let Some(new_graphic) = library.get_graphic(id) {
            self.id = new_graphic.id;
            self.shape = new_graphic.shape;
            self.bounds = new_graphic.bounds;
            self.base = new_graphic.base;
        }else {
            dbg!("PlaceObject: expected Graphic at character ID {}", id);
        }
    }
}
impl From<Graphic> for DisplayObject {
    fn from(graphic: Graphic) -> Self {
        DisplayObject::Graphic(graphic)
    }
}