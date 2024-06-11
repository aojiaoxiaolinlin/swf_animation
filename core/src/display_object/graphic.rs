use ruffle_render::{backend::ShapeHandle, commands::CommandHandler};
use swf::{CharacterId, Rectangle, Shape, Twips};

use crate::{
    context::RenderContext,
    display_object::{DisplayObject, DisplayObjectBase, TDisplayObject},
    drawing::Drawing,
    library::{MovieLibrary, MovieLibrarySource},
};

#[derive(Clone)]
pub struct Graphic {
    pub id: CharacterId,
    shape: Shape,
    bounds: Rectangle<Twips>,
    base: DisplayObjectBase,
    drawing: Option<Drawing>,
}

impl Graphic {
    pub fn from_swf_tag(shape: Shape) -> Self {
        Self {
            id: shape.id,
            bounds: shape.shape_bounds.clone(),
            shape,
            base: DisplayObjectBase::default(),
            drawing: None,
        }
    }
}

impl TDisplayObject for Graphic {
    fn base_mut(&mut self) -> &mut DisplayObjectBase {
        &mut self.base
    }

    fn base(&self) -> &DisplayObjectBase {
        &self.base
    }

    fn character_id(&self) -> CharacterId {
        self.id
    }
    fn replace_with(&mut self, id: CharacterId, library: &mut MovieLibrary) {
        if let Some(new_graphic) = library.get_graphic(id) {
            self.id = new_graphic.id;
            self.shape = new_graphic.shape;
            self.bounds = new_graphic.bounds;
            self.base = new_graphic.base;
        } else {
            dbg!("PlaceObject: expected Graphic at character ID {}", id);
        }
    }

    fn render_self(&self, render_context: &mut RenderContext<'_>) {
        if !render_context.is_offscreen {
            return;
        }
        let render_handle = Some(render_context.renderer.register_shape(
            (&self.shape).into(),
            &MovieLibrarySource {
                library: render_context.library,
            },
        ));
        if let Some(drawing) = self.drawing.clone() {
            drawing.render(render_context);
        } else if let Some(render_handle) = render_handle.clone() {
            render_context
                .commands
                .render_shape(render_handle, render_context.transform_stack.transform())
        }
    }

    fn self_bounds(&self) -> Rectangle<Twips> {
        todo!()
    }
}
impl From<Graphic> for DisplayObject {
    fn from(graphic: Graphic) -> Self {
        DisplayObject::Graphic(graphic)
    }
}
