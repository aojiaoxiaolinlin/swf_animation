use std::{
    path::PathBuf, rc::Rc, sync::{Arc, Mutex, MutexGuard}
};

use crate::movie::MovieView;
use anyhow::anyhow;
use player_core::{Player, PlayerBuilder};
use ruffle_render_wgpu::{backend::WgpuRenderBackend, descriptors::Descriptors};
use winit::window::Window;

pub struct PlayerController {
    active_player: Option<ActivePlayer>,
    window: Rc<Window>,
    descriptors: Arc<Descriptors>,
}

impl PlayerController {
    pub fn new(window: Rc<Window>, descriptors: Arc<Descriptors>) -> Self {
        PlayerController {
            window,
            descriptors,
            active_player: None,
        }
    }

    pub fn get(&self) -> Option<MutexGuard<Player>> {
        match &self.active_player {
            Some(active_player) => Some(active_player.player.try_lock().expect("获取 player 失败")),
            None => None,
        }
    }
    pub fn create(&mut self, movie_url: &PathBuf, movie_view: MovieView) {
        self.active_player = Some(ActivePlayer::new(
            movie_url.clone(),
            self.descriptors.clone(),
            movie_view,
        ))
    }
    pub fn destroy(&mut self) {
        self.active_player = None;
    }
}

struct ActivePlayer {
    movie_url: PathBuf,
    player: Arc<Mutex<Player>>,
}

impl ActivePlayer {
    pub fn new(
        movie_url: PathBuf,
        descriptors: Arc<Descriptors>,
        movie_view: MovieView,
    ) -> Self {
        let builder = PlayerBuilder::new();
        let renderer = WgpuRenderBackend::new(descriptors, movie_view)
            .map_err(|e| anyhow!(e.to_string()))
            .expect("创建WebGPU渲染器失败");
        let player = builder.with_renderer(renderer)
            .with_auto_play(true)
            .with_max_execution_duration(std::time::Duration::from_secs(15)).build();

        // let on_metadata = move | swf_header:&ruffle_core::swf::HeaderExt |{
            
        // };
        let mut player_lock = player.lock().unwrap();
        player_lock.load_movie(&movie_url);
        drop(player_lock);
        Self {
            movie_url,
            player,
        }
    }
}
