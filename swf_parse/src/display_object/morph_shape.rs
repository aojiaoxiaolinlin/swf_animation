use std::sync::Arc;

use swf::DefineMorphShape;

use crate::tag_utils::SwfMovie;
#[derive(Debug)]
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
}
#[derive(Debug)]
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
