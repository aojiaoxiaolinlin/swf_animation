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

   ```json
      {
         // 动画名称
         "name": "head",
         // 动画帧率
         "frame_rate": 30,
         // 子动画，Key为子动画ID(对一个swf文件，子动画ID是唯一的),Value为子动画数据
         "base_animations": {
            "2": {
               // 表示一个时间轴，Key为时间轴深度，深度按照从小到大渲染，即深度值越小的在底层，深度值越大的在顶层，深度值大的重叠时会遮住小的。
               "time_lines": {
               // key为深度值，frames为帧数据
                  "2": {
                     "frames": [
                        {
                           // 资源ID，子动画或者图形资源中的一个
                           "id": 1,
                           // 第几帧才开始渲染
                           "place_frame": 0,
                           // 帧持续时间，这里持续一帧
                           "duration": 1,
                           // 矩阵变换，用于缩放、旋转、平移等操作。
                           "matrix": {
                              "a": 5.4772186, // 缩放X
                              "b": 0.0, // 旋转倾斜0（RotateSkew0）
                              "c": 0.0, // 旋转倾斜1（RotateSkew1）
                              "d": 5.4772186, // 缩放Y
                              "tx": -203.5, // 平移X
                              "ty": -223.0 // 平移Y
                           },
                           // 颜色变换，用于改变颜色。
                           "color_transform": {
                              "mult_color": [
                                 0.0,
                                 0.0,
                                 0.0,
                                 0.0
                              ],
                              "add_color": [
                                 0.0,
                                 0.0,
                                 0.0,
                                 0.0
                              ]
                           },
                           // 混合模式，用于控制两个图像之间的混合方式。默认为Normal。TODO: 混合模式 改为数值判断？还是枚举？
                           "blend_mode": "Normal"
                           // 还有滤镜，暂未添加
                        },
                     ]
                  }
               }
            }
         },
         // 目标动画，key为动画名称，value为动画数据，这里的名称默认是`default`，因为没有在adobe animation中设置动画标签，所以默认为`default`。当设置了动画标签后，这里会根据标签名称进行分类，一个标签代表一个动画。
           "animations": {
               "default": {
                  "time_lines": {
                  "1": {
                     "frames": [
                        {
                        "id": 2,
                        "place_frame": 0,
                        "duration": 1,
                        "matrix": {
                           "a": 1.0,
                           "b": 0.0,
                           "c": 0.0,
                           "d": 1.0,
                           "tx": 608.2,
                           "ty": 534.15
                        },
                        "color_transform": {
                           "mult_color": [
                              0.0,
                              0.0,
                              0.0,
                              0.0
                           ],
                           "add_color": [
                              0.0,
                              0.0,
                              0.0,
                              0.0
                           ]
                        },
                        "blend_mode": "Normal"
                        }
                     ]
                  }
               }
            }
         }
      }
   ```
