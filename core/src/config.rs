use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "letterbox")]
pub enum Letterbox {
    /// The content will never be letterboxed.
    #[serde(rename = "off")]
    Off,

    /// The content will only be letterboxed if the content is running fullscreen.
    #[serde(rename = "fullscreen")]
    Fullscreen,

    /// The content will always be letterboxed.
    #[serde(rename = "on")]
    On,
}