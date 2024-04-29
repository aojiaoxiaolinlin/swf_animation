use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "letterbox")]
/// 当播放器的宽高比与电影的宽高比不一致时，控制内容是信箱还是方柱。
/// 信箱化时，内容的外部边缘将呈现黑条。
pub enum Letterbox {
    /// 内容永远不会采用`letterboxed`.
    #[serde(rename = "off")]
    Off,

    /// 只有在全屏运行时，才会对内容进行 `letterboxed` 处理
    #[serde(rename = "full_screen")]
    FullScreen,

    /// 总是对内容进行 `letterboxed` 处理
    #[serde(rename = "on")]
    On,
}