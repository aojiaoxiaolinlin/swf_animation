use std::{
    cell::RefCell,
    fmt::{self, Debug, Display, Formatter},
    rc::Rc,
    str::FromStr,
    sync::Arc,
};

use ruffle_render::{matrix::Matrix, quality::StageQuality, transform::Transform};
use ruffle_wstr::{FromWStr, WStr};
use swf::{Color, Rectangle, Twips};

use crate::{config::Letterbox, context::RenderContext, tag_utils::SwfMovie};

use super::{container::ChildContainer, interactive::InteractiveObjectBase};

#[derive(Debug, Clone)]
pub struct StageData {
    base: InteractiveObjectBase,
    child: ChildContainer,
    background_color: Option<Color>,
    letterbox: Letterbox,
    swf_movie_size: (u32, u32),
    quality: StageQuality,
    stage_size: (u32, u32),
    scale_mode: StageScaleMode,
    forced_scale_mode: bool,
    display_state: StageDisplayState,

    /// 下一次渲染时，是否发送 RENDER 事件。
    invalidated: bool,

    use_bitmap_downsampling: bool,

    /// 当前视口的边界。用于剔除
    view_bounds: Rectangle<Twips>,

    /// 视口的窗口模式
    /// 仅用于网页，以控制 Flash 内容如何与页面上的其他内容分层。
    window_mode: WindowMode,
    /// 对象对焦时是否显示发光边框。
    stage_focus_rect: bool,

    /// 渲染舞台时应用的最终视口变换矩阵。其中包括 HiDPI 缩放因子和舞台对齐平移。这些都不包括在 ActionScript 公开的 Stage.matrix 中
    /// （除非通过 ActionScript 明确设置，否则 Stage.matrix 始终是标识矩阵）
    swf_movie: Arc<SwfMovie>,

    viewport_matrix: Matrix,
}

#[derive(Clone)]
pub struct Stage(Rc<RefCell<StageData>>);

impl Debug for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stage")
            .field("ptr", &self.0.borrow().swf_movie.header())
            .finish()
    }
}

impl Stage {
    pub fn new_empty(full_screen: bool, swf_movie: Arc<SwfMovie>) -> Self {
        let stage = Self(Rc::new(RefCell::new(StageData {
            base: Default::default(),
            child: ChildContainer::new(swf_movie.clone()),
            letterbox: Letterbox::FullScreen,
            swf_movie_size: (0, 0),
            quality: Default::default(),
            stage_size: (0, 0),
            scale_mode: Default::default(),
            forced_scale_mode: false,
            display_state: if full_screen {
                StageDisplayState::FullScreen
            } else {
                StageDisplayState::Normal
            },
            invalidated: false,
            use_bitmap_downsampling: false,
            view_bounds: Default::default(),
            window_mode: Default::default(),
            stage_focus_rect: true,
            swf_movie,
            viewport_matrix: Matrix::IDENTITY,
            background_color: None,
        })));
        stage.set_is_root(true);
        stage
    }

    /// 设置此显示对象是否代表已加载内容的根。
    fn set_is_root(&self, is_root: bool) {
        // self.0.base.set_is_root(is_root);
    }
    pub fn background_color(&self) -> Option<Color> {
        self.0.borrow().background_color
    }
    pub fn set_background_color(&mut self, color: Option<Color>) {
        self.0.borrow_mut().background_color = color;
    }
    pub fn letterbox(&self) -> Letterbox {
        self.0.borrow().letterbox
    }
    pub fn set_letterbox(&mut self, letterbox: Letterbox) {
        self.0.borrow_mut().letterbox = letterbox;
    }
    pub fn swf_movie_size(&self) -> (u32, u32) {
        self.0.borrow().swf_movie_size
    }
    pub fn set_swf_movie_size(&mut self, size: (u32, u32)) {
        self.0.borrow_mut().swf_movie_size = size;
    }
    pub fn set_swf_movie(&mut self, swf_movie: Arc<SwfMovie>) {
        self.0.borrow_mut().swf_movie = swf_movie.clone();
        self.0.borrow_mut().child.set_swf_movie(swf_movie);
    }

    pub fn quality(self) -> StageQuality {
        self.0.borrow().quality
    }

    pub fn set_quality(&mut self, quality: StageQuality) {
        self.0.borrow_mut().quality = quality;
    }

    pub fn state_size(&self) -> (u32, u32) {
        self.0.borrow().stage_size
    }
    pub fn scale_mode(&self) -> StageScaleMode {
        self.0.borrow().scale_mode
    }
    pub fn set_scale_mode(&mut self, scale_mode: StageScaleMode) {
        self.0.borrow_mut().scale_mode = scale_mode;
    }
    pub fn forced_scale_mode(&self) -> bool {
        self.0.borrow().forced_scale_mode
    }
    pub fn set_forced_scale_mode(&mut self, forced_scale_mode: bool) {
        self.0.borrow_mut().forced_scale_mode = forced_scale_mode;
    }
    pub fn display_state(&self) -> StageDisplayState {
        self.0.borrow().display_state
    }
    pub fn set_display_state(&mut self, display_state: StageDisplayState) {
        self.0.borrow_mut().display_state = display_state;
    }
    pub fn invalidated(&self) -> bool {
        self.0.borrow().invalidated
    }
    pub fn set_invalidated(&mut self, invalidated: bool) {
        self.0.borrow_mut().invalidated = invalidated;
    }
    pub fn is_full_screen(&self) -> bool {
        self.display_state() == StageDisplayState::FullScreen
            || self.display_state() == StageDisplayState::FullScreenInteractive
    }

    pub fn window_mode(&self) -> WindowMode {
        self.0.borrow().window_mode
    }
    pub fn set_window_mode(&mut self, window_mode: WindowMode) {
        self.0.borrow_mut().window_mode = window_mode;
    }
    pub fn view_bounds(&self) -> Rectangle<Twips> {
        self.0.borrow().view_bounds.clone()
    }
    /// Determine if we should letterbox the stage content.
    fn should_letterbox(self) -> bool {
        // Only enable letterbox in the default `ShowAll` scale mode.
        // If content changes the scale mode or alignment, it signals that it is size-aware.
        // For example, `NoScale` is used to make responsive layouts; don't letterbox over it.
        let stage = self.0.borrow_mut();
        stage.scale_mode == StageScaleMode::ShowAll
            && stage.window_mode != WindowMode::Transparent
            && (stage.letterbox == Letterbox::On
                || (stage.letterbox == Letterbox::FullScreen && self.is_full_screen()))
    }

    pub fn render(&self,context:&mut RenderContext){
        context.transform_stack.push(&Transform{
            matrix:self.0.borrow().viewport_matrix,
            color_transform:Default::default(),
        });
        context.transform_stack.pop();
    }

    fn build_matrices(self) {
        todo!("build_matrices")
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Copy)]
pub enum StageScaleMode {
    /// The movie will be stretched to fit the container.
    /// zh-cn: 电影将被拉伸以适应容器。
    ExactFit,

    /// The movie will maintain its aspect ratio, but will be cropped.
    /// zh-cn: 电影将保持其纵横比，但将被裁剪。
    NoBorder,

    /// The movie is not scaled to fit the container.
    /// With this scale mode, `Stage.stageWidth` and `stageHeight` will return the dimensions of the container.
    /// SWF content uses this scale mode to resize dynamically and create responsive layouts.
    /// zh-cn: 电影不会被缩放以适应容器。
    /// 使用此缩放模式，`Stage.stageWidth` 和 `stageHeight` 将返回容器的尺寸。
    /// SWF 内容使用此缩放模式来动态调整大小并创建响应式布局。
    NoScale,

    /// The movie will scale to fill the container and maintain its aspect ratio, but will be letterboxed.
    /// This is the default scale mode.
    /// zh-cn: 电影将缩放以填充容器并保持其纵横比，但将添加黑边。
    /// 这是默认的缩放模式。
    #[default]
    ShowAll,
}

impl Display for StageScaleMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Match string values returned by AS.
        let s = match *self {
            StageScaleMode::ExactFit => "exactFit",
            StageScaleMode::NoBorder => "noBorder",
            StageScaleMode::NoScale => "noScale",
            StageScaleMode::ShowAll => "showAll",
        };
        f.write_str(s)
    }
}
pub struct ParseEnumError;
impl FromStr for StageScaleMode {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let scale_mode = match s.to_ascii_lowercase().as_str() {
            "exactfit" => StageScaleMode::ExactFit,
            "noborder" => StageScaleMode::NoBorder,
            "noscale" => StageScaleMode::NoScale,
            "showall" => StageScaleMode::ShowAll,
            _ => return Err(ParseEnumError),
        };
        Ok(scale_mode)
    }
}

impl FromWStr for StageScaleMode {
    type Err = ParseEnumError;

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s.eq_ignore_case(WStr::from_units(b"exactfit")) {
            Ok(StageScaleMode::ExactFit)
        } else if s.eq_ignore_case(WStr::from_units(b"noborder")) {
            Ok(StageScaleMode::NoBorder)
        } else if s.eq_ignore_case(WStr::from_units(b"noscale")) {
            Ok(StageScaleMode::NoScale)
        } else if s.eq_ignore_case(WStr::from_units(b"showall")) {
            Ok(StageScaleMode::ShowAll)
        } else {
            Err(ParseEnumError)
        }
    }
}

/// The scale mode of a stage.
/// This controls the behavior when the player viewport size differs from the SWF size.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageDisplayState {
    /// Sets AIR application or content in Flash Player to expand the stage over the user's entire screen.
    /// Keyboard input is disabled, with the exception of a limited set of non-printing keys.
    FullScreen,

    /// Sets the application to expand the stage over the user's entire screen, with keyboard input allowed.
    /// (Available in AIR and Flash Player, beginning with Flash Player 11.3.)
    FullScreenInteractive,

    /// Sets the stage back to the standard stage display mode.
    #[default]
    Normal,
}

impl Display for StageDisplayState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Match string values returned by AS.
        let s = match *self {
            StageDisplayState::FullScreen => "fullScreen",
            StageDisplayState::FullScreenInteractive => "fullScreenInteractive",
            StageDisplayState::Normal => "normal",
        };
        f.write_str(s)
    }
}

impl FromStr for StageDisplayState {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let display_state = match s.to_ascii_lowercase().as_str() {
            "fullscreen" => StageDisplayState::FullScreen,
            "fullscreeninteractive" => StageDisplayState::FullScreenInteractive,
            "normal" => StageDisplayState::Normal,
            _ => return Err(ParseEnumError),
        };
        Ok(display_state)
    }
}

impl FromWStr for StageDisplayState {
    type Err = ParseEnumError;

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s.eq_ignore_case(WStr::from_units(b"fullscreen")) {
            Ok(StageDisplayState::FullScreen)
        } else if s.eq_ignore_case(WStr::from_units(b"fullscreeninteractive")) {
            Ok(StageDisplayState::FullScreenInteractive)
        } else if s.eq_ignore_case(WStr::from_units(b"normal")) {
            Ok(StageDisplayState::Normal)
        } else {
            Err(ParseEnumError)
        }
    }
}

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowMode {
    /// The Flash content is rendered in its own window and layering is done with the browser's
    /// default behavior.
    ///
    /// In Ruffle, this mode functions like `WindowMode::Opaque` and will layer the Flash content
    /// together with other HTML elements.
    #[default]
    Window,

    /// The Flash content is layered together with other HTML elements, and the stage color is
    /// opaque. Content can render above or below Ruffle based on CSS rendering order.
    Opaque,

    /// The Flash content is layered together with other HTML elements, and the stage color is
    /// transparent. Content beneath Ruffle will be visible through transparent areas.
    Transparent,

    /// Request compositing with hardware acceleration when possible.
    ///
    /// This mode has no effect in Ruffle and will function like `WindowMode::Opaque`.
    Gpu,

    /// Request a direct rendering path, bypassing browser compositing when possible.
    ///
    /// This mode has no effect in Ruffle and will function like `WindowMode::Opaque`.
    Direct,
}

impl Display for WindowMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match *self {
            WindowMode::Window => "window",
            WindowMode::Opaque => "opaque",
            WindowMode::Transparent => "transparent",
            WindowMode::Direct => "direct",
            WindowMode::Gpu => "gpu",
        };
        f.write_str(s)
    }
}

impl FromStr for WindowMode {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let window_mode = match s.to_ascii_lowercase().as_str() {
            "window" => WindowMode::Window,
            "opaque" => WindowMode::Opaque,
            "transparent" => WindowMode::Transparent,
            "direct" => WindowMode::Direct,
            "gpu" => WindowMode::Gpu,
            _ => return Err(ParseEnumError),
        };
        Ok(window_mode)
    }
}
