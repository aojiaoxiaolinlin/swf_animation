# To Make Better Use Of SWF Resources

## 目标

  该库将实现重构flash纯动画运行时

## swf_to_json

一个将`Adobe Animate` / `Flash`动画转成JSON数据的工具。

1. 用法:

- 自己编译：

    ```bash
          cargo run --release --package=swf_to_json  D:\Code\Rust\swf_animation\core\tests\swfs\spirit2159src.swf
          // 或者指定放大倍数
          cargo run --release --package=swf_to_json  D:\Code\Rust\swf_animation\core\tests\swfs\spirit2159src.swf --scale 2.0
    ```

- 编译完成
  
    ```bash
        swf_to_json "D:\Code\Rust\swf_animation\core\tests\swfs\spirit2159src.swf"
        // 或者指定放大倍数
        swf_to_json "D:\Code\Rust\swf_animation\core\tests\swfs\spirit2159src.swf" --scale 2.0
    ```

> 可用第三方工具，如`Texture Packer`等工具打包成`atlas`，然后使用`SpriteSheet`加载。

1. 动画文件 animation.json 的格式（懒的写示例了）：
2. 注意事项（规则）
    - 只有根影片剪辑的动画才会解析为`animations`项下的动画，其余的影片都作为子动画。若根影片无动画则`animations`下不会有可播放动画。
    - 使用多动画时要保证所有需要的动画都在根影片时间轴，使用影片中的`Label`分割，并且`Label`将作为动画的名称，若根影片无任何标签将会生成默认名（名称为`default`）。
    - 如果发现导入godot中素材堆叠在一起，可能是需要启用 `useRootTransform`。

3. 由于本人不熟悉`Godot`引擎，所有诚邀伙伴帮助完善`Godot`插件。
