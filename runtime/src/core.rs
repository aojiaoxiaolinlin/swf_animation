use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
    fmt::Debug,
};

use anyhow::Result;
use error::RuntimeError;
use glam::Mat4;
use swf::CharacterId;

use crate::parser::{
    Animation, DepthTimeline, FiltersTransform, KeyFrame, Matrix, MovieClip, Transform,
    types::{
        BevelFilter, BlendMode, BlurFilter, ColorMatrixFilter, DropShadowFilter, Filter,
        GlowFilter, GradientFilter,
    },
};

use filter::Filter as RenderFilter;

mod error;
mod filter;
mod state_machine;

type CompletionCallback = Box<dyn FnOnce() + Send + Sync + 'static>;
type FrameEventCallback = Box<dyn Fn() + Send + Sync + 'static>;

#[derive(Default)]
pub struct AnimationPlayer {
    // ---------资源-----------
    /// 原flash动画帧率
    frame_rate: f32,
    /// 动画数据
    animations: HashMap<String, Animation>,
    /// 动画子影片资源
    children_clip: HashMap<CharacterId, MovieClip>,
    /// 运行时实例，扁平化结构
    active_instances: Vec<RuntimeInstance>,

    // ----------控制-----------
    /// 播放速度
    speed: f32,
    /// 是否循环
    looping: bool,
    /// 当前时间
    current_time: f32,
    /// 是否播放
    playing: bool,
    /// 当前动画名
    current_animation_name: Option<String>,
    /// 皮肤
    current_skins: HashMap<String, String>,
    /// 播放完成时回调
    on_completion: Option<CompletionCallback>,
    /// 用于帧事件
    frame_event_listeners: HashMap<String, Vec<FrameEventCallback>>,
}

impl AnimationPlayer {
    /// 默认播放速度 x 1.0
    pub fn new(
        animations: HashMap<String, Animation>,
        children_clip: HashMap<CharacterId, MovieClip>,
        frame_rate: f32,
    ) -> Self {
        Self {
            animations,
            children_clip,
            speed: 1.0,
            frame_rate,
            playing: true,
            ..Default::default()
        }
    }

    pub fn update(&mut self, delta_time: f32) -> Result<()> {
        if !self.playing || self.current_animation_name.is_none() {
            return Ok(());
        }

        let animation_name = self.current_animation_name.as_ref().unwrap().clone();

        // 1. Time Advancement & Looping
        let previous_time = self.current_time;
        let elapsed_time = delta_time * self.speed;
        self.current_time += elapsed_time;

        let animation = self
            .animations
            .get(&animation_name)
            .ok_or(RuntimeError::AnimationNotFound(animation_name))?;
        let duration = animation.duration;
        let mut on_completion = None;

        let mut looped = false;
        if self.current_time >= duration {
            if self.looping {
                self.current_time = self.current_time % duration;
                looped = true;
            } else {
                self.current_time = duration;
                self.playing = false;

                // TODO: 触发动画完成事件
                on_completion = self.on_completion.take();
            }
        }

        let children_clip = &mut self.children_clip;
        let current_skins = &mut self.current_skins;

        // 2.Instance Lifecycle & Property Updates (Iterate through Depths)
        let mut active_instances = Vec::new();
        let base_transform = Mat4::IDENTITY;
        collect_current_time_active_shape(
            &animation.timeline,
            previous_time,
            elapsed_time,
            children_clip,
            current_skins,
            self.frame_rate,
            &mut active_instances,
            base_transform,
            BlendMode::Normal,
            Vec::new(),
        )?;

        self.active_instances = active_instances;

        // 3.Frame Event Handle
        // 处理时间值精度问题
        let cmp_current_time = (self.current_time * 1.0e6).trunc();
        let cmp_previous_time = (previous_time * 1.0e6).trunc();

        for event_keyframe in animation.events.iter() {
            let time = (event_keyframe.time * 1.0e6).trunc();
            if (time >= cmp_previous_time) && time < cmp_current_time {
                if let Some(frame_events) = self.frame_event_listeners.get(&event_keyframe.name) {
                    frame_events.iter().for_each(|event| event());
                }
            }
        }

        // 触发完成事件
        if let Some(on_completion) = on_completion {
            on_completion();
        }
        Ok(())
    }

    pub fn set_play_animation(
        &mut self,
        name: &str,
        looping: bool,
        on_completion: Option<CompletionCallback>,
    ) -> Result<()> {
        if !self.animations.contains_key(name) {
            return Err(RuntimeError::AnimationNotFound(name.to_owned()).into());
        }

        // 重置时间
        self.current_time = 0.0;
        // 清除活动实例
        self.active_instances.clear();

        self.current_animation_name = Some(name.to_owned());
        self.looping = looping;
        self.on_completion = on_completion;
        Ok(())
    }

    pub fn set_skip(&mut self, part_name: &str, skip_name: &str) -> Result<()> {
        let skip_name = skip_name.to_owned();
        if let Some(target_skip) = self
            .get_skips()
            .iter()
            .find(|part_skips| part_skips.contains_key(part_name))
        {
            if target_skip.get(part_name).unwrap().contains(&&skip_name) {
                self.current_skins.insert(part_name.to_owned(), skip_name);
            } else {
                return Err(RuntimeError::SkinNotFound(skip_name).into());
            }
        } else {
            return Err(RuntimeError::SkinPartNotFound(part_name.to_owned()).into());
        }
        Ok(())
    }

    pub fn get_skips(&self) -> Vec<HashMap<&str, Vec<&String>>> {
        self.children_clip
            .values()
            .filter(|clip| clip.is_skin_frame())
            .map(|clip| {
                let mut skip = HashMap::new();
                skip.insert(
                    clip.name().expect("没有就是有Bug"),
                    clip.skin_frames().keys().collect::<Vec<_>>(),
                );
                skip
            })
            .collect::<Vec<_>>()
    }

    pub fn current_skins(&self) -> &HashMap<String, String> {
        &self.current_skins
    }

    /// 注册一个监听特定名称帧事件的回调函数。
    ///
    /// # Arguments
    /// * `animation_name` - 要监听的动画名。
    /// * `event_name` - 要监听的事件名称 (例如 "footstep", "hit_impact")。
    /// * `callback` - 当事件触发时要调用的函数。
    pub fn register_frame_event<T>(
        &mut self,
        animation_name: &str,
        event_name: String,
        callback: T,
    ) -> Result<()>
    where
        T: Fn() + Send + Sync + 'static,
    {
        // 判断监听的事件是否存在
        if let Some(animation) = self.animations.get(animation_name) {
            if animation
                .events
                .iter()
                .any(|event| event.name == event_name)
            {
                self.frame_event_listeners
                    .entry(event_name)
                    .or_default()
                    .push(Box::new(callback));
            } else {
                return Err(RuntimeError::AnimationEventNotFound(event_name).into());
            }
        } else {
            return Err(RuntimeError::AnimationNotFound(animation_name.to_owned()).into());
        }

        Ok(())
    }

    /// 移除指定事件名称的所有监听器。
    pub fn clear_frame_event_listeners(&mut self, event_name: &str) {
        self.frame_event_listeners.remove(event_name);
    }
    /// 移除所有帧事件监听器。
    pub fn clear_all_frame_event_listeners(&mut self) {
        self.frame_event_listeners.clear();
    }

    pub fn current_animation_name(&self) -> Option<&str> {
        self.current_animation_name.as_deref()
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = if speed <= 0.0 { 0.0 } else { speed };
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    pub fn looping(&self) -> bool {
        self.looping
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.playing = playing;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }
}

fn collect_current_time_active_shape(
    timeline: &BTreeMap<u16, DepthTimeline>,
    current_time: f32,
    elapsed_time: f32,
    children_clip: &mut HashMap<CharacterId, MovieClip>,
    current_skins: &mut HashMap<String, String>,
    frame_rate: f32,
    active_instances: &mut Vec<RuntimeInstance>,
    base_transform: Mat4,
    blend_mode: BlendMode,
    filters: Vec<RenderFilter>,
) -> Result<()> {
    for (_, depth_timeline) in timeline {
        let placements = &depth_timeline.placement;
        let (Some(start), _) = find_key_frame(current_time, placements) else {
            continue;
        };

        let start_keyframe = depth_timeline.placement.get(start).unwrap();
        if let Some(id) = start_keyframe.resource_id() {
            let transforms = &depth_timeline.transforms;
            // 既然start存在那么transform一定存在
            let (transform_start_index, transform_end_index) =
                find_key_frame(current_time, transforms);
            let transform = transforms_lerp(
                transform_start_index.expect("transform 必须存在、否则这会是一个Bug"),
                transform_end_index,
                transforms,
                current_time,
            );

            let current_transform = base_transform.mul_mat4(&transform);

            if let Some(mut child_clip) = children_clip.remove(&id) {
                // 混合模式只能添加在影片上
                let blend_transforms = &depth_timeline.blend_transform;
                let blend_mode =
                    if let (Some(start), _) = find_key_frame(current_time, blend_transforms) {
                        blend_transforms
                            .get(start)
                            .expect("获取混合模式有Bug")
                            .blend_mode
                    } else {
                        BlendMode::Normal
                    };

                // 滤镜也只能添加到影片上
                let filter_transforms = &depth_timeline.filters_transforms;
                let (start, end) = find_key_frame(current_time, filter_transforms);
                let mut filters = Vec::new();
                lerp_filter(start, end, filter_transforms, current_time, &mut filters);

                // 判断是否是皮肤clip
                let child_current_time = if child_clip.is_skin_frame() {
                    // 是皮肤clip
                    let name = child_clip.name().expect("替换皮肤的影片剪辑必须命名！");
                    // 是否设置了皮肤
                    let skip_frame = if let Some(skip_name) = current_skins.get(name) {
                        if let Some(frame) = child_clip.skin_frame(skip_name) {
                            *frame
                        } else {
                            return Err(RuntimeError::SkinNotFound(skip_name.to_owned()).into());
                        }
                    } else {
                        // 没有设置皮肤，使用默认皮肤
                        child_clip.default_skip_frame()
                    };
                    // 计算对应帧对应的事件
                    skip_frame as f32 / frame_rate
                } else {
                    child_clip.current_time
                };

                collect_current_time_active_shape(
                    child_clip.timeline(),
                    child_current_time,
                    elapsed_time,
                    children_clip,
                    current_skins,
                    frame_rate,
                    active_instances,
                    current_transform,
                    blend_mode,
                    filters,
                )?;
                child_clip.current_time += elapsed_time;
                if child_clip.current_time >= child_clip.duration() {
                    child_clip.current_time = child_clip.current_time % child_clip.duration();
                }
                children_clip.insert(id, child_clip);
            } else {
                // 记录这个child_movie找到的shape为当前活动实例，将每一帧的实例Shape扁平化输出，游戏引擎中迭代实在不方便
                active_instances.push(RuntimeInstance::new(
                    id,
                    current_transform,
                    blend_mode,
                    filters.clone(),
                ));
            }
        }
    }
    Ok(())
}

fn find_key_frame<T: KeyFrame>(time: f32, key_frames: &Vec<T>) -> (Option<usize>, Option<usize>) {
    match key_frames.binary_search_by(|k| k.time().partial_cmp(&time).unwrap_or(Ordering::Less)) {
        // 刚好相等
        Ok(index) => (Some(index), None),
        Err(index) => {
            if index == 0 {
                // 时间在第一个关键帧之前，当前深度的keyframe不显示
                (None, None)
            } else if index >= key_frames.len() {
                // 时间在最后一个关键帧之后，使用最后一帧值
                (Some(key_frames.len() - 1), None)
            } else {
                // 时间在 index-1 到 index 之间
                (Some(index - 1), Some(index))
            }
        }
    }
}

/// 实例只需要存储用于引擎渲染的Shape就行吗？
/// 在多个Shape合成的MovieClip上应用滤镜，需要一起渲染，
#[derive(Debug, Default)]
pub struct RuntimeInstance {
    resource_id: CharacterId,

    current_skin: Option<String>,

    is_dirty: bool,

    transform: Mat4,
    blend: BlendMode,
    filters: Vec<RenderFilter>,
}

impl RuntimeInstance {
    fn new(id: CharacterId, transform: Mat4, blend: BlendMode, filters: Vec<RenderFilter>) -> Self {
        Self {
            resource_id: id,
            transform,
            blend,
            filters,
            ..Default::default()
        }
    }

    fn update_instance_property(
        &mut self,
        current_time: f32,
        depth_timeline: &crate::parser::DepthTimeline,
    ) {
        let transforms = &depth_timeline.transforms;
        let (Some(start_index), end_index) = find_key_frame(current_time, transforms) else {
            return;
        };
        // 更新transform
        let transform = transforms_lerp(start_index, end_index, transforms, current_time);
    }
}

fn transforms_lerp(
    start_index: usize,
    end_index: Option<usize>,
    transform: &Vec<Transform>,
    current_time: f32,
) -> Mat4 {
    let start = transform.get(start_index).unwrap();

    let matrix = if let Some(end_index) = end_index {
        let end = transform.get(end_index).unwrap();
        &lerp_matrix(
            &start.matrix,
            &end.matrix,
            calc_lerp_factor(start.time, end.time, current_time),
        )
    } else {
        &start.matrix
    };
    matrix.into()
}

/// 对两个 Matrix 进行线性插值
///
/// - `start`: 起始矩阵
/// - `end`: 结束矩阵
/// - `t`: 插值因子 (0.0 to 1.0)
///
/// 返回插值后的新矩阵
fn lerp_matrix(start: &Matrix, end: &Matrix, t: f32) -> Matrix {
    // 确保 t 在 [0, 1] 范围内
    let t = t.clamp(0.0, 1.0);

    Matrix {
        a: start.a + (end.a - start.a) * t,
        b: start.b + (end.b - start.b) * t,
        c: start.c + (end.c - start.c) * t,
        d: start.d + (end.d - start.d) * t,
        tx: start.tx + (end.tx - start.tx) * t,
        ty: start.ty + (end.ty - start.ty) * t,
    }
}

fn lerp_filter<'a>(
    start: Option<usize>,
    end: Option<usize>,
    filter_transforms: &[FiltersTransform],
    current_time: f32,
    res_filters: &'a mut Vec<RenderFilter>,
) {
    if let Some(start) = start {
        let start_filters = &filter_transforms[start].filters;
        if let Some(end) = end {
            let end_filters = &filter_transforms[end].filters;

            let t = calc_lerp_factor(
                filter_transforms[start].time,
                filter_transforms[end].time,
                current_time,
            );

            if start_filters.len() == end_filters.len() {
                for (start_filter, end_filter) in start_filters.iter().zip(end_filters) {
                    match (start_filter, end_filter) {
                        (Filter::BlurFilter(start_filter), Filter::BlurFilter(end_filter)) => {
                            res_filters.push(RenderFilter::from(&Filter::BlurFilter(BlurFilter {
                                blur_x: start_filter.blur_x
                                    + (end_filter.blur_x - start_filter.blur_x) * t,
                                blur_y: start_filter.blur_y
                                    + (end_filter.blur_y - start_filter.blur_y) * t,
                                flags: start_filter.flags,
                            })));
                        }
                        (Filter::GlowFilter(start_filter), Filter::GlowFilter(end_filter)) => {
                            res_filters.push(RenderFilter::from(&Filter::GlowFilter(GlowFilter {
                                color: start_filter.color,
                                blur_x: start_filter.blur_x
                                    + (end_filter.blur_x - start_filter.blur_x) * t,
                                blur_y: start_filter.blur_y
                                    + (end_filter.blur_y - start_filter.blur_y) * t,
                                strength: start_filter.strength
                                    + (end_filter.strength - start_filter.strength) * t,
                                flags: start_filter.flags,
                            })));
                        }
                        (Filter::BevelFilter(start_filter), Filter::BevelFilter(end_filter)) => {
                            res_filters.push(RenderFilter::from(&Filter::BevelFilter(
                                BevelFilter {
                                    shadow_color: start_filter.shadow_color,
                                    highlight_color: start_filter.shadow_color,
                                    blur_x: start_filter.blur_x
                                        + (end_filter.blur_x - start_filter.blur_x) * t,
                                    blur_y: start_filter.blur_y
                                        + (end_filter.blur_y - start_filter.blur_y) * t,
                                    angle: start_filter.angle
                                        + (end_filter.angle - start_filter.angle) * t,
                                    distance: start_filter.distance
                                        + (end_filter.distance - start_filter.distance) * t,
                                    strength: start_filter.strength
                                        + (end_filter.strength - start_filter.strength) * t,
                                    flags: start_filter.flags,
                                },
                            )));
                        }
                        (
                            Filter::ColorMatrixFilter(start_filter),
                            Filter::ColorMatrixFilter(end_filter),
                        ) => {
                            res_filters.push(RenderFilter::from(&Filter::ColorMatrixFilter(
                                ColorMatrixFilter {
                                    matrix: start_filter
                                        .matrix
                                        .iter()
                                        .zip(end_filter.matrix)
                                        .map(|(start, end)| start + (end - start) * t)
                                        .collect::<Vec<_>>()
                                        .try_into()
                                        .unwrap(),
                                },
                            )));
                        }
                        (
                            Filter::DropShadowFilter(start_filter),
                            Filter::DropShadowFilter(end_filter),
                        ) => {
                            res_filters.push(RenderFilter::from(&Filter::DropShadowFilter(
                                DropShadowFilter {
                                    color: start_filter.color,
                                    blur_x: start_filter.blur_x
                                        + (end_filter.blur_x - start_filter.blur_x) * t,
                                    blur_y: start_filter.blur_y
                                        + (end_filter.blur_y - start_filter.blur_y) * t,
                                    angle: start_filter.angle
                                        + (end_filter.angle - start_filter.angle) * t,
                                    distance: start_filter.distance
                                        + (end_filter.distance - start_filter.distance) * t,
                                    strength: start_filter.strength
                                        + (end_filter.strength - start_filter.strength) * t,
                                    flags: start_filter.flags,
                                },
                            )));
                        }
                        (
                            Filter::GradientBevelFilter(start_filter),
                            Filter::GradientBevelFilter(end_filter),
                        ) => {
                            res_filters.push(RenderFilter::from(&Filter::GradientBevelFilter(
                                GradientFilter {
                                    colors: start_filter.colors.clone(),
                                    blur_x: start_filter.blur_x
                                        + (end_filter.blur_x - start_filter.blur_x) * t,
                                    blur_y: start_filter.blur_y
                                        + (end_filter.blur_y - start_filter.blur_y) * t,
                                    angle: start_filter.angle
                                        + (end_filter.angle - start_filter.angle) * t,
                                    distance: start_filter.distance
                                        + (end_filter.distance - start_filter.distance) * t,
                                    strength: start_filter.strength
                                        + (end_filter.strength - start_filter.strength) * t,
                                    flags: start_filter.flags,
                                },
                            )));
                        }
                        (
                            Filter::GradientGlowFilter(start_filter),
                            Filter::GradientGlowFilter(end_filter),
                        ) => {
                            res_filters.push(RenderFilter::from(&Filter::GradientGlowFilter(
                                GradientFilter {
                                    colors: start_filter.colors.clone(),
                                    blur_x: start_filter.blur_x
                                        + (end_filter.blur_x - start_filter.blur_x) * t,
                                    blur_y: start_filter.blur_y
                                        + (end_filter.blur_y - start_filter.blur_y) * t,
                                    angle: start_filter.angle
                                        + (end_filter.angle - start_filter.angle) * t,
                                    distance: start_filter.distance
                                        + (end_filter.distance - start_filter.distance) * t,
                                    strength: start_filter.strength
                                        + (end_filter.strength - start_filter.strength) * t,
                                    flags: start_filter.flags,
                                },
                            )));
                        }
                        _ => {
                            res_filters.append(
                                &mut start_filters
                                    .iter()
                                    .map(|f| RenderFilter::from(f))
                                    .filter(|f| !f.impotent())
                                    .collect::<Vec<_>>(),
                            );
                            return;
                        }
                    }
                }
            }
        }
        res_filters.append(
            &mut start_filters
                .iter()
                .map(|f| RenderFilter::from(f))
                .filter(|f| !f.impotent())
                .collect::<Vec<_>>(),
        );
    }
}

fn calc_lerp_factor(start_time: f32, end_time: f32, current_time: f32) -> f32 {
    let segment_duration = end_time - start_time;
    let raw_t = if segment_duration <= 0.0 {
        0.0
    } else {
        (current_time - start_time) / segment_duration
    };

    raw_t.clamp(0.0, 1.0)
}
