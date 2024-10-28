use std::{fs::File, io::BufReader};

use swf::{Encoding, SwfStr, Tag};
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();
    for i in 1..2759 {
        let path = format!("D:/kabu/kabuxiyou/assets/battle/battleRole/spirit{i}src.swf");
        info!("加载文件：{path}");
        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => {
                info!("文件不存在，跳过");
                continue;
            }
        };
        let reader = BufReader::new(file);
        let swf_buf = swf::decompress_swf(reader).unwrap();
        let swf = swf::parse_swf(&swf_buf).unwrap();
        println!("The SWF has {} frame(s).", swf.header.num_frames());
        println!("The SWF has {} tag(s).", swf.tags.len());
        let mut new_tags = Vec::new();
        info!("解析完成，转换");
        let mut total_frames = swf.header.swf_header().num_frames;
        let encoding_for_version = SwfStr::encoding_for_version(swf.header.swf_header().version);
        cover_new_swf(
            &mut new_tags,
            swf.tags,
            &mut total_frames,
            "_mc",
            encoding_for_version,
        );
        let new_header = swf::Header {
            num_frames: total_frames,
            ..swf.header.swf_header().clone()
        };

        let file = std::fs::File::create(format!(
            "D:/kabu/kabuxiyou/assets/battle/new_battleRole/spirit{i}src.swf"
        ))
        .unwrap();
        let writer = std::io::BufWriter::new(file);
        swf::write_swf(&new_header, &new_tags, writer).unwrap();
        info!("写入完成");
    }
}

fn cover_new_swf<'a>(
    new_tags: &mut Vec<swf::Tag<'a>>,
    tags: Vec<Tag<'a>>,
    total_frames: &mut u16,
    target: &str,
    encoding_for_version: &'static Encoding,
) {
    for tag in tags {
        let res = match tag {
            swf::Tag::ShowFrame => swf::Tag::ShowFrame,
            swf::Tag::DefineBitsJpeg2 { id, jpeg_data } => {
                swf::Tag::DefineBitsJpeg2 { id, jpeg_data }
            }
            swf::Tag::DefineBitsJpeg3(define_bits_jpeg3) => {
                swf::Tag::DefineBitsJpeg3(define_bits_jpeg3)
            }
            swf::Tag::DefineShape(shape) => swf::Tag::DefineShape(shape),
            swf::Tag::DefineSprite(sprite) => swf::Tag::DefineSprite(sprite),
            swf::Tag::Metadata(metadata) => swf::Tag::Metadata(metadata),
            swf::Tag::PlaceObject(place_object) => match place_object.action {
                swf::PlaceObjectAction::Place(id) => {
                    if let Some(name) = place_object.name {
                        if target == name.to_str_lossy(&encoding_for_version) {
                            // 删除new_tags中id相同的ShowFrame标签
                            new_tags.retain(|tag| {
                                if let swf::Tag::ShowFrame = tag {
                                    return false;
                                }
                                true
                            });
                            // 从new_tags中找到id相同的DefineSprite，并删除
                            let target = new_tags
                                .iter_mut()
                                .position(|tag| {
                                    if let swf::Tag::DefineSprite(sprite) = tag {
                                        return id == sprite.id;
                                    }
                                    false
                                })
                                .map(|index| new_tags.remove(index));
                            if let Some(target) = target {
                                if let swf::Tag::DefineSprite(mut sprite) = target {
                                    // cover_target_new_swf(new_tags, sprite.tags, total_frames);
                                    new_tags.append(&mut sprite.tags);
                                    return; // 返回，不再处理后续的PlaceObject标签
                                }
                            }
                        }
                    }
                    swf::Tag::PlaceObject(place_object)
                }
                _ => swf::Tag::PlaceObject(place_object),
            },
            swf::Tag::FileAttributes(file_attributes) => swf::Tag::FileAttributes(file_attributes),
            swf::Tag::Unknown {
                tag_code: _,
                data: _,
            } => {
                continue;
            }
            _ => {
                continue;
            }
        };

        new_tags.push(res);
    }
}

fn cover_target_new_swf<'a>(
    new_tags: &mut Vec<swf::Tag<'a>>,
    tags: Vec<Tag<'a>>,
    total_frames: &mut u16,
) {
    let mut sub_frames = 0;
    for tag in tags {
        let res = match tag {
            swf::Tag::ShowFrame => {
                if sub_frames > 0 {
                    sub_frames -= 1;
                    continue;
                }
                swf::Tag::ShowFrame
            }
            swf::Tag::DefineBitsJpeg2 { id, jpeg_data } => {
                swf::Tag::DefineBitsJpeg2 { id, jpeg_data }
            }
            swf::Tag::DefineBitsJpeg3(define_bits_jpeg3) => {
                swf::Tag::DefineBitsJpeg3(define_bits_jpeg3)
            }
            swf::Tag::DefineShape(shape) => swf::Tag::DefineShape(shape),
            swf::Tag::DefineSprite(sprite) => {
                dbg!(sprite.id);
                swf::Tag::DefineSprite(sprite)
            }
            swf::Tag::Metadata(metadata) => swf::Tag::Metadata(metadata),
            swf::Tag::SetBackgroundColor(color) => swf::Tag::SetBackgroundColor(color),
            swf::Tag::PlaceObject(place_object) => match place_object.action {
                swf::PlaceObjectAction::Place(id) => {
                    let mut frame_delta = 0;
                    new_tags.iter().for_each(|tag| {
                        if let swf::Tag::DefineSprite(sprite) = tag {
                            if sprite.id == id {
                                if sprite.num_frames > 10 {
                                    frame_delta += sprite.num_frames - 10;
                                    *total_frames += frame_delta;
                                } else {
                                    // 减少帧数
                                    sub_frames = 10 - sprite.num_frames;
                                    *total_frames -= sub_frames;
                                    dbg!(sub_frames);
                                }
                            }
                        }
                    });
                    new_tags.push(swf::Tag::PlaceObject(place_object));
                    for _ in 0..frame_delta {
                        new_tags.push(swf::Tag::ShowFrame);
                    }
                    continue;
                }
                _ => swf::Tag::PlaceObject(place_object),
            },
            swf::Tag::RemoveObject(remove_object) => swf::Tag::RemoveObject(remove_object),
            swf::Tag::FileAttributes(file_attributes) => swf::Tag::FileAttributes(file_attributes),
            swf::Tag::FrameLabel(frame_label) => swf::Tag::FrameLabel(frame_label),
            swf::Tag::Unknown {
                tag_code: _,
                data: _,
            } => {
                continue;
            }
            _ => {
                continue;
            }
        };
        new_tags.push(res);
    }
}
