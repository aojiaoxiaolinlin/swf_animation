use std::{cell::{RefCell, RefMut}, collections::BTreeMap, ops::Bound, rc::Rc, sync::Arc};

use swf::Depth;

use crate::{
    display_object::{DisplayObject, TDisplayObject},
    tag_utils::SwfMovie,
};

pub struct ChildrenContainer {
    render_list: Rc<RefCell<Vec<DisplayObject>>>,
    depth_list: BTreeMap<Depth, DisplayObject>,
    movie: Arc<SwfMovie>,
}

impl ChildrenContainer {
    pub fn new(movie: Arc<SwfMovie>) -> Self {
        Self {
            render_list: Rc::new(RefCell::new(Vec::new())),
            depth_list: BTreeMap::new(),
            movie,
        }
    }
    fn insert_child_into_depth_list(
        &mut self,
        depth: Depth,
        child: DisplayObject,
    ) -> Option<DisplayObject> {
        self.depth_list.insert(depth, child)
    }
}
impl ChildrenContainer {
    fn render_list_mut(&mut self)->RefMut<'_, Vec<DisplayObject>, >{
        self.render_list.borrow_mut()
    }
    fn insert_id(&mut self, id: usize, child: DisplayObject) {
        self.render_list_mut().insert(id, child);
    }
    fn push_id(&mut self, child:DisplayObject){
        self.render_list_mut().push(child);
    }
    fn replace_at_depth(&mut self, child: DisplayObject, depth: Depth) -> Option<DisplayObject> {
        let prev_child = self.insert_child_into_depth_list(depth, child);
        if let Some(prev_child) = prev_child {
            if let Some(position) = self
                .render_list
                .borrow_mut()
                .iter()
                .position(|x| x.depth() == prev_child.depth())
            {
                // self.insert_id(position + 1, child);
                None
            } else {
                // self.push_id(child);
                None
            }
        } else {
            let above = self
                .depth_list
                .range((Bound::Excluded(depth), Bound::Unbounded))
                .map(|(_, o)| o)
                .next();
            if let Some(above_child) = above {
                if let Some(position) = self.render_list.borrow_mut().iter().position(|x|x.depth()==above_child.depth()){
                    // self.insert_id(position,child);
                    None
                }else {
                    // self.push_id(child);
                    None
                }
            }else {
                // self.push_id(child);
                None
            }
        }
    }
}

pub trait TDisplayObjectContainer:
Into<DisplayObject> {
    fn container(&mut self) -> ChildrenContainer;
    fn replace_at_depth(&mut self, mut child: DisplayObject, depth: Depth) -> Option<DisplayObject> {
        // child.set_parent(Some(Rc::new((*self).into())));
        // child.set_place_frame(0);
        // child.set_depth(depth);

        // let removed_child = self.container().replace_at_depth(child, depth);

        // if let Some(mut removed_child) = removed_child {
        //     removed_child.set_parent(None);
            
        // }
        // removed_child
        todo!()
    }
}
