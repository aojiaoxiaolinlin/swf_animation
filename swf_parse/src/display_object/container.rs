use swf::Depth;

use crate::{character::Character, context::UpdateContext};

use super::{DisplayObject, TDisplayObject};

pub trait TDisplayObjectContainer {
    fn replace_at_depth(
        &self,
        update_context: &mut UpdateContext<'_>,
        child: &mut DisplayObject,
        depth: Depth,
    ) {
        child.set_place_frame(0);
    }
}