mod animation;
pub mod render;
mod shape;

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    env,
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use swf::{CharacterId, DefineBitsLossless, SwfStr};

use crate::{
    animation::parse_animations,
    render::bitmap::{CompressedBitmap, decoder::decode_define_bits_jpeg_dimensions},
    shape::parse_shape_generate_img,
};

pub fn parse_swf(
    file_path: &str,
    scale: f32,
    special_scale: HashMap<CharacterId, f32>,
    output: Option<&str>,
) -> anyhow::Result<()> {
    let reader = BufReader::new(File::open(file_path)?);
    let swf_buf = swf::decompress_swf(reader)?;
    let swf: swf::Swf<'_> = swf::parse_swf(&swf_buf)?;
    let tags = swf.tags;

    let mut bitmaps = HashMap::new();
    for tag in &tags {
        match tag {
            swf::Tag::DefineBitsJpeg3(jpeg_data) => {
                let (width, height) = decode_define_bits_jpeg_dimensions(jpeg_data.data).unwrap();

                bitmaps.insert(
                    jpeg_data.id,
                    CompressedBitmap::Jpeg {
                        data: jpeg_data.data.to_vec(),
                        alpha: Some(jpeg_data.alpha_data.to_vec()),
                        width,
                        height,
                    },
                );
            }
            swf::Tag::DefineBitsLossless(bit_loss_less) => {
                bitmaps.insert(
                    bit_loss_less.id,
                    CompressedBitmap::Lossless(DefineBitsLossless {
                        version: bit_loss_less.version,
                        id: bit_loss_less.id,
                        format: bit_loss_less.format,
                        width: bit_loss_less.width,
                        height: bit_loss_less.height,
                        data: Cow::Owned(bit_loss_less.data.clone().into_owned()),
                    }),
                );
            }
            _ => {
                continue;
            }
        }
    }

    // 解析出文件名
    let path = Path::new(file_path);
    let file_name = if let Some(file_name) = path.file_name() {
        file_name.to_str().unwrap()
    } else {
        return Err(anyhow::anyhow!("无法获取文件名"));
    };

    let output = if let Some(output) = output {
        let path = Path::new(output);
        path.to_path_buf()
    } else {
        // 默认使用当前文件夹路径
        // 获取当前文件路径
        let current_dir: PathBuf = env::current_dir()?;
        let file_name: Vec<&str> = file_name.split(".").collect();
        current_dir.join("output").join(file_name.first().unwrap())
    };
    // 判断路径是否存在，如果不存在则创建
    if !output.exists() {
        std::fs::create_dir_all(&output)?;
    }
    let encoding_for_version = SwfStr::encoding_for_version(swf.header.version());

    let mut shape_offset = BTreeMap::new();

    parse_shape_generate_img(&tags, &mut shape_offset, scale, special_scale, &output)?;
    let flash_animation = parse_animations(
        tags,
        shape_offset,
        file_name,
        swf.header.frame_rate().to_f32() as u16,
        swf.header.num_frames(),
        encoding_for_version,
    )?;

    let file_name: Vec<&str> = file_name.split(".").collect();
    let writer = BufWriter::new(File::create(
        output.join(format!("{}.json", file_name.first().unwrap())),
    )?);
    serde_json::to_writer(writer, &flash_animation)?;

    Ok(())
}
