use std::{cell::RefCell, collections::BTreeMap, ops::Bound, rc::Rc};

use swf::Depth;

use crate::context::UpdateContext;

use super::{DisplayObject, TDisplayObject};

#[derive(Clone)]
pub struct ChildContainer {
    render_list: Rc<Vec<Rc<RefCell<DisplayObject>>>>,
    depth_list: BTreeMap<Depth, Rc<RefCell<DisplayObject>>>,
}
impl ChildContainer {
    pub fn new() -> Self {
        Self {
            render_list: Rc::new(Vec::new()),
            depth_list: BTreeMap::new(),
        }
    }
    pub fn insert_child_into_depth_list(
        &mut self,
        depth: Depth,
        child: Rc<RefCell<DisplayObject>>,
    ) -> Option<Rc<RefCell<DisplayObject>>> {
        self.depth_list.insert(depth, child)
    }
    fn render_list_mut(&mut self) -> &mut Vec<Rc<RefCell<DisplayObject>>> {
        Rc::make_mut(&mut self.render_list)
    }
    fn insert_id(&mut self, id: usize, child: Rc<RefCell<DisplayObject>>) {
        self.render_list_mut().insert(id, child);
    }
    fn push_id(&mut self, child: Rc<RefCell<DisplayObject>>) {
        self.render_list_mut().push(child);
    }
    pub fn replace_at_depth(&mut self, depth: Depth, child: Rc<RefCell<DisplayObject>>) {
        let prev_child = self.insert_child_into_depth_list(depth, child.clone());
        if let Some(prev_child) = prev_child {
            if let Some(position) = self
                .render_list
                .iter()
                .position(|x| x.as_ptr() == prev_child.as_ptr())
            {
                self.insert_id(position + 1, child);
            } else {
                dbg!("Child not found in render list");
                self.push_id(child);
            }
        } else {
            let above = self
                .depth_list
                .range((Bound::Excluded(depth), Bound::Unbounded))
                .map(|(_, v)| v.clone())
                .next();

            if let Some(above_child) = above {
                if let Some(position) = self
                    .render_list
                    .iter()
                    .position(|x| x.as_ptr() == above_child.as_ptr())
                {
                    self.insert_id(position, child)
                } else {
                    self.push_id(child);
                }
            } else {
                self.push_id(child);
            }
        }
    }
}
pub trait TDisplayObjectContainer {
    fn raw_container_mut(&mut self) -> &mut ChildContainer;

    fn replace_at_depth(
        &mut self,
        update_context: &mut UpdateContext<'_>,
        child: Rc<RefCell<DisplayObject>>,
        depth: Depth,
    ) {
        self.raw_container_mut()
            .replace_at_depth(depth, child.clone());
        let mut child = child.borrow_mut();
        child.set_place_frame(0);
        child.set_depth(depth);
    }
}
