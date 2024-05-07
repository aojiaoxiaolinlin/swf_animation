pub mod movie_clip;
pub mod morph_shape;
pub mod graphic;
use swf::{Rectangle, Twips};

pub trait TDisplayObject {
    // fn base_mut(&mut self) -> &mut DisplayObjectBase;
    fn set_scaling_grid(&self,rect:Rectangle<Twips>){
        // self.base_mut()
    }
}