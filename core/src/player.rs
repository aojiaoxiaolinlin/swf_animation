use std::{
    collections::VecDeque,
    ops::DerefMut,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use ruffle_render::{
    backend::{null::NullRenderer, RenderBackend, ViewportDimensions},
    quality::StageQuality,
};

use crate::{
    config::Letterbox,
    context::UpdateContext,
    display_object::{
        movie_clip::{self, MovieClip},
        stage::{self, Stage, StageAlign, StageScaleMode},
    },
    library::{self, Library},
    tag_utils::SwfMovie,
};
pub const NEWEST_PLAYER_VERSION: u8 = 32;

pub struct PlayerData {
    library: Library,
    stage: Stage,
    // action_queue: ActionQueue
    // timers: Timers,
}
impl PlayerData {
    fn update_context(&mut self) -> (&mut Stage, &mut Library) {
        (&mut self.stage, &mut self.library)
    }
}
pub struct Player {
    player_version: u8,
    swf: Arc<SwfMovie>,
    player_data: PlayerData,
    is_playing: bool,

    needs_render: bool,

    renderer: Box<dyn RenderBackend>,

    frame_rate: f64,

    forced_frame_rate: bool,

    // frame_phase:FramePhase,
    frame_accumulator: f64,
    recent_run_frame_timings: VecDeque<f64>,

    time_offset: u32,

    instance_counter: i32,

    time_til_next_timer: Option<f64>,

    start_time: Instant,

    max_execution_duration: Duration,

    current_frame: Option<u16>,
}
impl Player {
    fn max_frames_per_tick(&self) -> u32 {
        const MAX_FRAMES_PER_TICK: u32 = 5;

        if self.recent_run_frame_timings.is_empty() {
            5
        } else {
            let frame_time = 1000.0 / self.frame_rate;
            let average_run_frame_time = self.recent_run_frame_timings.iter().sum::<f64>()
                / self.recent_run_frame_timings.len() as f64;
            ((frame_time / average_run_frame_time) as u32).clamp(1, MAX_FRAMES_PER_TICK)
        }
    }
    fn add_frame_timing(&mut self, elapsed: f64) {
        self.recent_run_frame_timings.push_back(elapsed);
        if self.recent_run_frame_timings.len() >= 10 {
            self.recent_run_frame_timings.pop_front();
        }
    }
    pub fn time_til_next_frame(&self) -> std::time::Duration {
        let frame_time = 1000.0 / self.frame_rate;
        let mut dt = if self.frame_accumulator <= 0.0 {
            frame_time
        } else if self.frame_accumulator >= frame_time {
            0.0
        } else {
            frame_time - self.frame_accumulator
        };

        if let Some(time_til_next_timer) = self.time_til_next_timer {
            dt = dt.min(time_til_next_timer)
        }

        dt = dt.max(0.0);

        std::time::Duration::from_micros(dt as u64 * 1000)
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
    pub fn set_is_playing(&mut self, v: bool) {
        self.is_playing = v;
    }
    pub fn needs_render(&self) -> bool {
        self.needs_render
    }

    pub fn load_movie(&mut self, movie_path: &PathBuf) {
        let movie = SwfMovie::from_path(movie_path).unwrap();
        let movie = Arc::new(movie);
        let mut movie_clip = MovieClip::new(movie.clone());
        self.update_context(|update_context| {
            movie_clip.parse(update_context);
            update_context.library.length(&movie.clone());
            update_context.set_root_movie(movie.clone())
        });
        self.swf = movie.clone();
    }

    pub fn tick(&mut self) {
        self.run_frame()
    }
    pub fn run_frame(&mut self) {
        dbg!("run_frame");
    }

    pub fn render(&mut self) {}

    pub fn update_context<F, R>(&mut self, f: F) -> R
    where
        F: for<'a> FnOnce(&mut UpdateContext<'a>) -> R,
    {
        let (stage, library) = self.player_data.update_context();

        let mut update_context = UpdateContext {
            library,
            stage,
            player_version: self.player_version,
            renderer: self.renderer.deref_mut(),
            forced_frame_rate: self.forced_frame_rate,
            frame_rate: &mut self.frame_rate,
        };

        let ret = f(&mut update_context);
        ret
    }
}
pub struct PlayerBuilder {
    movie: Option<SwfMovie>,

    renderer: Option<Box<dyn RenderBackend>>,

    auto_play: bool,
    align: StageAlign,
    forced_align: bool,
    scale_mode: StageScaleMode,
    forced_scale_mode: bool,
    allow_full_screen: bool,
    full_screen: bool,
    letterbox: Letterbox,
    max_execution_duration: Duration,
    viewport_width: u32,
    viewport_height: u32,
    viewport_scale_factor: f64,
    player_version: Option<u8>,
    quality: StageQuality,
    frame_rate: Option<f64>,
}

impl PlayerBuilder {
    #[inline]
    pub fn new() -> Self {
        Self {
            movie: None,
            renderer: None,
            auto_play: true,
            align: StageAlign::default(),
            forced_align: false,
            scale_mode: StageScaleMode::default(),
            forced_scale_mode: false,
            allow_full_screen: false,
            full_screen: false,
            letterbox: Letterbox::Fullscreen,
            max_execution_duration: Duration::from_secs(if cfg!(debug_assertions) {
                u64::MAX
            } else {
                15
            }),
            viewport_width: 550,
            viewport_height: 400,
            viewport_scale_factor: 1.0,
            player_version: None,
            quality: StageQuality::High,
            frame_rate: None,
        }
    }
    #[inline]
    pub fn with_movie(mut self, movie: SwfMovie) -> Self {
        self.movie = Some(movie);
        self
    }
    #[inline]
    pub fn with_renderer(mut self, renderer: impl 'static + RenderBackend) -> Self {
        self.renderer = Some(Box::new(renderer));
        self
    }
    #[inline]
    pub fn with_boxed_renderer(mut self, renderer: Box<dyn RenderBackend>) -> Self {
        self.renderer = Some(renderer);
        self
    }
    #[inline]
    pub fn with_align(mut self, align: StageAlign, force: bool) -> Self {
        self.align = align;
        self.forced_align = force;
        self
    }
    #[inline]
    pub fn with_auto_play(mut self, auto_play: bool) -> Self {
        self.auto_play = auto_play;
        self
    }
    #[inline]
    pub fn with_letterbox(mut self, letterbox: Letterbox) -> Self {
        self.letterbox = letterbox;
        self
    }
    #[inline]
    pub fn with_max_execution_duration(mut self, duration: Duration) -> Self {
        self.max_execution_duration = duration;
        self
    }
    #[inline]
    pub fn with_viewport_dimensions(
        mut self,
        width: u32,
        height: u32,
        dpi_scale_factor: f64,
    ) -> Self {
        self.viewport_width = width;
        self.viewport_height = height;
        self.viewport_scale_factor = dpi_scale_factor;
        self
    }
    #[inline]
    pub fn with_scale_mode(mut self, scale: StageScaleMode, force: bool) -> Self {
        self.scale_mode = scale;
        self.forced_scale_mode = force;
        self
    }

    /// Sets whether the stage is fullscreen.
    pub fn with_full_screen(mut self, fullscreen: bool) -> Self {
        self.full_screen = fullscreen;
        self
    }
    pub fn with_quality(mut self, quality: StageQuality) -> Self {
        self.quality = quality;
        self
    }
    pub fn with_player_version(mut self, version: Option<u8>) -> Self {
        self.player_version = version;
        self
    }
    pub fn with_frame_rate(mut self, frame_rate: Option<f64>) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    pub fn build(self) -> Arc<Mutex<Player>> {
        let renderer = self.renderer.unwrap_or_else(|| {
            Box::new(NullRenderer::new(ViewportDimensions {
                width: self.viewport_width,
                height: self.viewport_height,
                scale_factor: self.viewport_scale_factor,
            }))
        });
        let player_version = self.player_version.unwrap_or(NEWEST_PLAYER_VERSION);
        let fake_movie = Arc::new(SwfMovie::empty(player_version));
        let frame_rate = self.frame_rate.unwrap_or(24.0);
        let forced_frame_rate = self.frame_rate.is_some();
        let player = Arc::new(Mutex::new(Player {
            player_data: PlayerData {
                stage: Stage::empty(self.full_screen, fake_movie.clone()),
                library: Library::empty(),
            },
            renderer,
            player_version,
            swf: fake_movie.clone(),
            current_frame: None,
            frame_rate,
            forced_frame_rate,
            frame_accumulator: 0.0,
            recent_run_frame_timings: VecDeque::with_capacity(10),
            time_offset: 0,
            start_time: Instant::now(),
            time_til_next_timer: None,
            is_playing: self.auto_play,
            needs_render: true,
            instance_counter: 0,
            max_execution_duration: self.max_execution_duration,
        }));
        let mut player_lock = player.lock().unwrap();
        drop(player_lock);
        player
    }
}
