use std::collections::HashMap;

use swf::{CharacterId, Shape};
use tessellator::{Mesh, ShapeTessellator};

use super::bitmap::CompressedBitmap;

mod matrix;
mod shape_utils;
pub mod tessellator;

pub fn parse_shape_and_bitmap(
    shapes: HashMap<CharacterId, Shape>,
    bitmaps: HashMap<CharacterId, CompressedBitmap>,
) -> HashMap<CharacterId, Mesh> {
    let mut shape_library = HashMap::new();
    let mut tessellator = ShapeTessellator::new();
    for (id, shape) in shapes.iter() {
        let lyon_mesh = tessellator.tessellate_shape(shape.into(), &bitmaps);
        shape_library.insert(*id, lyon_mesh);
    }
    shape_library
}
