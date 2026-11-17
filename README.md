# Seedmap Studio

[![build](https://github.com/TuBo0617/seedmap-studio/actions/workflows/build.yml/badge.svg)](https://github.com/TuBo0617/seedmap-studio/actions/workflows/build.yml)
[![license: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)

把 Axolotl Launcher 里的「种子地图」实验室工具拆出来单独做的一个应用：
输入种子，按 Minecraft 版本和维度实时生成地形图，并标出村庄、要塞、远古城市等结构。
桌面（Windows / macOS / Linux）与移动端（Android / iOS）共用同一套 RusT + cubiomes 引擎。

界面用 Vue 3 + TypeScript，壳层是 Tauri 2，地形由 Rust 调用 cubiomes（C）生成。

安装包见 [Releases](https://github.com/TuBo0617/seedmap-studio/releases)。

## 功能

从 Axolotl 的种子地图实验室完整移植过来的能力：

| 模块 | 说明 |
|---|---|
| 地形图 | 1:1 / 1:4 / 1:16 / 1:64 / 1:256 五档缩放，主世界、下界、末地三个维度，按版本生成地表着色 |
| 结构标记 | 26 类结构，标记使用原版启动器的方块图标（80×80 WebP，随缩放收缩、过密时降级为小点），图例同步显示图标 |
| 结构详情 | 分段列表 + 俯视投影图（有布局数据的结构），区块坐标、朝向 |
| 容器战利品 | 逐容器解析战利品表，显示物品、数量区间与概率 |
| 生物群系 | 按群系高亮地图、单点探针查询、视野内群系抽样统计（条形图 + 百分比）；选择器带群系缩略图 |
| 矿石分布 | 钻石 / 铁 / 金 / 红石 / 青金石 / 绿宝石 / 铜 / 煤 / 下界残骸，分批扫描区块候选点；面板用方块图标，地图点用 16×16 平铺纹理（含深板岩变体） |
| 地下剖面 | 指定 Y 高度的水平切片（岩石 / 洞穴 / 水 / 岩浆），体积渲染 |
| 区域分析 | 视野内各类结构的数量与最近距离 |
| 历史与工作区 | 最近种子记录、拖入 `level.dat` 或存档文件夹导入种子、退出前保存视图状态 |
| 移动端手势 | 单指平移（合成层跟手，跨边距才补图）、双指捏合（吸附到五档）、双击放大、长按群系探针 |

## 目录结构

```
.
├── crates/seedmap-core/        # 地形引擎（Rust + cubiomes）
│   ├── src/
│   │   ├── lib.rs              # 对外 API：render_tile / find_features / spawn / biome_at / level.dat
│   │   ├── bridge.rs           # 与 C 的 FFI
│   │   ├── cubiomes_bridge.c   # 渲染、结构查找（按类型分配配额）、出生点
│   │   ├── cubiomes_details.c  # 结构细节
│   │   ├── ores.rs             # 矿石候选
│   │   ├── details.rs          # 战利品 / 结构详情 / 地下切片
│   │   ├── underground.rs      # 地下体积
│   │   └── assets/versions/    # 各版本结构布局数据（gzip JSON）
│   ├── build.rs                # 用 cc 编译 cubiomes + 桥接层
│   └── vendor/cubiomes/        # cubiomes 源码（xpple 分支，commit a49d5bc）
├── src-tauri/                  # 桌面外壳与命令注册
│   ├── src/lib.rs              # seedmap_* 命令 → seedmap-core
│   ├── src/seedmap_ext.rs      # 矿石 / 存档 / 结构详情 / 地下切片命令
│   ├── gen/android/            # Tauri 生成的 Android Gradle 工程（四 ABI）
│   └── tauri.conf.json
├── public/seed-map-assets/     # 原版启动器素材（2.8 MB）
│   ├── structures/             # 29 张 80×80 结构方块图标
│   ├── biomes/                 # 70 张群系缩略图
│   └── ores/                   # 方块图标 + 16×16 平铺纹理（含深板岩变体）
└── src/                        # 前端
    ├── App.vue                 # 地图、瓦片缓存、手势、结构标记、图例
    ├── components/             # 群系 / 矿石 / 结构 / 分析 / 剖面 / 历史面板
    ├── lib/api.ts              # invoke 封装与类型
    ├── lib/features.ts         # 结构名称与配色
    └── lib/icons.ts            # 图标路径映射、预加载与缓存
```

## 平台支持

桌面端与原 Axolotl 启动器一致，覆盖 6 个目标；移动端覆盖 4 个 Android ABI 和 iOS：

| 平台 | 目标三元组 | 产物 | C 编译器 |
|---|---|---|---|
| Windows x64 | `x86_64-pc-windows-msvc` | `.msi` / `-setup.exe` | `clang-cl`（回退 `cl.exe`） |
| Windows arm64 | `aarch64-pc-windows-msvc` | `-setup.exe` | `clang-cl` |
| Windows x64（MinGW） | `x86_64-pc-windows-gnu` | `.exe` | `x86_64-w64-mingw32-gcc` |
| macOS arm64 | `aarch64-apple-darwin` | `.dmg` | Apple clang |
| macOS x64 | `x86_64-apple-darwin` | `.dmg` | Apple clang |
| Linux x64 / arm64 | `*-unknown-linux-gnu` | `.deb` / `.rpm` / `.AppImage` | gcc / clang |
| Android arm64 | `aarch64-linux-android` | `.apk`（arm64-v8a） | NDK clang 21+ |
| Android arm | `armv7-linux-androideabi` | `.apk`（armeabi-v7a） | NDK clang 21+ |
| Android x86 | `i686-linux-android` | `.apk`（x86） | NDK clang 21+ |
| Android x86_64 | `x86_64-linux-android` | `.apk`（x86_64） | NDK clang 21+ |
| iOS arm64 | `aarch64-apple-ios` | `.app` / 未签名 `.ipa` | Xcode clang |
| iOS 模拟器 | `aarch64-apple-ios-sim`、`x86_64-apple-ios` | 模拟器 `.app` | Xcode clang |

Android 默认产出**一个通用 APK，同时包含 4 个 ABI**，不需要为不同设备分包；`minSdk` 为 24（Android 7.0）。
iOS 的部署目标写成 **13.0**，这是 Tauri 2 / WKWebView 的下限（`UIWindowScene` 从 iOS 13 才有），
再往下没有可用的运行时支持。CI 用 Xcode 26.5 的 SDK 构建，产物的 `Info.plist` 里
`MinimumOSVersion` 实测就是 **13.0**（`DTSDKName: iphoneos26.5`），没有被抬到 15。
不过 iOS 13/14 真机没有实测过：声明的最低版本只代表二进制允许安装的系统下限，
实际能不能跑还取决于系统框架的可用 API。

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

### 移动端

两端共用同一份前端与引擎，区别只在壳工程和 C 工具链。`build.rs` 会按目标自动挑选编译器：
Android 用 NDK 里的 `<triple><api>-clang`，iOS 用 Xcode 的 clang 并带上
`-mios-version-min=13.0`。

**Android**（Linux / macOS / Windows 均可构建）

```bash
# 依赖：JDK 17+、Android SDK、NDK r27、四个 Rust target
export ANDROID_HOME=$HOME/Android/Sdk
export NDK_HOME=$ANDROID_HOME/ndk/27.2.12479018
rustup target add aarch64-linux-android armv7-linux-androideabi \
                  i686-linux-android x86_64-linux-android

pnpm install
pnpm build
pnpm tauri android init --ci       # 只需一次，生成 src-tauri/gen/android
pnpm tauri android build --apk     # 通用 APK，一次打包四个 ABI
```

产物：`src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk`。

只想要某个架构（体积小一半以上）时加 `--target` 或直接分 ABI 打包：

```bash
pnpm tauri android build --apk --target armv7   # aarch64 / armv7 / i686 / x86_64
pnpm tauri android build --apk --split-per-abi  # 四个独立 APK
```

签名：仓库里带了一个**开发用自签名证书**（`app/dev-keystore.jks`，密码 `seedmapstudio`），
所以默认就能产出可安装的 APK，方便侧载测试。正式发布请换成自己的 key —— 在
`src-tauri/gen/android/keystore.properties` 写入 `storeFile` / `storePassword` /
`keyAlias` / `keyPassword`，Gradle 会优先使用它；这个文件已在 `.gitignore` 里。

> 通用包同时含四个 ABI（约 37 MB）；分 ABI 后单个约 12–14 MB。Android 7.0（API 24）以下
> 不支持，这是 Tauri 2 的下限。

**iOS**（必须有 macOS + Xcode）

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
pnpm install && pnpm build
brew install xcodegen          # `tauri ios init` 会调用它
pnpm tauri ios init --ci
pnpm tauri ios build --target aarch64        # 真机；模拟器用 aarch64-sim / x86_64
```

命令里的 `--target` 是 Tauri 自己的名字（`aarch64` / `aarch64-sim` / `x86_64`），
不是 Rust 三元组；`ios build` 默认就是 release，没有 `--release` 这个参数。

未签名产物用 `--no-sign` 生成，产物是可直接重签的 `.ipa`：

```bash
pnpm tauri ios build --target aarch64 --no-sign
# src-tauri/gen/apple/build/arm64/Seedmap Studio.ipa
```

再用 AltStore、Sideloadly 或 `codesign -f -s "Apple Development: ..."` 重签后安装。
真机调试需要在 `tauri.conf.json` 的 `bundle.iOS.developmentTeam` 填上 Team ID
（或设 `APPLE_DEVELOPMENT_TEAM` 环境变量），也可以直接用 Xcode 打开
`src-tauri/gen/apple/seedmap-studio.xcodeproj` 选个人开发者账号。

> Apple SDK 不能交叉编译，Linux / Windows 上无法产出 iOS 包；这也是 CI 把 iOS 单独
> 放在 `macos-26` 上的原因。

**只跑 iOS 的 CI**：Actions → `build` → Run workflow，把 `jobs` 选成 `ios`。
CI 没有 Apple 证书，所以走 `--no-sign`，`APPLE_DEVELOPMENT_TEAM` 用占位值只为
通过 Xcode 工程的团队校验。跑完在 run 页面下载 `ios-unsigned` 工件，里面是
`Seedmap-Studio-<版本>-ios-unsigned.ipa`。`jobs` 也可以选 `android` / `desktop` / `all`。

### 触屏操作

| 手势 | 行为 |
|---|---|
| 单指拖动 | 平移地图 |
| 双指捏合 | 连续缩放，松手吸附到 1:1 / 1:4 / 1:16 / 1:64 / 1:256 五档 |
| 双击 | 以点击处为中心放大一档 |
| 长按 | 探针：显示该点坐标与生物群系 |

移动端布局与桌面不同：顶部只留种子输入和「生成」，版本 / 维度 / 图例收进底部浮动
Dock 与上滑面板，缩放置于右侧竖排，全部避开刘海和手势条的安全区。


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
