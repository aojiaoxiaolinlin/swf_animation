use std::{cell::RefCell, sync::Arc};

use swf::{CharacterId, Rectangle, Shape, Twips};

use crate::tag_utils::SwfMovie;

use super::{DisplayObject, DisplayObjectBase, TDisplayObject};

#[derive(Clone)]
pub struct Graphic {
    pub id: CharacterId,
    pub shape: Shape,
    pub bounds: Rectangle<Twips>,
    base: Arc<RefCell<DisplayObjectBase>>,
    swf_movie: Arc<SwfMovie>,
}

impl Graphic {
    pub fn from_swf_tag(shape: Shape, swf_movie: Arc<SwfMovie>) -> Self {
        Self {
            id: shape.id,
            bounds: shape.shape_bounds.clone(),
            shape,
            base: Default::default(),
            swf_movie,
        }
    }
}

impl From<Graphic> for Arc<RefCell<DisplayObject>> {
    fn from(value: Graphic) -> Self {
        Arc::new(RefCell::new(DisplayObject::Graphic(Arc::new(
            RefCell::new(value),
        ))))
    }
}

impl TDisplayObject for Graphic {
    fn base(&self) -> Arc<RefCell<DisplayObjectBase>> {
        self.base.clone()
    }

    fn movie(&self) -> Arc<SwfMovie> {
        self.swf_movie.clone()
    }

    fn character_id(&self) -> CharacterId {
        self.id
    }

    fn as_graphic(&self) -> Option<Graphic> {
        Some(self.clone())
    }
}
