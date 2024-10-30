# SWF Animation Runtime

## To make better use of SWF resources

## SWF动画运行数据（JSON）

1. 用法:

   ```bash
      cargo run --release --package=convert  D:\Code\Rust\swf_animation\core\tests\swfs\spirit2159src.swf
      // 或者指定放大倍数
      cargo run --release --package=convert  D:\Code\Rust\swf_animation\core\tests\swfs\spirit2159src.swf --scale 2.0
   ```

   >可用第三方工具，如`Texture Packer`等工具打包成`atlas`，然后使用`SpriteSheet`加载。

2. 动画文件animation.json的格式如下:
