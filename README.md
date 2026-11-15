# Seedmap Studio

[![build](https://github.com/TuBo0617/seedmap-studio/actions/workflows/build.yml/badge.svg)](https://github.com/TuBo0617/seedmap-studio/actions/workflows/build.yml)
[![license: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)

把 Axolotl Launcher 里的「种子地图」实验室工具拆出来单独做的一个桌面应用：
输入种子，按 Minecraft 版本和维度实时生成地形图，并标出村庄、要塞、远古城市等结构。

界面用 Vue 3 + TypeScript，桌面壳是 Tauri 2，地形由 Rust 调用 cubiomes（C）生成。

安装包见 [Releases](https://github.com/TuBo0617/seedmap-studio/releases)。

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

## 平台支持

与原 Axolotl 启动器一致，覆盖 6 个目标：

| 平台 | 目标三元组 | 产物 | C 编译器 |
|---|---|---|---|
| Windows x64 | `x86_64-pc-windows-msvc` | `.msi` / `-setup.exe` | `clang-cl`（回退 `cl.exe`） |
| Windows arm64 | `aarch64-pc-windows-msvc` | `-setup.exe` | `clang-cl` |
| Windows x64（MinGW） | `x86_64-pc-windows-gnu` | `.exe` | `x86_64-w64-mingw32-gcc` |
| macOS arm64 | `aarch64-apple-darwin` | `.dmg` | Apple clang |
| macOS x64 | `x86_64-apple-darwin` | `.dmg` | Apple clang |
| Linux x64 / arm64 | `*-unknown-linux-gnu` | `.deb` / `.rpm` / `.AppImage` | gcc / clang |

CI（`.github/workflows/build.yml`）按这张表跑矩阵，标签 push 时产出全部安装包。

已在 Linux 上一路验证到产物：`deb`、`rpm`、Windows x64 交叉编译的 `exe` 都可正常启动；
macOS 的 `dmg` 必须有 macOS 主机（Apple SDK 不能交叉），Windows arm64 同理，交给 CI。

**为什么 Windows 上默认用 clang-cl**：cubiomes 用了 C99 变长数组（VLA，6 个文件 15 处），
`cl.exe` 不支持；`clang-cl` 命令行与 MSVC 完全兼容，但接受这些 GNU 扩展。`build.rs`
会自动检测 `clang-cl`，找不到就退回 `cl.exe`（届时会因 VLA 编译失败）。不想装 LLVM 的话，
也可以用 MinGW 目标 `x86_64-pc-windows-gnu`，GCC 直接支持 VLA。

## 构建与运行

```bash
pnpm install
pnpm build                     # 前端 → dist/
pnpm tauri build               # 本机安装包（src-tauri/target/release/bundle/）
```

只想跑起来看效果：

```bash
pnpm build && cd src-tauri && cargo build && ./target/debug/seedmap-studio
```

### 各平台依赖

**Linux（Debian/Ubuntu）**

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev \
                 libayatana-appindicator3-dev librsvg2-dev patchelf
```

**Windows**：装 [WebView2 运行时](https://developer.microsoft.com/microsoft-edge/webview2/)（Win11 自带）
和 [LLVM](https://llvm.org/)（提供 `clang-cl`），或者直接用 MinGW 工具链。

**macOS**：装 Xcode Command Line Tools（`xcode-select --install`）。

### 交叉编译 Windows

Linux 上可以产出 Windows x64 可执行文件：

```bash
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
pnpm build
cd src-tauri && cargo build --release --target x86_64-pc-windows-gnu
```

产物在 `src-tauri/target/x86_64-pc-windows-gnu/release/seedmap-studio.exe`，
运行需要同目录下有 WebView2 运行时（系统级安装即可，`downloadBootstrapper` 安装包会自动装）。


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
