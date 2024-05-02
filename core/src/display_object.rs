mod container;
mod interactive;
mod movie_clip;
pub mod stage;
mod graphic;

use bitflags::bitflags;
use ruffle_macros::enum_trait_object;
use ruffle_render::{
    backend::RenderBackend,
    bitmap::{BitmapHandle, BitmapInfo},
    blend::ExtendedBlendMode,
    filters::Filter,
    matrix::Matrix,
    pixel_bender::PixelBenderShaderHandle,
    transform::Transform,
};
use std::fmt::Debug;
use swf::{Color, Depth, Point, Rectangle, Twips};

use crate::types::{Degrees, Percent};

use self::stage::Stage;
#[enum_trait_object(
    #[derive(Debug,Clone)]
    pub enum DisplayObject{
        Stage(Stage),
        // Bitmap(Bitmap),
        // EditText(EditText),
        // Graphic(Graphic),
        // MorphShape(MorphShape),
        // MovieClip(MovieClip),
        // Text(Text),
        // Video(Video),
        // LoaderDisplay(LoaderDisplay),
    }
)]
pub trait TDisplayObject: Clone + Debug + Into<DisplayObject> {
    // fn id(&self) -> CharacterId;
    // fn depth(&self) -> Depth {
    //     self.base().depth()
    // }
}
/// If a `DisplayObject` is marked `cacheAsBitmap` (via tag or AS),
/// this struct keeps the information required to uphold that cache.
/// A cached Display Object must have its bitmap invalidated when
/// any "visual" change happens, which can include:
/// - Changing the rotation
/// - Changing the scale
/// - Changing the alpha
/// - Changing the color transform
/// - Any "visual" change to children, **including** position changes
///
/// Position changes to the cached Display Object does not regenerate the cache,
/// allowing Display Objects to move freely without being regenerated.
///
/// Flash isn't very good at always recognising when it should be invalidated,
/// and there's cases such as changing the blend mode which don't always trigger it.
///
#[derive(Clone, Debug, Default)]
pub struct BitmapCache {
    /// The `Matrix.a` value that was last used with this cache
    matrix_a: f32,
    /// The `Matrix.b` value that was last used with this cache
    matrix_b: f32,
    /// The `Matrix.c` value that was last used with this cache
    matrix_c: f32,
    /// The `Matrix.d` value that was last used with this cache
    matrix_d: f32,

    /// The width of the original bitmap, pre-filters
    source_width: u16,

    /// The height of the original bitmap, pre-filters
    source_height: u16,

    /// The offset used to draw the final bitmap (i.e. if a filter increases the size)
    draw_offset: Point<i32>,

    /// The current contents of the cache, if any. Values are post-filters.
    bitmap: Option<BitmapInfo>,

    /// Whether we warned that this bitmap was too large to be cached
    warned_for_oversize: bool,
}

impl BitmapCache {
    /// Forcefully make this BitmapCache invalid and require regeneration.
    /// This should be used for changes that aren't automatically detected, such as children.
    pub fn make_dirty(&mut self) {
        // Setting the old transform to something invalid is a cheap way of making it invalid,
        // without reserving an extra field for.
        self.matrix_a = f32::NAN;
    }

    fn is_dirty(&self, other: &Matrix, source_width: u16, source_height: u16) -> bool {
        self.matrix_a != other.a
            || self.matrix_b != other.b
            || self.matrix_c != other.c
            || self.matrix_d != other.d
            || self.source_width != source_width
            || self.source_height != source_height
            || self.bitmap.is_none()
    }

    /// Clears any dirtiness and ensure there's an appropriately sized texture allocated
    #[allow(clippy::too_many_arguments)]
    fn update(
        &mut self,
        renderer: &mut dyn RenderBackend,
        matrix: Matrix,
        source_width: u16,
        source_height: u16,
        actual_width: u16,
        actual_height: u16,
        draw_offset: Point<i32>,
        swf_version: u8,
    ) {
        self.matrix_a = matrix.a;
        self.matrix_b = matrix.b;
        self.matrix_c = matrix.c;
        self.matrix_d = matrix.d;
        self.source_width = source_width;
        self.source_height = source_height;
        self.draw_offset = draw_offset;
        if let Some(current) = &mut self.bitmap {
            if current.width == actual_width && current.height == actual_height {
                return; // No need to resize it
            }
        }
        let acceptable_size = if swf_version > 9 {
            let total = actual_width as u32 * actual_height as u32;
            actual_width < 8191 && actual_height < 8191 && total < 16777215
        } else {
            actual_width < 2880 && actual_height < 2880
        };
        if renderer.is_offscreen_supported()
            && actual_width > 0
            && actual_height > 0
            && acceptable_size
        {
            let handle = renderer.create_empty_texture(actual_width as u32, actual_height as u32);
            self.bitmap = handle.ok().map(|handle| BitmapInfo {
                width: actual_width,
                height: actual_height,
                handle,
            });
        } else {
            self.bitmap = None;
        }
    }

    /// Explicitly clears the cached value and drops any resources.
    /// This should only be used in situations where you can't render to the cache and it needs to be
    /// temporarily disabled.
    fn clear(&mut self) {
        self.bitmap = None;
    }

    fn handle(&self) -> Option<BitmapHandle> {
        self.bitmap.as_ref().map(|b| b.handle.clone())
    }
}
#[derive(Debug, Clone)]
pub struct DisplayObjectBase {
    parent: Option<DisplayObject>,
    depth: Depth,
    place_frame: u16,
    transform: Transform,
    clip_depth: Option<Depth>,
    name: Option<String>,
    filters: Vec<Filter>,

    rotation: Degrees,
    scale_x: Percent,
    scale_y: Percent,

    skew: f64,

    masker: Option<DisplayObject>,

    masking: Option<DisplayObject>,

    /// 渲染此显示对象时使用的混合模式。
    /// 除默认 BlendMode::Normal 之外的其他值都会隐式地导致 "缓存即位图 "行为。
    blend_mode: ExtendedBlendMode,
    blend_shader: Option<PixelBenderShaderHandle>,
    /// 此显示对象的不透明背景颜色。显示对象的边界框将填充给定的颜色。这也会触发缓存即位图（cache-as-bitmap）行为。
    /// 仅支持纯色背景；alpha 通道将被忽略。
    opaque_background: Option<Color>,

    /// 各种显示对象属性的位标志。
    flags: DisplayObjectFlags,
    scroll_rect: Option<Rectangle<Twips>>,
    scaling_grid: Rectangle<Twips>,
    cache: Option<BitmapCache>,
}
impl Default for DisplayObjectBase {
    fn default() -> Self {
        Self {
            parent: None,
            depth: 0,
            place_frame: 0,
            transform: Default::default(),
            clip_depth: None,
            name: None,
            filters: Vec::new(),
            rotation: 0.0.into(),
            scale_x: 1.0.into(),
            scale_y: 1.0.into(),
            skew: 0.0,
            masker: None,
            masking: None,
            blend_mode: ExtendedBlendMode::Normal,
            blend_shader: None,
            opaque_background: None,
            flags: DisplayObjectFlags::empty(),
            scroll_rect: None,
            scaling_grid: Rectangle::default(),
            cache: None,
        }
    }
}
bitflags! {
    /// Bit flags used by `DisplayObject`.
    #[derive(Clone, Copy,Debug)]
    struct DisplayObjectFlags: u16 {
        /// Whether this object has been removed from the display list.
        /// Necessary in AVM1 to throw away queued actions from removed movie clips.
        const AVM1_REMOVED             = 1 << 0;

        /// If this object is visible (`_visible` property).
        const VISIBLE                  = 1 << 1;

        /// Whether the `_xscale`, `_yscale` and `_rotation` of the object have been calculated and cached.
        const SCALE_ROTATION_CACHED    = 1 << 2;

        /// Whether this object has been transformed by ActionScript.
        /// When this flag is set, changes from SWF `PlaceObject` tags are ignored.
        const TRANSFORMED_BY_SCRIPT    = 1 << 3;

        /// Whether this object has been placed in a container by ActionScript 3.
        /// When this flag is set, changes from SWF `RemoveObject` tags are ignored.
        const PLACED_BY_SCRIPT         = 1 << 4;

        /// Whether this object has been instantiated by a SWF tag.
        /// When this flag is set, attempts to change the object's name from AVM2 throw an exception.
        const INSTANTIATED_BY_TIMELINE = 1 << 5;

        /// Whether this object is a "root", the top-most display object of a loaded SWF or Bitmap.
        /// Used by `MovieClip.getBytesLoaded` in AVM1 and `DisplayObject.root` in AVM2.
        const IS_ROOT                  = 1 << 6;

        /// Whether this object has `_lockroot` set to true, in which case
        /// it becomes the _root of itself and of any children
        const LOCK_ROOT                = 1 << 7;

        /// Whether this object will be cached to bitmap.
        const CACHE_AS_BITMAP          = 1 << 8;

        /// Whether this object has a scroll rectangle applied.
        const HAS_SCROLL_RECT          = 1 << 9;

        /// Whether this object has an explicit name.
        const HAS_EXPLICIT_NAME        = 1 << 10;

        /// Flag set when we should skip running our next 'enterFrame'
        /// for ourself and our children.
        /// This is set for objects constructed from ActionScript,
        /// which are observed to lag behind objects placed by the timeline
        /// (even if they are both placed in the same frame)
        const SKIP_NEXT_ENTER_FRAME    = 1 << 11;

        /// If this object has already had `invalidate_cached_bitmap` called this frame
        const CACHE_INVALIDATED        = 1 << 12;

        /// If this AVM1 object is pending removal (will be removed on the next frame).
        const AVM1_PENDING_REMOVAL     = 1 << 13;
    }
}
