pub(crate) mod graphic;
pub mod movie_clip;
use std::{cell::RefCell, sync::Arc};

use bitflags::bitflags;
use graphic::Graphic;
use movie_clip::MovieClip;

use ruffle_render::{
    blend::ExtendedBlendMode, filters::Filter, matrix::Matrix, transform::Transform,
};
use swf::{CharacterId, Color, ColorTransform, Depth};

use super::{library::MovieLibrary, tag_utils::SwfMovie};

bitflags! {
    /// Bit flags used by `DisplayObject`.
    #[derive(Clone, Copy)]
    struct DisplayObjectFlags: u16 {
        /// If this object is visible (`_visible` property).
        const VISIBLE                  = 1 << 0;

        /// Whether this object is a "root", the top-most display object of a loaded SWF or Bitmap.
        /// Used by `MovieClip.getBytesLoaded` in AVM1 and `DisplayObject.root` in AVM2.
        const IS_ROOT                  = 1 << 1;

        /// Whether this object will be cached to bitmap.
        const CACHE_AS_BITMAP          = 1 << 2;

        /// If this object has already had `invalidate_cached_bitmap` called this frame
        const CACHE_INVALIDATED        = 1 << 3;
    }
}

#[derive(Clone)]
pub struct DisplayObjectBase {
    place_frame: u16,
    depth: Depth,
    clip_depth: Depth,
    name: Option<String>,
    transform: Transform,
    blend_mode: ExtendedBlendMode,
    flags: DisplayObjectFlags,
    // scaling_grid: Rectangle<Twips>,
    opaque_background: Option<Color>,
    filters: Vec<Filter>,
}
unsafe impl Send for DisplayObjectBase {}
unsafe impl Sync for DisplayObjectBase {}

impl Default for DisplayObjectBase {
    fn default() -> Self {
        Self {
            place_frame: Default::default(),
            depth: Default::default(),
            clip_depth: Default::default(),
            name: None,
            transform: Default::default(),
            blend_mode: Default::default(),
            opaque_background: Default::default(),
            flags: DisplayObjectFlags::VISIBLE,
            filters: Default::default(),
            // scaling_grid: Default::default(),
        }
    }
}
impl DisplayObjectBase {
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    fn blend_mode(&self) -> ExtendedBlendMode {
        self.blend_mode
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn matrix(&self) -> &Matrix {
        &self.transform.matrix
    }

    fn filters(&self) -> Vec<Filter> {
        self.filters.clone()
    }

    fn visible(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::VISIBLE)
    }

    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }

    fn set_depth(&mut self, depth: Depth) {
        self.depth = depth;
    }

    fn set_matrix(&mut self, matrix: Matrix) {
        self.transform.matrix = matrix;
    }

    pub fn set_color_transform(&mut self, color_transform: ColorTransform) {
        self.transform.color_transform = color_transform;
    }

    pub fn set_is_root(&mut self, value: bool) {
        self.flags.set(DisplayObjectFlags::IS_ROOT, value);
    }

    fn set_blend_mode(&mut self, value: ExtendedBlendMode) -> bool {
        let changed = self.blend_mode != value;
        self.blend_mode = value;
        changed
    }
    fn set_filters(&mut self, filters: Vec<Filter>) -> bool {
        if filters != self.filters {
            self.filters = filters;
            true
        } else {
            false
        }
    }

    fn set_visible(&mut self, visible: bool) -> bool {
        let changed = self.visible() != visible;
        self.flags.set(DisplayObjectFlags::VISIBLE, visible);
        changed
    }

    fn set_place_frame(&mut self, frame: u16) {
        self.place_frame = frame;
    }

    fn set_clip_depth(&mut self, depth: Depth) {
        self.clip_depth = depth;
    }

    fn is_root(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::IS_ROOT)
    }
}

#[derive(Clone)]
pub enum DisplayObject {
    MovieClip(Arc<RefCell<MovieClip>>),
    Graphic(Arc<RefCell<Graphic>>),
}

pub trait TDisplayObject: Clone + Into<Arc<RefCell<DisplayObject>>> {
    fn base(&self) -> Arc<RefCell<DisplayObjectBase>>;
    fn movie(&self) -> Arc<SwfMovie>;
    fn character_id(&self) -> CharacterId;
    fn depth(&self) -> Depth {
        self.base().borrow().depth
    }
    fn transform(&self) -> Transform {
        self.base().borrow().transform.clone()
    }
    fn place_frame(&self) -> u16 {
        self.base().borrow().place_frame
    }

    fn is_root(&self) -> bool {
        self.base().borrow().is_root()
    }
    fn filters(&self) -> Vec<Filter> {
        self.base().borrow().filters()
    }
    fn opaque_background(&self) -> Option<Color> {
        self.base().borrow().opaque_background
    }
    fn allow_as_mask(&self) -> bool {
        true
    }
    fn visible(&self) -> bool {
        self.base().borrow().visible()
    }
    fn name(&self) -> Option<String> {
        self.base().borrow().name().map(|item| item.to_owned())
    }

    fn blend_mode(&self) -> ExtendedBlendMode {
        self.base().borrow().blend_mode()
    }

    fn set_name(&mut self, name: Option<String>) {
        self.base().borrow_mut().set_name(name);
    }
    fn set_clip_depth(&mut self, depth: Depth) {
        self.base().borrow_mut().set_clip_depth(depth);
    }
    fn set_matrix(&mut self, matrix: Matrix) {
        self.base().borrow_mut().set_matrix(matrix);
    }
    fn set_color_transform(&mut self, color_transform: ColorTransform) {
        self.base()
            .borrow_mut()
            .set_color_transform(color_transform);
    }
    fn set_blend_mode(&mut self, blend_mode: ExtendedBlendMode) {
        self.base().borrow_mut().set_blend_mode(blend_mode);
    }

    fn set_filters(&mut self, filters: Vec<Filter>) {
        self.base().borrow_mut().set_filters(filters);
    }
    fn set_visible(&mut self, visible: bool) {
        self.base().borrow_mut().set_visible(visible);
    }
    fn set_place_frame(&mut self, frame: u16) {
        self.base().borrow_mut().set_place_frame(frame);
    }
    fn set_depth(&mut self, depth: Depth) {
        self.base().borrow_mut().set_depth(depth);
    }
    fn set_is_root(&mut self, is_root: bool) {
        self.base().borrow_mut().set_is_root(is_root);
    }

    fn set_default_instance_name(&mut self, library: &mut MovieLibrary) {
        if self.base().borrow().name.is_none() {
            let name = format!("instance{}", library.instance_count);
            self.set_name(Some(name));
            library.instance_count = library.instance_count.wrapping_add(1);
        }
    }

    fn post_instantiation(&mut self, _library: &mut MovieLibrary) {}

    fn enter_frame(&mut self, _library: &mut MovieLibrary) {}

    fn replace_with(&mut self, _id: CharacterId, _library: &mut MovieLibrary) {}

    fn as_movie_clip(&self) -> Option<MovieClip> {
        None
    }

    fn as_graphic(&self) -> Option<Graphic> {
        None
    }

    fn apply_place_object(&mut self, place_object: &swf::PlaceObject, swf_version: u8) {
        if let Some(matrix) = place_object.matrix {
            self.set_matrix(matrix.into());
        }
        if let Some(color_transform) = &place_object.color_transform {
            self.set_color_transform(*color_transform);
        }

        if let Some(blend_mode) = place_object.blend_mode {
            self.set_blend_mode(blend_mode.into());
        }
        if swf_version >= 11 {
            if let Some(visible) = place_object.is_visible {
                self.set_visible(visible);
            }
        }
        if let Some(filters) = &place_object.filters {
            self.set_filters(filters.iter().map(Filter::from).collect())
        }
    }
}

impl From<DisplayObject> for Arc<RefCell<DisplayObject>> {
    fn from(display_object: DisplayObject) -> Self {
        Arc::new(RefCell::new(display_object))
    }
}

impl TDisplayObject for DisplayObject {
    fn base(&self) -> Arc<RefCell<DisplayObjectBase>> {
        match self {
            DisplayObject::MovieClip(movie_clip) => movie_clip.borrow().base().clone(),
            DisplayObject::Graphic(graphic) => graphic.borrow().base().clone(),
        }
    }

    fn movie(&self) -> Arc<SwfMovie> {
        match self {
            DisplayObject::MovieClip(movie_clip) => movie_clip.borrow().movie().clone(),
            DisplayObject::Graphic(graphic) => graphic.borrow().movie().clone(),
        }
    }

    fn character_id(&self) -> CharacterId {
        match self {
            DisplayObject::MovieClip(movie_clip) => movie_clip.borrow().character_id(),
            DisplayObject::Graphic(graphic) => graphic.borrow().character_id(),
        }
    }

    fn enter_frame(&mut self, library: &mut MovieLibrary) {
        match self {
            DisplayObject::MovieClip(movie_clip) => movie_clip.borrow_mut().enter_frame(library),
            _ => {}
        }
    }

    fn as_movie_clip(&self) -> Option<MovieClip> {
        match self {
            DisplayObject::MovieClip(movie_clip) => movie_clip.borrow().as_movie_clip(),
            _ => None,
        }
    }
}
