use bitflags::bitflags;
use ruffle_render::{matrix::Matrix, quality::StageQuality};
use ruffle_wstr::{FromWStr, WStr};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
    sync::Arc,
};
use swf::{Color, Rectangle, Twips};

use crate::{config::Letterbox, context::{self, UpdateContext}, tag_utils::SwfMovie};

use super::DisplayObjectBase;

pub struct Stage {
    base: DisplayObjectBase,

    background_color: Option<Color>,

    letterbox: Letterbox,

    movie_size: (u32, u32),

    quality: StageQuality,

    stage_size: (u32, u32),

    scale_mode: StageScaleMode,

    forced_scale_mode: bool,

    display_state: StageDisplayState,

    align: StageAlign,

    forced_align: bool,

    allow_full_screen: bool,

    invalidated: bool,

    use_bitmap_down_sampling: bool,

    view_bounds: Rectangle<Twips>,

    window_mode: WindowMode,

    stage_focus_rect: bool,

    movie: Arc<SwfMovie>,

    viewport_matrix: Matrix,
}

impl Stage {
    pub fn empty(full_screen: bool, movie: Arc<SwfMovie>) -> Self {
        Self {
            base: DisplayObjectBase::default(),
            background_color: None,
            letterbox: Letterbox::Fullscreen,
            movie_size: (0, 0),
            quality: StageQuality::High,
            stage_size: (0, 0),
            scale_mode: StageScaleMode::ShowAll,
            forced_scale_mode: false,
            display_state: if full_screen {
                StageDisplayState::FullScreen
            } else {
                StageDisplayState::Normal
            },
            align: StageAlign::default(),
            forced_align: false,
            allow_full_screen: true,
            invalidated: false,
            use_bitmap_down_sampling: false,
            view_bounds: Rectangle::default(),
            window_mode: WindowMode::Opaque,
            stage_focus_rect: false,
            movie,
            viewport_matrix: Matrix::IDENTITY,
        }
    }

    pub fn background_color(&self) -> Option<Color> {
        self.background_color
    }
    pub fn set_background_color(&mut self, color: Option<Color>) {
        self.background_color = color;
    }
    pub fn view_matrix(self) -> Matrix {
        self.viewport_matrix
    }
    pub fn letterbox(self) -> Letterbox {
        self.letterbox
    }
    pub fn set_letterbox(&mut self, letterbox: Letterbox) {
        self.letterbox = letterbox;
    }
    pub fn movie_size(self) -> (u32, u32) {
        self.movie_size
    }
    pub fn set_movie_size(&mut self, size: (u32, u32)) {
        self.movie_size = size;
    }
    pub fn set_movie(&mut self, movie: Arc<SwfMovie>) {
        self.movie = movie;
    }
    pub fn invalidated(self) -> bool {
        self.invalidated
    }
    pub fn set_invalidated(&mut self, invalidated: bool) {
        self.invalidated = invalidated;
    }

    pub fn quality(self) -> StageQuality {
        self.quality
    }
    pub fn set_quality(&mut self,context:&mut UpdateContext, quality: StageQuality) {
        self.quality = quality;
        self.use_bitmap_down_sampling = matches!(
            quality,
            StageQuality::Best | StageQuality::High8x8 | StageQuality::High16x16|
            StageQuality::High8x8Linear | StageQuality::High16x16Linear
        );
        // context.renderer.set_quality(quality);
    }
    pub fn stage_focus_rect(self) -> bool {
        self.stage_focus_rect
    }
    pub fn set_stage_focus_rect(&mut self, stage_focus_rect: bool) {
        self.stage_focus_rect = stage_focus_rect;
    }
    pub fn stage_size(self) -> (u32, u32) {
        self.stage_size
    }
    pub fn scale_mode(self) -> StageScaleMode {
        self.scale_mode
    }
    pub fn set_scale_mode(&mut self,context:&mut UpdateContext, scale_mode: StageScaleMode) {
        if !self.forced_scale_mode(){
            self.scale_mode = scale_mode;
            self.build_matrices(context);
        }
    }
    fn forced_scale_mode(&self) -> bool {
        self.forced_scale_mode
    }
    pub fn set_forced_scale_mode(&mut self, forced_scale_mode: bool) {
        self.forced_scale_mode = forced_scale_mode;
    }

    pub fn align(self) -> StageAlign {
        self.align
    }
    pub fn set_align(&mut self,context:&mut UpdateContext, align: StageAlign) {
        if !self.forced_align(){
            self.align = align;
            self.build_matrices(context);
        }
    }
    fn forced_align(&self) -> bool {
        self.forced_align
    }
    pub fn set_forced_align(&mut self, forced_align: bool) {
        self.forced_align = forced_align;
    }
    pub fn use_bitmap_down_sampling(self) -> bool {
        self.use_bitmap_down_sampling
    }
    pub fn set_use_bitmap_down_sampling(&mut self, use_bitmap_down_sampling: bool) {
        self.use_bitmap_down_sampling = use_bitmap_down_sampling;
    }
    pub fn window_mode(self) -> WindowMode {
        self.window_mode
    }
    pub fn set_window_mode(&mut self, window_mode: WindowMode) {
        self.window_mode = window_mode;
    }
    pub fn is_full_screen(self) -> bool {
        let display_state = self.display_state;
        Self::is_fullscreen_state(display_state)
    }
    fn is_fullscreen_state(display_state: StageDisplayState) -> bool {
        display_state == StageDisplayState::FullScreen
            || display_state == StageDisplayState::FullScreenInteractive
    }
    fn should_letterbox(self) -> bool {
        self.scale_mode == StageScaleMode::ShowAll
        && self.align.is_empty()
        && self.window_mode != WindowMode::Transparent
        && (self.letterbox == Letterbox::On
            || (self.letterbox == Letterbox::Fullscreen && self.is_full_screen()))
    }
    pub fn build_matrices(&mut self, context:&mut UpdateContext){
        let scale_mode = self.scale_mode;
        let align = self.align;
        let prev_stage_size = self.stage_size;
        let viewport_size = context.renderer.viewport_dimensions();

        self.stage_size = if self.scale_mode == StageScaleMode::NoScale {
            let width = f64::from(viewport_size.width)/viewport_size.scale_factor;
            let height = f64::from(viewport_size.height)/viewport_size.scale_factor;
            (width as u32, height as u32)
        }else{
            self.movie_size
        };

        let stage_size_changed = prev_stage_size != self.stage_size;
        let (movie_width, movie_height) = self.movie_size;
        let movie_width = movie_width as f64;
        let movie_height = movie_height as f64;

        let viewport_width = viewport_size.width as f64;
        let viewport_height = viewport_size.height as f64;

        let movie_aspect = movie_width / movie_height;
        let viewport_aspect = viewport_width / viewport_height;

        let (scale_x,scale_y) = match scale_mode {
            StageScaleMode::ShowAll => {
                // Keep aspect ratio, padding the edges.
                let scale = if viewport_aspect > movie_aspect {
                    viewport_height / movie_height
                } else {
                    viewport_width / movie_width
                };
                (scale, scale)
            }
            StageScaleMode::NoBorder => {
                 // Keep aspect ratio, cropping off the edges.
                let scale = if viewport_aspect < movie_aspect {
                    viewport_height / movie_height
                } else {
                    viewport_width / movie_width
                };
                (scale, scale)
            }
            StageScaleMode::ExactFit => (viewport_width / movie_width, viewport_height / movie_height),

            StageScaleMode::NoScale => (viewport_size.scale_factor, viewport_size.scale_factor),
        };

        let width_delta = viewport_width - movie_width * scale_x;
        let height_delta = viewport_height - movie_height * scale_y;

        // The precedence is important here to match Flash behavior.
        // L > R > "", T > B > "".
        let tx = if align.contains(StageAlign::LEFT) {
            0.0
        } else if align.contains(StageAlign::RIGHT) {
            width_delta
        } else {
            width_delta / 2.0
        };
        let ty = if align.contains(StageAlign::TOP) {
            0.0
        } else if align.contains(StageAlign::BOTTOM) {
            height_delta
        } else {
            height_delta / 2.0
        };
        self.viewport_matrix = Matrix {
            a: scale_x as f32,
            b: 0.0,
            c: 0.0,
            d: scale_y as f32,
            tx: Twips::from_pixels(tx),
            ty: Twips::from_pixels(ty),
        };
        self.view_bounds = if self.should_letterbox() {
            // Letterbox: movie area
            Rectangle {
                x_min: Twips::ZERO,
                y_min: Twips::ZERO,
                x_max: Twips::from_pixels(movie_width),
                y_max: Twips::from_pixels(movie_height),
            }
        } else {
            // No letterbox: full visible stage area
            let margin_left = tx / scale_x;
            let margin_right = (width_delta - tx) / scale_x;
            let margin_top = ty / scale_y;
            let margin_bottom = (height_delta - ty) / scale_y;
            Rectangle {
                x_min: Twips::from_pixels(-margin_left),
                y_min: Twips::from_pixels(-margin_top),
                x_max: Twips::from_pixels(movie_width + margin_right),
                y_max: Twips::from_pixels(movie_height + margin_bottom),
            }
        };

        // Fire resize handler if stage size has changed.
        if scale_mode == StageScaleMode::NoScale && stage_size_changed {
            // self.fire_resize_event(context);
        }
    }   
}

pub struct ParseEnumError;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageScaleMode {
    /// The movie will be stretched to fit the container.
    ExactFit,

    /// The movie will maintain its aspect ratio, but will be cropped.
    NoBorder,

    /// The movie is not scaled to fit the container.
    /// With this scale mode, `Stage.stageWidth` and `stageHeight` will return the dimensions of the container.
    /// SWF content uses this scale mode to resize dynamically and create responsive layouts.
    NoScale,

    /// The movie will scale to fill the container and maintain its aspect ratio, but will be letterboxed.
    /// This is the default scale mode.
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

bitflags! {
    /// The alignment of the stage.
    /// This controls the position of the movie after scaling to fill the viewport.
    /// The default alignment is centered (no bits set).
    ///
    /// This is a bitflags instead of an enum to mimic Flash Player behavior.
    /// You can theoretically have both TOP and BOTTOM bits set, for example.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct StageAlign: u8 {
        /// Align to the top of the viewport.
        const TOP    = 1 << 0;

        /// Align to the bottom of the viewport.
        const BOTTOM = 1 << 1;

        /// Align to the left of the viewport.
        const LEFT   = 1 << 2;

        /// Align to the right of the viewport.;
        const RIGHT  = 1 << 3;
    }
}

impl FromStr for StageAlign {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Chars get converted into flags.
        // This means "tbbtlbltblbrllrbltlrtbl" is valid, resulting in "TBLR".
        let mut align = StageAlign::default();
        for c in s.bytes().map(|c| c.to_ascii_uppercase()) {
            match c {
                b'T' => align.insert(StageAlign::TOP),
                b'B' => align.insert(StageAlign::BOTTOM),
                b'L' => align.insert(StageAlign::LEFT),
                b'R' => align.insert(StageAlign::RIGHT),
                _ => (),
            }
        }
        Ok(align)
    }
}

impl FromWStr for StageAlign {
    type Err = std::convert::Infallible;

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        // Chars get converted into flags.
        // This means "tbbtlbltblbrllrbltlrtbl" is valid, resulting in "TBLR".
        let mut align = StageAlign::default();
        for c in s.iter() {
            match u8::try_from(c).map(|c| c.to_ascii_uppercase()) {
                Ok(b'T') => align.insert(StageAlign::TOP),
                Ok(b'B') => align.insert(StageAlign::BOTTOM),
                Ok(b'L') => align.insert(StageAlign::LEFT),
                Ok(b'R') => align.insert(StageAlign::RIGHT),
                _ => (),
            }
        }
        Ok(align)
    }
}

/// The window mode of the Ruffle player.
///
/// This setting controls how the Ruffle container is layered and rendered with other content on
/// the page. This setting is only used on web.
///
/// [Apply OBJECT and EMBED tag attributes in Adobe Flash Professional](https://helpx.adobe.com/flash/kb/flash-object-embed-tag-attributes.html)
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
