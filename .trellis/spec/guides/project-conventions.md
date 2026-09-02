# 项目通用约定

> 跨层生效的项目级约定:语言、Git 提交、注释。分层编码规范见 [frontend](../frontend/index.md) 与 [tauri](../tauri/index.md)。

---

## 语言

- 项目主语言为中文:与用户交流、代码注释、文档、commit 描述一律用中文
- 代码标识符(变量 / 函数 / 类型名)仍用英文命名,不用拼音

---

## Git 提交规范(Conventional Commits)

- 完整结构:标题行 + 空行 + 正文(可选)+ 空行 + footer(可选),一次提交只做一件事
- 标题行 `type(scope): 中文描述`:简洁祈使句说明做了什么,结尾不加句号
  - type 取值:`feat` / `fix` / `refactor` / `perf` / `docs` / `style` / `test` / `build` / `chore`
  - scope 可选,用模块名(如 `launcher`、`ui`、`tauri`)
- 正文:解释改动的动机、方案取舍和影响范围(回答「为什么」,不复述代码);简单自明的改动可省略,复杂改动必须写
- footer:破坏性变更写 `BREAKING CHANGE: 说明`(或在 type 后加 `!`),关联问题写 `Closes #123`

```
fix(launcher): 失焦自动隐藏窗口改为仅在非置顶模式下生效

置顶模式下用户期望窗口常驻,此前失焦一律隐藏导致置顶开关形同虚设。
现在隐藏逻辑读取置顶状态,仅普通模式下失焦隐藏。

Closes #12
```

```
feat(launcher): 支持 alt+enter 全局快捷键唤起
```

---

## 结构体 / 类型注释

- 跨端传输、配置、领域模型等关键结构体必须写文档注释:结构体本身说明用途,每个字段说明含义
- Rust 用 `///` 文档注释,TypeScript 用 `/** */`,跨端结构体两侧字段与注释一一对应
- 函数内部一眼能看懂的临时结构可以不写,但公开 API 和跨端结构必须写
- 具体写法与真实示例:后端见 [tauri/commands-and-ipc.md](../tauri/commands-and-ipc.md) 的「跨端结构体」,前端见 [frontend/type-safety.md](../frontend/type-safety.md) 的「跨端(IPC)类型镜像」

---

## 版本号与发布

**版本号有三处源文件,必须同值**:`package.json`、`src-tauri/Cargo.toml`(`[package]` 段)、`src-tauri/tauri.conf.json`;`src-tauri/Cargo.lock` 里本 crate 的条目随 `Cargo.toml` 刷新。**不要手改任何一处**,一律走脚本:

```bash
bun run release <x.y.z | patch | minor | major> [--dry-run] [--no-push]   # scripts/release.ts
bun run version:check [vX.Y.Z]                                             # scripts/version.ts,CI verify job 复用
```

- `release` 流程:安全检查 → 写三处 + `cargo update --workspace --offline` 刷 lock → `git commit -m "chore(release): vX.Y.Z"`(只 add 这四个文件)→ 附注 tag `vX.Y.Z` → `git push --follow-tags origin main`;之后 `.github/workflows/release.yml` 接手打包与 Release
- 安全检查全部通过才动文件,任一失败中文报错、退出码 1:工作区脏 / 不在 `main` / 本地与 `origin/main` 不同步 / tag 已存在(本地或远端)
- `--dry-run` 只跑只读 git 查询并打印计划,不写文件;检查未通过时同样退出码 1,便于看到「真跑会被拒」
- 版本格式 `x.y.z` 或带 `-` 预发布后缀(`0.2.0-beta.1`);带 `-` 的 tag 会被 Release 自动标为 prerelease;`patch/minor/major` 基于 `package.json` 当前值递增(预发布后缀先丢弃)
- 写入用**文本正则替换**而非解析后重序列化:`tauri.conf.json` 是带中文注释的 JSON5,`JSON.parse` 会丢注释;`Cargo.toml` 只在 `[package]` 段内替换,不误伤依赖声明

> **Gotcha**:`git push --follow-tags` 只推送**附注** tag,轻量 tag(`git tag vX`)会被落下、Actions 不触发。手动打 tag 时必须 `git tag -a vX -m vX`。
>
> **Gotcha**:`cargo metadata --no-deps` 不会写 `Cargo.lock`;去掉 `--no-deps` 又会在离线时因平台专属依赖(如 `android_log-sys`)失败。刷新本 crate 版本用 `cargo update --workspace --offline`,只重解析工作区成员、不下载不编译。

## CI 与工作流约定

- `.github/workflows/ci.yml`:`main` 的 push / PR 门禁,前端 `lint → typecheck:node → test → build`,Rust 在 **windows-latest + ubuntu-24.04** 双平台跑 `fmt --check → clippy --all-targets -D warnings → test`(Windows 覆盖真实 Win32 平台层,Ubuntu 覆盖 stub,见 [tauri/platform-windows.md](../tauri/platform-windows.md));Rust job 先 `mkdir -p ../dist`,因为 `tauri-build` 会检查 `frontendDist` 目录存在
- `.github/workflows/release.yml`:`v*` tag 触发,`verify`(版本一致 + lint/test)→ `build`(4 平台 matrix,`fail-fast: false`,tauri-action 只构建不建 Release)→ `publish`(全部成功后一次性建 Release,附件齐全才发)
- 工作流写法:step `name` 与 `#` 注释中文,决策点写「为什么」;顶层 `permissions: contents: read`,只在需要写的 job 提升;第三方 action 固定大版本 tag、**只选 Node 24 运行时的版本**(托管 runner 2026-09-23 移除 Node 20,`actions/*@v4` 仍是 Node 20,当前用 `checkout@v6` / `upload-artifact@v6` / `download-artifact@v7` / `action-gh-release@v3`);runner 不用已进入弃用流程的镜像(`ubuntu-22.04` 自 2026-09-17 弃用)
- 本地无法跑 Actions,改 yml 后用 `actionlint` 静态校验(`go install github.com/rhysd/actionlint/cmd/actionlint@latest`)

> **Gotcha**:GitHub Actions 表达式的字符串字面量不支持 `\n` 等转义,`join(fromJSON(x), '\n')` 拼出来的是字面 `\n`。需要多行 `path` 时直接写 YAML 块标量(release.yml 的 upload-artifact 按扩展名逐行 glob 就是这么做的)。
