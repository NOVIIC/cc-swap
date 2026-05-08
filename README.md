# cc-swap

Claude Code 的 `settings.json` 秒切工具。原生 Rust + Slint，无 Web 套壳；用完即走，不驻后台。

## 是什么

如果你需要在多份 `settings.json` 之间反复切换（多账号 / 多 token / 多环境 / 多模型供应商），手工复制或写 PowerShell 脚本都不够轻巧。本工具的设计目标：

- **秒开**：软渲染、零 GPU 初始化、单文件 ~4 MB。
- **极简**：双击 → 一个按钮 → 切完自动关窗。
- **不动态解析配置**：纯字节拷贝，不会破坏 JSON 里的注释或格式。
- **替换前自动备份**：原文件落到同目录的 `settings.json.bak`。

## 使用

### 1. 安装

到 [Releases](../../releases) 页下载对应平台的二进制：

- **Windows**: `cc-swap.exe`
- **Linux**: `cc-swap`

放到你喜欢的目录，例如 `~/tools/cc-swap/`。

或者自己构建（需要 Rust 1.85+，edition 2024）：

```bash
cargo build --release
# 产物：target/release/cc-swap (Linux) 或 cc-swap.exe (Windows)
```

Linux 构建前需安装系统依赖：

```bash
# Debian / Ubuntu
sudo apt install libx11-dev libxkbcommon-dev libfontconfig1-dev
```

### 2. 首次运行

双击运行，自动使用默认路径 `~/.claude/settings.json`（即 `%USERPROFILE%\.claude\settings.json` 或 `$HOME/.claude/settings.json`）作为目标配置，并写入程序同级的 `cc-swap.conf`，下次启动直接读取。

### 3. 准备配置文件

在程序同级的 `settings/` 文件夹里放任意多份配置（程序首次运行时会自动建好这个文件夹），文件名随意：

```
cc-swap/
├── cc-swap(.exe)
├── cc-swap.conf
└── settings/
    ├── work.json
    ├── personal.json
    └── glm-coding-plan.json
```

### 4. 日常使用

双击运行 → 看到每份配置对应一个按钮 → 点击 → 状态栏显示「已切换到 xxx」→ 窗口约 500 ms 后自动关闭。

每次切换前，旧的目标 `settings.json` 会被复制为 `settings.json.bak`（与目标同目录，覆盖式保留最近一次，方便误操作时恢复）。

### 5. 修改目标路径

需要换目标时，直接编辑程序同级的 `cc-swap.conf`，把里面那一行路径改成新的目标即可。

## 工作原理

```
启动
 ├─ 读取 cc-swap.conf
 │   ├─ 不存在 → 自动写入默认路径 (~/.claude/settings.json)
 │   └─ 存在   → 用里面的路径
 ├─ ensure settings/ 文件夹存在
 ├─ 扫描 settings/ 下所有文件（按文件名升序）
 ├─ 渲染 Slint 窗口（每个文件一个按钮）
 └─ 点击按钮
     ├─ 拷贝目标 settings.json → settings.json.bak（如目标存在）
     ├─ 拷贝选中文件 → 目标 settings.json
     ├─ 显示「已切换到 xxx」
     └─ 500 ms 后退出事件循环 → 进程结束
```

## 项目结构

```
cc-swap/
├── Cargo.toml          # 依赖与 release profile
├── build.rs            # slint_build::compile("ui/app.slint")
├── ui/
│   └── app.slint       # UI 定义
└── src/
    └── main.rs         # 全部 Rust 逻辑
```

## 冷启动调优

| 项 | 做法 |
|---|---|
| 渲染后端 | Slint `renderer-software`，跳过 OpenGL 初始化 |
| 二进制大小 | `opt-level = "z"` + `lto = true` + `codegen-units = 1` + `strip = true` |
| 异常处理 | `panic = "abort"`，去 unwind 表 |
| 控制台 | `cfg_attr(windows, windows_subsystem = "windows")`，避免 Windows 上 console flash |
| 配置序列化 | 不引 serde，conf 是纯文本路径 |

## 边界情况

| 情况 | 处理 |
|---|---|
| `settings/` 不存在 | 自动创建 |
| `settings/` 为空 | UI 显示提示「请把配置文件放进 settings/ 文件夹」 |
| 目标 `settings.json` 不存在（首次） | 跳过备份，直接写入 |
| 复制失败（权限/被占用） | 状态栏显示错误，窗口不关，可重试 |
| `settings/` 含子目录 | 跳过，仅列文件 |

## 平台

支持 Windows 11 与 Linux（X11 / Wayland）。
