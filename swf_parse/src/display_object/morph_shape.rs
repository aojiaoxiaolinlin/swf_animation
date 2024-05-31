use std::sync::Arc;

use swf::DefineMorphShape;

use crate::tag_utils::SwfMovie;

use super::{DisplayObjectBase, TDisplayObject};
#[derive(Debug, Clone)]
pub struct MorphShape {
    static_data: MorphShapeData,
    ratio: u16,
}

impl MorphShape {
    pub fn from_swf_tag(swf_shape: DefineMorphShape, movie: Arc<SwfMovie>) -> Self {
        let static_data = MorphShapeData::from_swf_tag(swf_shape, movie);
        MorphShape {
            static_data,
            ratio: 0,
        }
    }
    pub fn set_ratio(&mut self, ratio: u16) {
        self.ratio = ratio;
    }
}
#[derive(Debug, Clone)]
pub struct MorphShapeData {
    id: u16,
    start: swf::MorphShape,
    end: swf::MorphShape,
    movie: Arc<SwfMovie>,
}
impl MorphShapeData {
    pub fn from_swf_tag(swf_shape: DefineMorphShape, movie: Arc<SwfMovie>) -> Self {
        Self {
            id: swf_shape.id,
            start: swf_shape.start.clone(),
            end: swf_shape.end.clone(),
            movie,
        }
    }
}

impl TDisplayObject for MorphShape {
    fn base_mut(&mut self) ->  &mut DisplayObjectBase {
        todo!()
    }
    fn as_morph_shape(&mut self) -> Option< &mut self::MorphShape> {
        Some(self)
    }
}
