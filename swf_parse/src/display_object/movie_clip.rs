type FrameNumber = u16;
#[derive(Debug)]
pub struct MovieClip {
    pub num_frames: u16,
}

impl MovieClip {
    pub fn new(num_frames:FrameNumber) -> Self {
        Self {
            num_frames,
        }
    }
}
