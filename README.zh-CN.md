# Herdr — 非官方 compact-herds 分支

[English](README.md) · [简体中文](README.zh-CN.md) · [本分支发行版](https://github.com/JSRRosenbaum/herdr/releases/tag/compact-herds-v0.9.5-20261018) · [原版 Herdr](https://github.com/herdrdev/herdr)

这是 [Herdr](https://github.com/herdrdev/herdr)（运行编程智能体的终端环境）的 **JSRRosenbaum 非官方定制分支**，不是 Herdr 官方发行版，也不是侧栏插件。[公开截图](https://pbs.twimg.com/media/HUw9wCvXcAIHsIz.png?name=orig)（[原帖](https://x.com/JezzickRose/status/2111131042123792478)）展示的是本分支内置侧栏、特定配置和实际运行的工作区。Herdr 提供底层智能体终端、状态、机器与工作树功能；本分支增加了编号 herd 分页及各页的焦点与导航状态、紧凑工作树布局和可选工作区 ID。截图中的智能体、项目、机器连接和个人配置并不随发行版提供。

## 获取这个定制版本

从**本仓库的非官方** [`compact-herds-v0.9.5-20261018` 发行版](https://github.com/JSRRosenbaum/herdr/releases/tag/compact-herds-v0.9.5-20261018)下载，并核对源码提交及 SHA-256 校验值。按系统与架构选择文件（Linux x86_64/aarch64、macOS x86_64/aarch64、Windows x86_64），直接运行该版本的 `herdr`，不要覆盖原版安装。Windows 请解压**完整的 `herdr-windows-x86_64.zip`**，将附带的 `conpty/` 目录与 `herdr.exe` 保持在一起；不能只复制可执行文件。

**不要通过** `herdr.dev/install.sh`、`install.ps1`、`brew install herdr`、`mise use -g herdr`、原版 GitHub 发行页或 `herdr update` 获取或更新此分支：它们安装/更新的是**原版 Herdr**。此分支没有单独的自动更新通道。为避免与默认会话的原版混用，体验时使用隔离的命名会话：

```bash
/path/to/this-fork/herdr --session compact-herds-demo
```

Windows 下，在解压后的目录用 PowerShell 运行 `./herdr.exe --session compact-herds-demo`。命名会话不会连接日常默认会话；请保持 Windows 解压文件完整。不要用此分支更新正在运行的原版服务器。

### 从标签编译本分支

安装 [Rust 1.96.1](https://www.rust-lang.org/tools/install)（含 Cargo）、[Zig 0.16.0](https://ziglang.org/download/)、CMake、Ninja、Git 以及本机 C/C++ 编译工具链，并确保 `zig`、`cmake`、`ninja` 在 `PATH` 上。Linux 需要发行版提供的 C/C++ 构建工具；若要编译发行版使用的 musl 目标，还需 musl 工具（交叉编译 aarch64 时还需对应交叉编译器）。macOS 需要 Xcode Command Line Tools。Windows 本机编译需要 Visual Studio C++ Build Tools 和 Windows SDK；如果 `PATH` 上是其他 Zig 版本，请将 `ZIG` 指向 Zig 0.16.0 可执行文件。Cargo 单独编译出的 Windows exe **不是**完整的 ConPTY 压缩包；需要打包运行组件时请使用发行版 ZIP。

```bash
git clone https://github.com/JSRRosenbaum/herdr.git herdr-compact-herds
cd herdr-compact-herds
git fetch origin tag compact-herds-v0.9.5-20261018
git checkout --detach compact-herds-v0.9.5-20261018
cargo +1.96.1 --config 'env.HERDR_BUILD_CHANNEL="local"' --config 'env.HERDR_BUILD_ID="compact-herds-20261018"' build --release --locked
# Linux/macOS 可执行文件：target/release/herdr
```

以上命令固定到本分支的标签，不会误用 `master` 或原版仓库。需要与发行包相同的目标平台构建时，请参阅仓库的[手动构建工作流](.github/workflows/build-artifacts-manual.yml)，其中有准确的 Rust 目标、工具链和 Windows ConPTY 打包流程。运行自行编译的二进制文件时，请写明路径并加 `--session compact-herds-demo`，避免使用 `PATH` 上的其他 `herdr`。

## 侧栏配置

在 Linux/macOS 的 `~/.config/herdr/config.toml` 或 Windows 的 `%APPDATA%\herdr\config.toml` 加入以下配置（若已有 `[ui.sidebar.spaces]`，请合并，不要重复创建该节）：

```toml
[ui.sidebar.spaces]
worktree_layout = "compact"
show_workspace_ids = true
worktree_group_separator = "none"
```

`worktree_layout = "compact"` 使用本分支较浅的工作树连接符及默认单行空间布局；`show_workspace_ids = true` 在空间行右侧显示稳定的 `w…` 工作区标识。`worktree_group_separator` 可为 `"none"`（默认）、`"collapsed"` 或 `"always"`，控制工作树组后的空行；显式设置 `rows` 会覆盖紧凑模式的默认行布局。工作树分组、保存的 SSH 机器和智能体状态基于 Herdr 原有功能。本分支的编号 herd 分页将机器分组并保留各页的焦点与导航状态，默认用 `prefix+,` 和 `prefix+.` 切换（先按 Ctrl+B，再按逗号/句号）。这些是内置客户端功能，不是 OMP 插件。实际内容取决于自己的机器、工作树和智能体。[原版配置](https://herdr.dev/zh-cn/docs/configuration/)与[远程机器](https://herdr.dev/zh-cn/docs/connecting-machines/)文档介绍通用模型，不一定涵盖此分支的 herd、紧凑布局与 ID 选项。

Tern 的原生/ANSI 呈现集成属于**独立的实验工作**，不是此侧栏的必要条件；当前源码快照也不包含 `terminal.presentation.get` 策略。

## 原版文档与致谢

原项目：[herdrdev/herdr](https://github.com/herdrdev/herdr)、[原版快速开始](https://herdr.dev/zh-cn/docs/quick-start/)、[原版文档](https://herdr.dev/zh-cn/docs/)、[原版插件](https://herdr.dev/plugins/)。这些资料介绍原版 Herdr；其中的安装器和下载文件不提供本分支。[SPONSORS.md](SPONSORS.md) 记载原项目的支持者。本分支采用 [Apache 2.0](LICENSE) 许可证。协助修改本仓库的智能体请阅读 [AGENTS.md](AGENTS.md) 和 [CONTRIBUTING.md](CONTRIBUTING.md)。
