use std::{cell::RefCell, collections::BTreeMap, ops::Bound, sync::Arc};

use swf::Depth;

use super::display_object::DisplayObject;

#[derive(Clone)]
pub struct ChildContainer {
    render_list: Arc<Vec<Arc<RefCell<DisplayObject>>>>,
    depth_list: BTreeMap<Depth, Arc<RefCell<DisplayObject>>>,
}

impl Default for ChildContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl ChildContainer {
    pub fn new() -> Self {
        Self {
            render_list: Arc::new(Vec::new()),
            depth_list: BTreeMap::new(),
        }
    }
    pub fn render_list_len(&self) -> usize {
        self.render_list.len()
    }

    pub fn render_list(&self) -> Arc<Vec<Arc<RefCell<DisplayObject>>>> {
        self.render_list.clone()
    }

    pub fn display_objects_mut(&mut self) -> &mut BTreeMap<Depth, Arc<RefCell<DisplayObject>>> {
        &mut self.depth_list
    }

    pub fn child_by_depth(&mut self, depth: Depth) -> Option<Arc<RefCell<DisplayObject>>> {
        self.depth_list.get(&depth).cloned()
    }

    pub fn render_list_mut(&mut self) -> &mut Vec<Arc<RefCell<DisplayObject>>> {
        Arc::make_mut(&mut self.render_list)
    }

    fn push_id(&mut self, child: Arc<RefCell<DisplayObject>>) {
        self.render_list_mut().push(child);
    }

    fn insert_id(&mut self, id: usize, child: Arc<RefCell<DisplayObject>>) {
        self.render_list_mut().insert(id, child);
    }

    fn insert_child_into_depth_list(
        &mut self,
        depth: Depth,
        child: Arc<RefCell<DisplayObject>>,
    ) -> Option<Arc<RefCell<DisplayObject>>> {
        self.depth_list.insert(depth, child)
    }

    pub fn replace_at_depth(&mut self, depth: Depth, child: Arc<RefCell<DisplayObject>>) {
        if let Some(prev_child) = self.insert_child_into_depth_list(depth, child.clone()) {
            if let Some(position) = self
                .render_list
                .iter()
                .position(|x| x.as_ptr() == prev_child.as_ptr())
            {
                self.insert_id(position + 1, child);
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
                    self.insert_id(position, child);
                } else {
                    self.push_id(child)
                }
            } else {
                self.push_id(child)
            }
        }
    }

    pub fn remove_child_from_depth_list(
        &mut self,
        child: Depth,
    ) -> Option<Arc<RefCell<DisplayObject>>> {
        if let Some(remove_display_object) = self.depth_list.remove(&child) {
            Some(remove_display_object)
        } else {
            None
        }
    }
    pub fn remove_child(&mut self, child: Depth) {
        if let Some(remove_display_object) = self.remove_child_from_depth_list(child) {
            Self::remove_child_from_render_list(self, remove_display_object);
        }
    }
    fn remove_child_from_render_list(
        container: &mut ChildContainer,
        child: Arc<RefCell<DisplayObject>>,
    ) -> bool {
        let render_list_position: Option<usize> = container
            .render_list
            .iter()
            .position(|x| x.as_ptr() == child.as_ptr());
        if let Some(position) = render_list_position {
            container.render_list_mut().remove(position);
            true
        } else {
            false
        }
    }
}
