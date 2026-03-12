# OpenClaw Console (macOS App)

OpenClaw 的本地桌面控制台，基于 `Vue 3 + Tauri 2`，仅提供 macOS App 形态。

## 主要功能
- 本机状态面板：检测 OpenClaw CLI、Gateway 状态、会话文件状态
- Gateway 端口选择：从本机进程发现端口并切换目标 Gateway
- 会话面板：
  - 今天/过往分组
  - Session 搜索
  - 对话视图 + 调试视图（JSON）
  - 向选中 session 发送消息
  - 会话模型切换（`provider/model`）
- Agent 面板：展示 agent 运行状态
- Skill 面板：展示可用 skills
- 日志面板：展示 Gateway logs，支持过滤与导出
- 本机配置编辑：读取并修改 `~/.openclaw/openclaw.json`

## 环境要求
- macOS（Apple Silicon）
- Node.js 18+
- Rust toolchain（`rustc` / `cargo`）
- 本机已安装 OpenClaw CLI（默认会探测 `/opt/homebrew/bin/openclaw`）

## 开发运行
```bash
npm install
npm run dev
```

## 构建与打包
```bash
npm run build
```

打包输出：
- `.app`: `src-tauri/target/release/bundle/macos/OpenClaw Console.app`
- `.dmg`: `src-tauri/target/release/bundle/dmg/OpenClaw Console_0.1.0_aarch64.dmg`

## 项目结构
- `src/`: Vue 前端
- `src-tauri/`: Tauri/Rust 后端与打包配置
- `src-tauri/icons/`: 应用图标资源

## 说明
- 本项目不再支持浏览器模式。
- 如遇 macOS 图标缓存问题，可重新安装后执行：
  - `killall Finder && killall Dock`
