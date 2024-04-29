use std::time::Instant;

use super::DisplayObjectBase;
use bitflags::bitflags;

bitflags! {
    /// Boolean state flags used by `InteractiveObject`.
    #[derive(Clone, Copy, Debug)]
    struct InteractiveObjectFlags: u8 {
        /// Whether this `InteractiveObject` accepts mouse and other user
        /// events.
        const MOUSE_ENABLED = 1 << 0;

        /// Whether this `InteractiveObject` accepts double-clicks.
        const DOUBLE_CLICK_ENABLED = 1 << 1;
    }
}

#[derive(Debug, Clone)]
pub struct InteractiveObjectBase{
    pub base:DisplayObjectBase,
    flags:InteractiveObjectFlags,
    last_click: Option<Instant>,

    /// 对象对焦时是否显示黄色发光边框。
    focus_rect: Option<bool>,
}

impl Default for InteractiveObjectBase {
    fn default() -> Self {
        Self {
            base: Default::default(),
            flags: InteractiveObjectFlags::MOUSE_ENABLED,
            last_click: None,
            focus_rect: None,
        }
    }
}