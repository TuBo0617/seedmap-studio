# Seedmap Studio

把 Axolotl Launcher 里的「种子地图」实验室工具拆出来单独做的一个桌面应用：
输入种子，按 Minecraft 版本和维度实时生成地形图，并标出村庄、要塞、远古城市等结构。

界面用 Vue 3 + TypeScript，桌面壳是 Tauri 2，地形由 Rust 调用 cubiomes（C）生成。

## 目录结构

```
.
├── crates/seedmap-core/        # 地形引擎（Rust + cubiomes）
│   ├── src/
│   │   ├── lib.rs              # 对外 API：render_tile / find_features / spawn / biome_at
│   │   ├── bridge.rs           # 与 C 的 FFI
│   │   ├── cubiomes_bridge.c   # 渲染、结构查找、出生点
│   │   ├── cubiomes_details.c  # 结构细节
│   │   ├── ores.rs             # 矿石候选
│   │   ├── details.rs          # 战利品 / 结构详情
│   │   ├── underground.rs      # 地下体积
│   │   └── assets/versions/    # 各版本结构布局数据（gzip JSON）
│   ├── build.rs                # 用 cc 编译 cubiomes + 桥接层
│   └── vendor/cubiomes/        # cubiomes 源码（xpple 分支，commit a49d5bc）
├── src-tauri/                  # 桌面外壳与命令注册
│   ├── src/lib.rs              # seedmap_* 命令 → seedmap-core
│   └── tauri.conf.json
└── src/                        # 前端
    ├── App.vue                 # 地图、缩放、拖拽、结构标记、图例
    ├── lib/api.ts              # invoke 封装与类型
    └── lib/features.ts         # 结构名称与配色
```

## 构建与运行

```bash
pnpm install
pnpm build                     # 产出 dist/

cd src-tauri
cargo build                    # 首次会编译 cubiomes 与 Tauri 依赖
./target/debug/seedmap-studio
```

需要系统里有 GTK3 / WebKitGTK 4.1（Linux）。Ubuntu 上：

```bash
sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0 libayatana-appindicator3-1
```

## 已实现

- 种子输入、版本切换（26.3 向下到 1.16.5，由引擎版本表决定）、维度切换（主世界 / 下界 / 末地）
- 拖拽平移、滚轮以光标为锚点缩放、缩放按钮、回到出生点
- 两级渲染：先出 1/4 分辨率预览，再补全分辨率
- 视口内结构查询与标记（26 种），左下角图例按类型统计
- 状态栏显示中心坐标、光标坐标、比例尺与渲染耗时

## 坐标约定

`render_tile` 的 `x` / `z` 是瓦片左上角的**方块坐标**，`scale` 表示一个像素对应多少方块，
引擎内部用 `x.div_euclid(scale)` 换算成像素坐标。前端由视口中心和缩放反推左上角。

## 来源与许可

- 地形生成：[cubiomes](https://github.com/xpple/cubiomes)（MIT），vendored 在 `crates/seedmap-core/vendor/cubiomes`
- 引擎封装、结构表与前端逻辑取自 [Axolotl Launcher](https://github.com/Axolotl-Launcher/Axolotl)（GPL-3.0）
- 本项目整体按 GPL-3.0 发布
