# zach-tools

Windows 桌面效率工具:全局快捷键唤起的启动器面板,内置剪贴板历史等工具。技术栈 Tauri 2 + Vue 3,包管理器 bun。

## 开发

```bash
bun install          # 安装前端依赖(锁定 bun.lock)
bun run tauri dev    # 完整桌面运行时联调
bun run dev          # 仅浏览器预览前端(IPC 降级为 no-op)
```

提交前自查:`bun run format && bun run lint && bun run test && bun run build`,以及在 `src-tauri/` 下 `cargo fmt && cargo clippy && cargo test`。分层编码规范见 `.trellis/spec/`。

## 发布流程

一条命令完成「安全检查 → 同步版本号 → commit → tag → push」,后续打包与 GitHub Release 由 Actions 自动完成:

```bash
bun run release 0.2.0                # 指定版本号
bun run release patch                # 或按 patch / minor / major 递增(基于 package.json 当前版本)
bun run release 0.2.0-beta.1         # 带 `-` 的预发布版本,Release 会自动标记为 prerelease
```

脚本会依次:

1. 安全检查(任一不满足即中止并给出中文原因):工作区干净、当前在 `main`、与 `origin/main` 同步、tag `vX.Y.Z` 不存在
2. 把版本号写入 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json5`(保留注释),并刷新 `src-tauri/Cargo.lock`
3. `git commit -m "chore(release): vX.Y.Z"` → `git tag -a vX.Y.Z` → `git push --follow-tags origin main`

可选参数:

| 参数 | 作用 |
|---|---|
| `--dry-run` | 只执行只读的 git 检查并打印计划,不改文件、不 commit / tag / push |
| `--no-push` | 本地 commit + tag 后停下,自行确认后再 `git push --follow-tags origin main` |

tag 推送后 `.github/workflows/release.yml` 接手:先校验 tag 与三处版本号一致并跑前端 lint / test,再并行打包四个平台,全部成功后一次性创建带全部安装包的 GitHub Release(说明由 GitHub 按 commit 自动生成)。任一平台失败则不创建 Release,修复后删除 tag 重推即可。

本地随时可用 `bun run version:check [vX.Y.Z]` 校验三处版本号一致(以及与 tag 一致)。

### 平台支持

| 平台 | 安装包 | 功能 |
|---|---|---|
| Windows(x64) | `.msi`、`-setup.exe` | 完整功能:托盘、全局快捷键、剪贴板监听与粘贴 |
| macOS(Apple Silicon / Intel) | `.dmg` | 仅保证可安装可启动;平台能力(剪贴板监听、焦点还原、按键注入)为 stub 空实现 |
| Linux(x64) | `.deb`、`.rpm`、`.AppImage` | 同上,仅保证可安装可启动;基于 Ubuntu 24.04 构建,更老发行版可能因 glibc 版本无法运行 |

安装包均未签名:Windows 会出现 SmartScreen 提示,macOS 需在「系统设置 → 隐私与安全性」中允许打开。

`main` 分支的 push / PR 由 `.github/workflows/ci.yml` 跑前端(lint / typecheck / test / build)与 Rust(Windows + Ubuntu 的 fmt / clippy / test)门禁。
