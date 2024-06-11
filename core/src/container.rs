use std::{collections::BTreeMap, ops::Bound, rc::Rc};

use ruffle_render::commands::CommandHandler;
use swf::Depth;

use crate::{context::RenderContext, display_object::{movie_clip::MovieClip, DisplayObject, TDisplayObject}};

#[derive(Clone)]
pub struct ChildContainer {
    render_list: Vec<DisplayObject>,
    depth_list: BTreeMap<Depth, DisplayObject>,
}

impl ChildContainer {
    pub fn new() -> Self {
        Self {
            render_list: Vec::new(),
            depth_list: BTreeMap::new(),
        }
    }
    fn insert_child_into_depth_list(
        &mut self,
        depth: Depth,
        child: DisplayObject,
    ) -> Option<DisplayObject> {
        self.depth_list.insert(depth, child)
    }
    fn insert_id(&mut self, id: usize, child: DisplayObject) {
        self.render_list.insert(id, child);
    }
    fn replace_at_depth(&mut self, depth: Depth, child: DisplayObject) {
        let prev_child = self.insert_child_into_depth_list(depth, child.clone());
        if let Some(prev_child) = prev_child {
            if let Some(position) = self
                .render_list
                .iter()
                .position(|x| x.character_id() == prev_child.character_id())
            {
                self.render_list[position] = child;
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
                    .position(|x| x.character_id() == above_child.character_id())
                {
                    self.insert_id(position, child);
                } else {
                    self.render_list.push(child)
                }
            } else {
                self.render_list.push(child)
            }
        }
    }
    fn highest_depth(&self) -> Depth {
        self.depth_list.keys().next_back().copied().unwrap_or(0)
    }
}

#[derive(Clone)]
pub enum DisplayObjectContainer {
    MovieClip(MovieClip),
}

pub trait TDisplayObjectContainer: Into<DisplayObjectContainer> + Clone {
    fn raw_container(&self) -> &ChildContainer;
    fn raw_container_mut(&mut self) -> &mut ChildContainer;
    fn replace_at_depth(&mut self, depth: Depth, child: DisplayObject) {
        self.raw_container_mut().replace_at_depth(depth, child);
    }
    fn iter_render_list(self) -> RenderIter {
        RenderIter::from_container(self.into())
    }
    fn num_children(&self) -> usize {
        self.raw_container().render_list.len()
    }
    fn child_by_depth(&self, depth: Depth) -> Option<DisplayObject> {
        self.raw_container().depth_list.get(&depth).cloned()
    }
    fn render_children(&self, render_context:&mut RenderContext) {
        let mut clip_depth = 0;
        let mut clip_depth_stack: Vec<(Depth, DisplayObject)> = vec![];
        for mut child in self.clone().iter_render_list() {
            let depth = child.depth();

            child.pre_render(render_context);
            if child.clip_depth() > 0 && child.allow_as_mask() {
                // Push and render the mask.
                clip_depth = child.clip_depth();
                child.render(render_context);
                clip_depth_stack.push((clip_depth, child));
                render_context.commands.push_mask();
                render_context.commands.activate_mask();
            } else if child.visible() || render_context.commands.drawing_mask() {
                // Either a normal visible child, or a descendant of a mask object
                // that we're drawing. The 'visible' flag is ignored for all descendants
                // of a mask.
                child.render(render_context);
            }
            // Check if we need to pop off a mask.
            // This must be a while loop because multiple masks can be popped
            // at the same depth.
            while clip_depth > 0 && depth > clip_depth {
                // Clear the mask stencil and pop the mask.
                let (prev_clip_depth, clip_child) = clip_depth_stack.pop().unwrap();
                clip_depth = prev_clip_depth;
                render_context.commands.deactivate_mask();
                clip_child.render(render_context);
                render_context.commands.pop_mask();
            }
            
        }

        // Pop any remaining masks.
        for (_, clip_child) in clip_depth_stack.into_iter().rev() {
            render_context.commands.deactivate_mask();
            clip_child.render(render_context);
            render_context.commands.pop_mask();
        }
    }
}

impl TDisplayObjectContainer for DisplayObjectContainer {
    fn raw_container(&self) -> &ChildContainer {
        match self {
            DisplayObjectContainer::MovieClip(movie_clip) => movie_clip.raw_container(),
        }
    }
    fn raw_container_mut(&mut self) -> &mut ChildContainer {
        match self {
            DisplayObjectContainer::MovieClip(movie_clip) => movie_clip.raw_container_mut(),
        }
    }
    fn iter_render_list(self) -> RenderIter {
        RenderIter::from_container(self.into())
    }
}

pub struct RenderIter {
    src: Rc<Vec<DisplayObject>>,
    i: usize,
    neg_i: usize,
}
impl RenderIter {
    fn from_container(container: DisplayObjectContainer) -> Self {
        Self {
            src: Rc::new(container.raw_container().render_list.clone()),
            i: 0,
            neg_i: container.num_children(),
        }
    }
}

impl Iterator for RenderIter {
    type Item = DisplayObject;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.neg_i {
            return None;
        }

        let this = self.src.get(self.i).cloned();
        self.i += 1;
        this
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.neg_i - self.i;
        (len, Some(len))
    }
}

impl DoubleEndedIterator for RenderIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.i == self.neg_i {
            return None;
        }

        self.neg_i -= 1;
        self.src.get(self.neg_i).cloned()
    }
}
