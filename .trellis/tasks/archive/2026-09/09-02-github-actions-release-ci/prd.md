# GitHub Actions 自动发布与 main 门禁

## Goal

推 `v*` tag 后由 GitHub Actions 自动跨平台打包 Tauri 安装包并创建 GitHub Release;main 分支的 push / PR 自动跑前后端质量门禁。同时提供本地版本同步脚本,保证 tag 与代码内版本号一致。

## 背景

- 项目:Tauri 2 + Vue 3,包管理器 bun(`bun.lock`),主目标平台 Windows(托盘 / 全局快捷键 / 剪贴板监听),平台层通过 `#[cfg(windows)]` + `stub.rs` 保证其他平台可编译(见 `.trellis/spec/tauri/platform-windows.md`)
- 当前仓库无 `.github/`、无任何 tag
- 版本号目前散落三处且均为 `0.1.0`:`package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`(另 `Cargo.lock` 内也记录本 crate 版本)
- 本地已验证 `cargo fmt --check` / `cargo clippy` 干净,前端 lint / test 已在上一任务接入

## Requirements

### R1 发布工作流 `.github/workflows/release.yml`

- 触发:`push` 且 tag 匹配 `v*`
- 前置校验 job:
  - tag 版本(去掉 `v` 前缀)必须与 `package.json`、`Cargo.toml`、`tauri.conf.json` 三处版本号一致,不一致直接失败并给出中文提示
  - 跑前端 `bun run lint` + `bun run test`(不通过不发布)
- 构建矩阵(全部通过前置校验后并行):
  - `windows-latest` → `.msi` + NSIS `.exe`
  - `macos-latest`(aarch64) 与 `macos-latest --target x86_64-apple-darwin` → `.dmg`(两个架构分别产出)
  - `ubuntu-24.04` → `.deb` / `.rpm` / `.AppImage`(`ubuntu-22.04` 自 2026-09-17 起进入弃用流程,不再采用)
- 发布 job:所有平台构建成功后,统一创建一个 GitHub Release,上传全部安装包,Release 说明由 GitHub 自动生成(commit 列表);tag 含 `-`(如 `v0.2.0-beta.1`)时标记为 prerelease
- 任一平台构建失败 → 不创建 Release(保证 Release 完整,重跑即可)
- Rust 编译缓存(`swatinem/rust-cache`)以缩短构建时间

### R2 门禁工作流 `.github/workflows/ci.yml`

- 触发:`push` 到 `main`、`pull_request` 目标为 `main`
- 同一分支的新提交自动取消在跑的旧任务(`concurrency`)
- 前端 job(ubuntu):`bun install --frozen-lockfile` → `bun run lint` → `bun run test` → `bun run build`
- 后端 job(matrix:`windows-latest` + `ubuntu-24.04`,在 `src-tauri/` 下):`cargo fmt --check` → `cargo clippy --all-targets -- -D warnings` → `cargo test`
  - Windows 覆盖真实平台代码,Ubuntu 覆盖 stub 分支,防止改坏跨平台编译

### R3 一键发布脚本 `scripts/release.ts`(bun 运行)

- `bun run release <x.y.z | patch | minor | major>`:一条命令完成 **安全检查 → 同步版本号 → commit → tag → push**
  - 安全检查:工作区干净、当前在 `main`、tag `vX.Y.Z` 不存在、本地与远端同步;任一不满足则中止并给出中文原因
  - 同步写入 `package.json`、`Cargo.toml`、`tauri.conf.json`,并刷新 `Cargo.lock` 中本 crate 的版本(不触发编译)
  - `git commit -m "chore(release): vX.Y.Z"` → `git tag vX.Y.Z` → `git push --follow-tags`
  - `--dry-run`:只打印将执行的动作与结果版本,不改文件不跑 git;`--no-push`:本地 commit + tag 后停下,由用户自行 push
- `bun run version:check [vX.Y.Z]`:校验三处版本一致;传入 tag 时同时校验 tag 与版本号一致;release.yml 的 verify job 复用这个命令
- 版本号格式校验:semver(`x.y.z` 或带 `-` 预发布后缀);`patch/minor/major` 基于 `package.json` 当前版本递增
- 保留 `tauri.conf.json` 的 JSON5 注释(只做文本替换,不整文件重序列化)
- 只用 Node 内置模块(`fs` / `child_process`),不依赖 Bun 专有 API,不新增第三方依赖

### R3.1 脚本的工程化配置

- `scripts/**/*.ts` 纳入 `tsconfig.node.json` 的 include(补 `strict`、`types: ["node"]`),并保证它被类型检查覆盖——顺带覆盖此前未被类型检查的 `vite.config.ts`
  - 最终落地(`542e57f`):不另设 `typecheck:node` 脚本,而是把根 `tsconfig.json` 改为只含 `references` 的壳(`tsconfig.app.json` 浏览器侧 + `tsconfig.node.json` Node 侧),`build` 改为 `vue-tsc -b && vite build` 顺着 references 一次检查两侧;CI 前端 job 因此只需 lint → test → build
- `format` 脚本范围扩为 `src/ scripts/`;oxlint 现有配置已覆盖 `scripts/`,无需改动

### R4 文档

- `README` 说明发布流程(`bun run release <ver>` 一条命令 + 各平台支持程度);README 若不存在则新建

## 非目标(本轮不做)

- Tauri updater 自动更新(需签名密钥 + `latest.json`)
- Windows / macOS 代码签名与公证(用户安装时会看到 SmartScreen / Gatekeeper 提示,先接受)
- CSP 收紧(现有 spec 已记录为发布前债务,独立任务处理)
- `Cargo.toml` 的 `authors = ["you"]` / `description = "A Tauri App"` 元信息修正(顺手改亦可,不作为验收)

## Acceptance Criteria

- [x] `.github/workflows/release.yml` 存在,`actionlint`(或 GitHub 侧语法校验)无错误
- [ ] `.github/workflows/ci.yml` 存在(已完成),push 到 main 后前端 job 与后端 job(Windows + Ubuntu)全部绿(待线上验证)
- [x] `bun run release 0.1.1 --dry-run` 打印计划且不改任何文件;`bun run release 0.1.1 --no-push` 后三处版本号与 `Cargo.lock` 一致、`tauri.conf.json` 注释未丢失、产生 commit `chore(release): v0.1.1` 与 tag `v0.1.1`(在临时克隆 + 临时裸远端中验证,未触碰真实仓库)
- [x] 工作区脏 / 不在 main / tag 已存在时 `bun run release` 拒绝执行并输出中文原因
- [x] `bun run version:check v0.1.0` 通过;`bun run version:check v9.9.9` 失败并输出中文提示
- [x] `scripts/**` 与 `vite.config.ts` 被类型检查覆盖:`bun run build`(`vue-tsc -b`)通过,CI 前端 job 包含 build 步骤
- [ ] 推送 `v0.1.0`(或测试用 tag)后,Actions 产出 Windows msi/exe、macOS 两架构 dmg、Linux deb/rpm/AppImage,并自动创建带这些附件的 Release(待线上验证)
- [ ] tag 版本与代码版本不一致时,release 工作流在校验阶段失败,不进入构建(待线上验证;`version:check` 本地已验证退出码 1)
- [x] 所有 yml / 脚本注释为中文,commit 遵循 Conventional Commits

## Notes

- 技术方案与取舍见 `design.md`,执行步骤见 `implement.md`
