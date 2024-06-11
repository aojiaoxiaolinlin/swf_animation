use std::{
    collections::VecDeque,
    fs::read,
    ops::DerefMut,
    path::Path,
    time::{Duration, Instant},
};

use ruffle_render::{backend::RenderBackend, quality::StageQuality};
use swf::Color;

use crate::{
    context::RenderContext,
    display_object::{movie_clip::MovieClip, TDisplayObject},
    library::MovieLibrary,
    stage::StageScaleMode,
};

pub struct Player {
    player_version: u8,
    root_movie_clip: MovieClip,
    movie_library: MovieLibrary,
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

    pub fn render(&mut self) {
        let mut cache_draws = Vec::new();
        let mut render_context = RenderContext {
            renderer: self.renderer.deref_mut(),
            commands: Default::default(),
            cache_draws: &mut cache_draws,
            transform_stack: &mut Default::default(),
            is_offscreen: false,
            use_bitmap_cache: false,
            library: &mut self.movie_library,
        };
        self.root_movie_clip.render(&mut render_context);

        let commands = render_context.commands;

        self.renderer
            .submit_frame(Color::WHITE, commands, cache_draws)
    }
}

pub struct PlayerBuilder {
    swf_resource: String,

    renderer: Option<Box<dyn RenderBackend>>,

    auto_play: bool,
    // align: StageAlign,
    forced_align: bool,
    scale_mode: StageScaleMode,
    forced_scale_mode: bool,
    allow_full_screen: bool,
    full_screen: bool,
    // letterbox: Letterbox,
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
            swf_resource: String::new(),
            renderer: None,
            auto_play: true,
            forced_align: false,
            scale_mode: StageScaleMode::default(),
            forced_scale_mode: false,
            allow_full_screen: true,
            full_screen: false,
            max_execution_duration: Duration::from_secs(15),
            viewport_width: 550,
            viewport_height: 400,
            viewport_scale_factor: 1.0,
            player_version: None,
            quality: StageQuality::High,
            frame_rate: None,
        }
    }
    #[inline]
    pub fn with_movie(mut self, swf_resource: String) -> Self {
        self.swf_resource = swf_resource;
        self
    }
    #[inline]
    pub fn with_renderer(mut self, renderer: Box<dyn RenderBackend>) -> Self {
        self.renderer = Some(renderer);
        self
    }
    #[inline]
    pub fn with_auto_play(mut self, auto_play: bool) -> Self {
        self.auto_play = auto_play;
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
    pub fn with_scale_mode(mut self, scale_mode: StageScaleMode) -> Self {
        self.scale_mode = scale_mode;
        self
    }
    #[inline]
    pub fn with_quality(mut self, quality: StageQuality) -> Self {
        self.quality = quality;
        self
    }
    #[inline]
    pub fn with_frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = Some(frame_rate);
        self
    }
    pub fn build(self) -> Player {
        let frame_rate = self.frame_rate.unwrap_or(24.0);
        let renderer = self.renderer.unwrap();
        let start_time = Instant::now();
        let recent_run_frame_timings = VecDeque::with_capacity(10);
        let time_til_next_timer = None;
        let current_frame = None;
        let frame_accumulator = 0.0;
        let instance_counter = 0;
        let time_offset = 0;
        let needs_render = false;
        let is_playing = self.auto_play;
        let (root_movie_clip, movie_library) = load_swf_resource(&self.swf_resource);
        Player {
            player_version: self.player_version.unwrap_or(0),
            root_movie_clip,
            movie_library,
            is_playing,
            needs_render,
            renderer,
            frame_rate,
            forced_frame_rate: false,
            frame_accumulator,
            recent_run_frame_timings,
            time_offset,
            instance_counter,
            time_til_next_timer,
            start_time,
            max_execution_duration: self.max_execution_duration,
            current_frame,
        }
    }
}

fn load_swf_resource<P: AsRef<Path>>(path: P) -> (MovieClip, MovieLibrary) {
    let data = read(path).unwrap();
    let swf_buf = swf::decompress_swf(&data[..]).unwrap();
    let parse_swf = swf::parse_swf(&swf_buf).unwrap();
    let mut root_movie_clip = MovieClip::new(parse_swf.header);
    let mut movie_library = MovieLibrary::new();

    root_movie_clip.set_name(Some("root".to_string()));
    root_movie_clip.load_swf(parse_swf.tags, &mut movie_library);

    return (root_movie_clip, movie_library);
}
