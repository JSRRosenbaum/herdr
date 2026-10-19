# Herdr — unofficial compact-herds fork

[English](README.md) · [简体中文](README.zh-CN.md) · [Fork release](https://github.com/JSRRosenbaum/herdr/releases/tag/compact-herds-v0.9.5-20261018-r1) · [Original Herdr](https://github.com/herdrdev/herdr)

This is **JSRRosenbaum's unofficial custom fork** of [Herdr](https://github.com/herdrdev/herdr), the terminal runtime for coding agents. It is not an upstream Herdr release or a sidebar plugin. The [shared screenshot](https://pbs.twimg.com/media/HUw9wCvXcAIHsIz.png?name=orig) ([post](https://x.com/JezzickRose/status/2111131042123792478)) shows the fork's built-in sidebar with a particular configuration and live workspaces. Herdr provides the underlying agent terminals, status, machines and worktrees; this fork adds numbered herd pages with per-herd focus/navigation, a compact worktree presentation and optional visible workspace IDs. It does not ship the screenshot's agents, projects, machine connections or personal configuration.

## Get this exact fork

Get the **unofficial** [`compact-herds-v0.9.5-20261018-r1` release](https://github.com/JSRRosenbaum/herdr/releases/tag/compact-herds-v0.9.5-20261018-r1) from this repository and check its source commit and SHA-256 checksums before using an asset. Choose the artifact for your OS/architecture (Linux x86_64 or aarch64, macOS x86_64 or aarch64, Windows x86_64). Run its `herdr` binary directly and keep it separate from an upstream installation. On Windows, extract the **complete `herdr-windows-x86_64.zip`**, keeping the bundled `conpty/` directory alongside `herdr.exe`; do not copy out the executable alone.

**Do not use** `herdr.dev/install.sh`, `install.ps1`, `brew install herdr`, `mise use -g herdr`, upstream GitHub releases, or `herdr update` to obtain or keep this fork: those install/update **upstream Herdr**, not this custom build. This fork has no separate automatic update channel. Installing both against the same default session can mix versions; launch the fork in an isolated named session while evaluating it:

```bash
/path/to/this-fork/herdr --session compact-herds-demo
```

On Windows, from the extracted ZIP in PowerShell, use `./herdr.exe --session compact-herds-demo`. The named session prevents attaching to your usual default session; keep the extracted Windows files together. Do not use the fork to update an existing upstream server.

### Build the tagged fork from source

Install [Rust 1.96.1](https://www.rust-lang.org/tools/install) (including Cargo), [Zig 0.16.0](https://ziglang.org/download/), CMake, Ninja, Git, and a native C/C++ toolchain; ensure `zig`, `cmake`, and `ninja` are on `PATH`. On Linux, install your distribution's C/C++ build tools; for the release's musl targets also install musl tools (and an aarch64 cross compiler if cross-building). macOS needs Xcode Command Line Tools. Native Windows builds need Visual Studio C++ Build Tools and Windows SDK; set `ZIG` to the Zig 0.16.0 executable if a different Zig is on `PATH`. The Windows executable built by Cargo alone is **not** the complete packaged ConPTY ZIP; use the release ZIP for the packaged Windows runtime.

```bash
git clone https://github.com/JSRRosenbaum/herdr.git herdr-compact-herds
cd herdr-compact-herds
git fetch origin tag compact-herds-v0.9.5-20261018-r1
git checkout --detach compact-herds-v0.9.5-20261018-r1
cargo +1.96.1 --config 'env.HERDR_BUILD_CHANNEL="local"' --config 'env.HERDR_BUILD_ID="compact-herds-20261018-r1"' build --release --locked
# Linux/macOS executable: target/release/herdr
```

These commands deliberately pin this fork's tag rather than `master` or the upstream repository. For release-equivalent target builds, the repository's [manual artifact workflow](.github/workflows/build-artifacts-manual.yml) documents the exact Rust targets, toolchain and Windows ConPTY packaging procedure. Run a locally built binary by its explicit path with `--session compact-herds-demo`, not a different `herdr` on your `PATH`.

## Sidebar setup

Put the following in `~/.config/herdr/config.toml` on Linux/macOS or `%APPDATA%\herdr\config.toml` on Windows (merge with any existing `[ui.sidebar.spaces]` section):

```toml
[ui.sidebar.spaces]
worktree_layout = "compact"
show_workspace_ids = true
worktree_group_separator = "none"
```

`worktree_layout = "compact"` selects the fork's shallow worktree connectors and single-row default space layout; `show_workspace_ids = true` right-aligns stable `w…` workspace tags on space rows. `worktree_group_separator` accepts `"none"` (default), `"collapsed"` or `"always"`; it controls separator spacing after worktree groups. Explicit `rows` settings override the compact default row layout. Worktree grouping, saved SSH machines and agent status build on Herdr's existing functionality. This fork's numbered herd pages group machines and retain their focus/navigation context; switch with `prefix+,` and `prefix+.` (by default Ctrl+B then comma/period). These are built-in client features, not an OMP plugin. Your own machines, worktrees and agents determine what appears. The [upstream configuration](https://herdr.dev/docs/configuration/) and [remote machines](https://herdr.dev/docs/connecting-machines/) guides cover the general Herdr model, not this fork's extra herd/compact/ID settings.

Tern's native/ANSI presentation integration is **separate experimental work**, not required for this sidebar and not included as a `terminal.presentation.get` policy in this source snapshot.

## Upstream documentation and credit

Original project: [herdrdev/herdr](https://github.com/herdrdev/herdr), [upstream quick start](https://herdr.dev/docs/quick-start/), [upstream documentation](https://herdr.dev/docs/), [upstream plugins](https://herdr.dev/plugins/). These resources document upstream Herdr; their downloads and installers do not install this fork. The original project's sponsors are listed in [SPONSORS.md](SPONSORS.md). This fork is licensed under [Apache 2.0](LICENSE). Agents editing this repository should read [AGENTS.md](AGENTS.md) and [CONTRIBUTING.md](CONTRIBUTING.md).
