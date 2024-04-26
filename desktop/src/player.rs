use std::{rc::Rc, sync::Arc};

use ruffle_render_wgpu::descriptors::Descriptors;
use winit::window::Window;

pub struct PlayerController {
    // player: Option<ActivePlayer>,
    window: Rc<Window>,
    descriptors: Arc<Descriptors>,
}

impl PlayerController {
    pub fn new(window: Rc<Window>, descriptors: Arc<Descriptors>) -> Self {
        PlayerController {
            window,
            descriptors,
        }
    }
}