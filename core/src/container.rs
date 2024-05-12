use std::{collections::BTreeMap, rc::Rc, sync::Arc};

use swf::Depth;

use crate::{display_object::DisplayObject, tag_utils::SwfMovie};

pub struct ChildrenContainer {
    render_list: Rc<Vec<DisplayObject>>,
    depth_list: BTreeMap<Depth, DisplayObject>,
    movie: Arc<SwfMovie>,
}

impl ChildrenContainer {
    pub fn new(movie: Arc<SwfMovie>) -> Self {
        Self {
            render_list: Rc::new(Vec::new()),
            depth_list: BTreeMap::new(),
            movie,
        }
    }
    fn insert_child_into_depth_list(&mut self,depth:Depth,child:DisplayObject)->Option<DisplayObject> {
        self.depth_list.insert(depth,child)
    }
}
