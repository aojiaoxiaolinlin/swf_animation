use std::{
    cell::RefCell, collections::VecDeque, sync::{Arc, Mutex, Weak}, time::{Duration, Instant}
};

use crate::{
    config::Letterbox, context::{RenderContext, UpdateContext}, display_object::{movie_clip::MovieClip, stage::WindowMode}, frame_lifecycle::FramePhase, loader::Loader, locale::get_current_date_time, tag_utils::SwfMovie, Stage
};
use rand::{rngs::SmallRng, SeedableRng};
use ruffle_render::{
    backend::{null::NullRenderer, RenderBackend, ViewportDimensions}, commands::CommandList, quality::{self, StageQuality}, transform::TransformStack
};
use ruffle_video::backend::VideoBackend;
use ruffle_video::null as videoNull;
use swf::Color;
use url::Url;

type Renderer = Box<dyn RenderBackend>;
type Video = Box<dyn VideoBackend>;

// struct RootData<'a>{
//     stage: &'a Stage,
//     stream_manager: StreamManager<'a>,
//     dynamic_root: DynamicRootSet<'a>,

// }

pub struct Player {
    player_version: u8,

    swf: Arc<SwfMovie>,

    is_playing: bool,

    needs_render: bool,

    renderer: Renderer,

    video: Video,

    transform_stack: TransformStack,

    rng: SmallRng,

    frame_rate: f64,

    forced_frame_rate: bool,

    action_since_timeout_check: u16,

    frame_phase: FramePhase,

    /// 执行帧的时间预算。通过主机帧之间的时间流逝获得，通过执行 SWF 帧花费。
    /// 这就是我们如何支持自定义 SWF 帧频，并通过 "追赶"（最高可达 MAX_FRAMES_PER_TICK）来补偿微小的滞后。
    frame_accumulator: f64,
    recent_run_frame_timings: VecDeque<f64>,

    /// 伪造时间流逝，骗过手写忙环 FPS 限制器
    time_offset: u32,

    page_url: Option<String>,

    /// 当前实例 ID。用于生成默认的 `instanceN` 名称。
    instance_counter: i32,

    /// 下一个计时器启动前的剩余时间。
    time_til_new_timer: Option<f64>,

    /// SWF 启动的瞬间。
    start_time: Instant,

    /// 在引发 `Error::ExecutionTimeout` 之前可调用的最长时间。默认值为 15 秒，但可以更改。
    max_execution_duration: Duration,

    /// 这是一个弱引用，在不同的上下文中会被升级并传递给播放器的其他部分。
    /// 它可用于确保播放器在异步代码中的等待调用期间仍能继续运行。
    self_reference: Weak<Mutex<Self>>,

    /// 当前帧, 第一帧是1
    current_frame: Option<u16>,
    // load_behavior: LoadBehavior,

    // 兼容性规则
    //compatibility_rules: CompatibilityRules,
    stage: Stage,
}

impl Player {
    pub fn render(&mut self) {
        let mut update_context = UpdateContext {
            renderer: &mut *self.renderer,
            video: &mut *self.video,
            rng: &mut self.rng,
            stage: self.stage.clone(),
            player: self.self_reference.clone(),
            instance_counter: &mut self.instance_counter,
            start_time: self.start_time,
            update_start: Instant::now(),
            max_execution_duration: self.max_execution_duration,
            frame_rate: &mut self.frame_rate,
            forced_frame_rate: self.forced_frame_rate,
            frame_phase: &mut self.frame_phase,
        };
        // let prev_frame_rate = update_context.frame_rate;
        let mut cache_draws = Vec::new();
        let mut render_context = RenderContext {
            renderer: &mut *self.renderer,
            commands: CommandList::new(),
            cache_draws: &mut cache_draws,
            transform_stack: &mut self.transform_stack,
            is_offscreen: false,
            stage: self.stage.clone(),
        };
        let background_color =
                if self.stage.window_mode() != WindowMode::Transparent || self.stage.is_full_screen() {
                    self.stage.background_color().unwrap_or(Color::WHITE)
                } else {
                    Color::from_rgba(0)
                };
        self.stage.render(&mut render_context);
        dbg!("render");
        render_context.renderer.submit_frame(background_color, render_context.commands, Vec::new());
    }

    pub fn mutate_with_update_context(&mut self, _context: &mut UpdateContext) {
        // todo!("mutate_with_update_context")
        dbg!("mutate_with_update_context");
    }

    pub fn set_quality(&mut self, quality: StageQuality) {
        // todo!("set_quality")
        dbg!("set_quality");
    }
    pub fn set_letterbox(&mut self, letterbox: Letterbox) {
        // todo!("set_letterbox")
        dbg!("set_letterbox");
    }
    pub fn set_viewport_dimensions(&mut self, dimensions: ViewportDimensions) {
        dbg!("set_viewport_dimensions");
    }

    pub fn set_root_movie(&mut self, movie: SwfMovie) {
        self.swf = Arc::new(movie);
        MovieClip::new(self.swf.clone()).load();
    }

    pub fn load_root_movie(&mut self, url: &Url) {
        dbg!("fetch_root_movie");
        Loader::root_movie_loader(&url, self).unwrap();
    }
}

pub struct PlayerBuilder {
    player_version: u8,
    swf_movie: Option<SwfMovie>,
    renderer: Option<Renderer>,
    video: Option<Video>,
    frame_rate: Option<f64>,
    forced_frame_rate: bool,
    max_execution_duration: Duration,
    page_url: Option<String>,

    auto_play: bool,
    allow_full_screen: bool,
    viewport_width: u32,
    viewport_height: u32,
    viewport_scale_factor: f64,
}

impl PlayerBuilder {
    #[inline]
    pub fn new() -> Self {
        PlayerBuilder {
            player_version: 0,
            swf_movie: None,
            renderer: None,
            video: None,
            frame_rate: None,
            forced_frame_rate: false,
            max_execution_duration: Duration::from_secs(15),
            page_url: None,
            auto_play: false,
            allow_full_screen: false,
            // 如果没有提供renderer 默认视口大小
            viewport_width: 550,
            // 默认视口高度
            viewport_height: 400,
            viewport_scale_factor: 1.0,
        }
    }
    #[inline]
    pub fn with_renderer(mut self, renderer: impl 'static + RenderBackend) -> Self {
        self.renderer = Some(Box::new(renderer));
        self
    }
    #[inline]
    pub fn with_auto_play(mut self, auto_play: bool) -> Self {
        self.auto_play = auto_play;
        self
    }

    #[inline]
    pub fn with_max_execution_duration(mut self, max_execution_duration: Duration) -> Self {
        self.max_execution_duration = max_execution_duration;
        self
    }

    // #[inline]
    // pub fn width_quality(mut self)

    #[inline]
    pub fn with_full_screen(mut self, allow_full_screen: bool) -> Self {
        self.allow_full_screen = allow_full_screen;
        self
    }

    #[inline]
    pub fn with_frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = Some(frame_rate);
        self
    }

    #[inline]
    pub fn with_player_version(mut self, player_version: u8) -> Self {
        self.player_version = player_version;
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
        let video = self
            .video
            .unwrap_or_else(|| Box::new(videoNull::NullVideoBackend::new()));
        let player_version = self.player_version;
        let fake_movie = Arc::new(SwfMovie::new_empty(player_version));
        let frame_rate = self.frame_rate.unwrap_or(24.0);
        let forced_frame_rate = self.forced_frame_rate;
        let stage = Stage::new_empty(true, fake_movie.clone());
        let player = Arc::new_cyclic(|self_ref| {
            Mutex::new(Player {
                renderer,
                video,
                swf: fake_movie.clone(),
                current_frame: None,
                frame_rate,
                forced_frame_rate,
                frame_accumulator: 0.0,
                recent_run_frame_timings: VecDeque::with_capacity(10),
                start_time: Instant::now(),
                time_offset: 0,
                time_til_new_timer: None,
                max_execution_duration: self.max_execution_duration,
                action_since_timeout_check: 0,

                player_version,
                needs_render: true,
                is_playing: self.auto_play,
                rng: SmallRng::seed_from_u64(get_current_date_time().timestamp_millis() as u64),
                page_url: self.page_url.clone(),
                instance_counter: 0,
                self_reference: self_ref.clone(),
                transform_stack: TransformStack::new(),
                frame_phase: Default::default(),

                stage,
            })
        });
        // 最终配置并加载movie
        let mut player_lock = player.lock().unwrap();

        player_lock.set_letterbox(Letterbox::FullScreen);
        player_lock.set_quality(quality::StageQuality::High);
        player_lock.set_viewport_dimensions(ViewportDimensions {
            width: self.viewport_width,
            height: self.viewport_height,
            scale_factor: self.viewport_scale_factor,
        });
        if let Some(movie) = self.swf_movie {
            player_lock.set_root_movie(movie);
        }
        drop(player_lock);
        player
    }
}
