use std::{sync::{Mutex, Weak}, time::{Duration, Instant}};

use rand::rngs::SmallRng;
use ruffle_render::{backend::{BitmapCacheEntry, RenderBackend}, commands::CommandList, transform::TransformStack};
use ruffle_video::backend::VideoBackend;

use crate::{frame_lifecycle::FramePhase, library::Library, Player, Stage};

pub struct UpdateContext<'a> {
    // pub library: &'a mut Library,
    pub renderer: &'a mut dyn RenderBackend,
    pub video: &'a mut dyn VideoBackend,
    pub rng: &'a mut SmallRng,
    pub stage: Stage,
    pub player: Weak<Mutex<Player>>,
    pub instance_counter: &'a mut i32,
    /// The instant at which the SWF was launched.
    pub start_time: Instant,

    /// The instant at which the current update started.
    pub update_start: Instant,

    /// The maximum amount of time that can be called before a `Error::ExecutionTimeout`
    /// is raised. This defaults to 15 seconds but can be changed.
    pub max_execution_duration: Duration,

    /// The current stage frame rate.
    pub frame_rate: &'a mut f64,
    pub forced_frame_rate: bool,
    pub frame_phase: &'a mut FramePhase,
}

pub struct RenderContext<'a>{
    pub renderer: &'a mut dyn RenderBackend,
    pub commands: CommandList,
    pub cache_draws: &'a mut Vec<BitmapCacheEntry>,
    pub transform_stack: &'a mut TransformStack,
    pub is_offscreen: bool,
    pub stage: Stage,
}