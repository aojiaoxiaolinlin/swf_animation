mod bitmap_data;
use ruffle_render::bitmap::{BitmapFormat, PixelSnapping};
use swf::CharacterId;

use self::bitmap_data::{BitmapDataWrapper, Color};

use super::{DisplayObjectBase, DisplayObjectWeak};

pub struct Bitmap{
    base:DisplayObjectBase,
    id:CharacterId,
    
    bitmap_data: BitmapDataWrapper,
    width: u32,
    height: u32,

    smoothing:bool,

    pixel_snapping:PixelSnapping
}

impl Bitmap{
    pub fn new_with_bitmap_data(id:CharacterId,bitmap_data:BitmapDataWrapper,smoothing:bool)->Self{
        let width = bitmap_data.width();
        let height = bitmap_data.height();
        let bitmap = Bitmap{
            base:Default::default(),
            id,
            bitmap_data,
            width,
            height,
            smoothing,
            pixel_snapping:PixelSnapping::Auto

        };
        // bitmap.add_display_object(DisplayObjectWeak::Bitmap(bitmap.downgrade()));
        bitmap
    }
    pub fn new(id:CharacterId,bitmap:ruffle_render::bitmap::Bitmap)->Self{
        let width = bitmap.width();
        let height = bitmap.height();
        let transparency = match bitmap.format() {
            BitmapFormat::Rgba => true,
            BitmapFormat::Rgb => false,
            _ => unreachable!("Bitmap objects can only be constructed from RGB or RGBA source bitmaps"),
        };
        let pixels: Vec<_> = bitmap.as_colors()
        .map(Color::from).collect();
        let bitmap_data = BitmapDataWrapper::new_with_pixels(width, height, transparency, pixels);
        let smoothing = true;
        // Ok(Self::new_with_bitmap_data(id,bitmap_data, smoothing))
        let bitmap = Bitmap{
            base:Default::default(),
            id,
            bitmap_data,
            width,
            height,
            smoothing,
            pixel_snapping:PixelSnapping::Auto

        };
        bitmap
    }
    pub fn id(&self)->CharacterId{
        self.id
    }
    pub fn width(&self)->u32{
        self.width
    }
    pub fn height(&self)->u32{
        self.height
    }
    pub fn smoothing(&self)->bool{
        self.smoothing
    }
    pub fn pixel_snapping(&self)->PixelSnapping{
        self.pixel_snapping
    }
    pub fn bitmap_data(&self)->&BitmapDataWrapper{
        &self.bitmap_data
    }
    pub fn bitmap_data_mut(&mut self)->&mut BitmapDataWrapper{
        &mut self.bitmap_data
    }
    
}