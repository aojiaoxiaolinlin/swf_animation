use std::{collections::HashMap, env, path::Path};

use clap::Parser;
use swf::CharacterId;
use swf_to_json::parse_swf;
use tracing_subscriber::{
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// 输入的swf文件路径
    #[arg(value_name = "FILE")]
    file_path: String,
    /// 图片放大倍数，默认为1
    #[arg(short, long, default_value = "1.0")]
    scale: f32,
    /// 特殊缩放比例配置文件路径，默认为当前目录下的Settings.toml
    /// 配置文件格式为：
    /// 1 = 1.0
    #[arg(long, value_name = "FILE")]
    settings_path: Option<String>,
    /// 输出的目录，默认为当前目录
    #[arg(short, long, value_name = "DIR")]
    output: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let file_path = &args.file_path;

    let env_filter = tracing_subscriber::EnvFilter::builder().parse_lossy(
        env::var("RUST_LOG")
            .as_deref()
            .unwrap_or("error,convert=info"),
    );
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .init();

    let settings_path = if let Some(settings_path) = &args.settings_path {
        Some(settings_path.as_str())
    } else if Path::new("Settings.toml").exists() {
        Some("Settings")
    } else {
        None
    };

    let special_scale = if let Some(settings_path) = settings_path {
        let config = config::Config::builder()
            .add_source(config::File::with_name(settings_path))
            .build()?;
        config
            .try_deserialize::<HashMap<CharacterId, f32>>()
            .map_err(|e| anyhow::anyhow!("无法解析Settings.toml: {}", e))?
    } else {
        HashMap::new()
    };

    parse_swf(file_path, args.scale, special_scale, args.output.as_deref())?;

    Ok(())
}
