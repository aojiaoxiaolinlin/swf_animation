use std::{collections::BTreeMap, rc::Rc};

use swf::Depth;

use crate::context::UpdateContext;

use super::{DisplayObject, TDisplayObject};

#[derive(Clone)]
pub struct ChildContainer {
    render_list: Rc<Vec<DisplayObject>>,
    depth_list: BTreeMap<Depth, DisplayObject>,
}
impl ChildContainer {
    pub fn new() -> Self {
        Self {
            render_list: Rc::new(Vec::new()),
            depth_list: BTreeMap::new(),
        }
    }
    pub fn insert_child_into_depth_list(&mut self, depth: Depth, child: DisplayObject)->Option<DisplayObject> {
        self.depth_list.insert(depth,child)
    }
    pub fn replace_at_depth(&mut self, depth: Depth, child: DisplayObject) {
        let prev_child = self.insert_child_into_depth_list(depth,child);
        if let Some(prev_child) = prev_child {
            // if let Some(position) = self.render_list
            // .iter().position(|x|x);
        }
    }
}
pub trait TDisplayObjectContainer {
    fn raw_container_mut(&mut self) -> &mut ChildContainer;

    fn replace_at_depth(
        &mut self,
        update_context: &mut UpdateContext<'_>,
        child: &mut DisplayObject,
        depth: Depth,
    ) {
        // self.raw_container_mut().replace_at_depth(depth,child);
        child.set_place_frame(0);
        child.set_depth(depth);
    }
}
