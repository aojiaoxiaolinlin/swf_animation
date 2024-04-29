use std::default;
/// 我们目前处于框架的哪个阶段
/// 有五个阶段：进入、构建、帧脚本、退出和空闲。
/// 事件进程
#[derive(Debug, Clone, Copy, PartialEq, Eq, default::Default)]
pub enum FramePhase {
    /// 我们正在进入下一帧。
    /// 当影片剪辑进入新帧时，它们必须做两件事：
    /// - 删除所有不应存在于下一帧的子帧。
    /// - 增加其当前帧数。 
    /// 这一阶段结束后，我们将在广播列表中启动 enterFrame。
    Enter,
    /// 我们正在构建现有显示对象的子对象。
    /// 此时应执行所有 `PlaceObject` 标记。
    /// 构建帧后，我们会在广播列表中启动 `frameConstructed` 。
    Construct,
    /// 我们正在运行所有排队的帧脚本。
    /// 帧脚本相当于 AS3 旧式的 "DoAction "标记。如果当前时间线的帧号 /// 与前一帧的帧号不同，这些脚本将在 `Update` 阶段排队。
    FrameScripts,
    /// 我们正完成帧进程。
    /// 当我们退出一个已完成的帧时，我们会在广播列表中触发 `exitFrame` 。
    Exit,
     /// 我们目前没有执行任何帧代码。
     /// 此时，事件处理程序将运行。无帧
     /// catch-up work should execute.
    #[default]
    Idle,
}