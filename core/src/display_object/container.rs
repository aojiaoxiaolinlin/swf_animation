use std::{collections::BTreeMap, fmt::Debug, sync::Arc};

use swf::Depth;

use crate::tag_utils::SwfMovie;

use super::DisplayObject;

#[derive(Clone)]
pub struct ChildContainer {
    render_list: Vec<DisplayObject>,

    depth_list: BTreeMap<Depth, DisplayObject>,

    /// 这个容器属于哪个影片剪辑。
    swf_movie: Arc<SwfMovie>,
}

impl Debug for ChildContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChildContainer")
            .field("render_list", &self.render_list)
            .field("depth_list", &self.depth_list)
            .finish()
    }
}

impl ChildContainer {
    pub fn new(swf_movie: Arc<SwfMovie>) -> Self {
        Self {
            render_list: Vec::new(),
            depth_list: BTreeMap::new(),
            swf_movie,
        }
    }
    pub fn swf_movie(&self) -> &Arc<SwfMovie> {
        &self.swf_movie
    }
    pub fn set_swf_movie(&mut self, swf_movie: Arc<SwfMovie>) {
        self.swf_movie = swf_movie;
    }
}
