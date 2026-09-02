# 技术设计:GitHub Actions 自动发布与 main 门禁

## 1. 整体结构

```
.github/workflows/
├── ci.yml        # main push / PR 门禁
└── release.yml   # v* tag 触发的构建 + 发布
scripts/
├── release.ts    # 一键发布:检查 → bump → commit → tag → push
├── version.ts    # 版本号读写 / 校验(release.ts 与 CI 共用)
└── lib/git.ts    # git 命令薄封装(可选,若 release.ts 体量小则内联)
package.json      # 新增 scripts: release / version:check;format 范围加 scripts/;build 改 vue-tsc -b
tsconfig.json     # 改为只含 references 的壳(→ tsconfig.app.json + tsconfig.node.json)
tsconfig.app.json # 浏览器侧 src/**(原根配置内容搬入)
tsconfig.node.json # include 加 scripts/**/*.ts,补 strict / types: node / noEmit
```

## 2. release.yml 流水线

```
verify (ubuntu)                 build (matrix, 4 个 job)              publish (ubuntu)
─────────────────    needs   ───────────────────────────   needs   ─────────────────
checkout             ──────▶ checkout                     ──────▶ download-artifact
setup-bun                    setup-bun / rust-toolchain            action-gh-release
bun install                  rust-cache                              - files: 全部安装包
version:check $TAG           apt deps (仅 linux)                     - generate_release_notes
bun run lint / test          tauri-action (仅构建,不发布)
                             upload-artifact
```

### 2.1 为什么不用 tauri-action 直接建 Release

`tauri-apps/tauri-action`(当前稳定大版本 `@v1`,`@v0` 已停更)自带 `tagName`/`releaseName` 可以边构建边建 Release,v1 也已支持 `generateReleaseNotes`,但:

- 每个矩阵 job 各自往同一个 Release 上传,任一平台失败会留下**半成品 Release**,这是不采用它建 Release 的核心原因

改为「构建 job 只产出 artifact → 单独 publish job 一次性建 Release」:

- Release 原子性:全部平台成功才有 Release
- `softprops/action-gh-release@v2` 支持 `generate_release_notes: true`(GitHub 服务端按 commit / PR 生成)
- tauri-action 省略 `tagName`/`releaseName`/`releaseId` 即「只构建不上传」;产物用 `actions/upload-artifact@v4` 按扩展名 glob 收集(`src-tauri/target/**/bundle/{msi,nsis,dmg,deb,rpm,appimage}/*.{msi,exe,dmg,deb,rpm,AppImage}` 逐行列出),`if-no-files-found: error` 兜底
  - 不用 `steps.<id>.outputs.artifactPaths`:它是 JSON 数组字符串,GitHub 表达式的字符串字面量不支持 `\n` 转义,`join(fromJSON(...), '\n')` 拼不出多行 path;glob 更直接且能排除 bundle 目录里的 staging 目录(`*.AppDir`、deb 解包目录、`.app`)
  - `--target x86_64-apple-darwin` 时产物在 `target/x86_64-apple-darwin/release/bundle/`,默认 target 在 `target/release/bundle/`,`**` 同时覆盖两者

### 2.2 构建矩阵

| runner | args | 产物 |
|---|---|---|
| `windows-latest` | (无) | `*.msi`、`*-setup.exe` |
| `macos-latest` | `--target aarch64-apple-darwin` | `*_aarch64.dmg` |
| `macos-latest` | `--target x86_64-apple-darwin` | `*_x64.dmg`(需 `rustup target add`) |
| `ubuntu-24.04` | (无) | `*.deb`、`*.rpm`、`*.AppImage` |

- matrix 用 `include` 列表写死四条,每条带 `platform` / `args` / `target`(供 rust-toolchain 的 `targets` 输入)
- `fail-fast: false`:一个平台挂了其他平台继续跑完,方便一次看到所有问题;publish 因 `needs` 仍不会执行
- Linux 系统依赖(Tauri 2 官方清单):`libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf xdg-utils`;`tray-icon` feature 依赖 appindicator,已包含
- Linux runner 选 `ubuntu-24.04` 而非 Tauri 文档惯用的 `ubuntu-22.04`:GitHub 已宣布 22.04 镜像 2026-09-17 起弃用、2027-04-17 下线(actions/runner-images#14254)。代价是 deb/AppImage 的 glibc 基线抬到 24.04,对更老发行版兼容性下降;Linux 是次要平台,可接受
- macOS / Linux 上功能受限(平台层走 stub),安装包仅保证「可安装可启动」,在 README 发布说明中如实标注

### 2.3 verify job

- 复用 `bun run version:check "$GITHUB_REF_NAME"`,失败信息由脚本输出中文
- lint / test 在 verify 跑一次即可,不必每个平台重复;`bun run build` 由 tauri-action 的 `beforeBuildCommand` 触发,不重复跑
- Rust 侧 fmt/clippy 由 ci.yml 保障(tag 通常打在已过门禁的 main 上),release 不重复跑以省时间

### 2.4 权限与安全

- 顶层 `permissions: contents: read`,仅 publish job 提升 `contents: write`
- 全程使用 `GITHUB_TOKEN`,不引入额外 secret
- 第三方 action 固定大版本 tag,且一律选 Node 24 运行时的版本(GitHub 托管 runner 2026-06-16 起默认 Node 24、2026-09-23 移除 Node 20;`actions/*@v4` 与 `action-gh-release@v2` 仍声明 `node20`):`actions/checkout@v6`、`actions/upload-artifact@v6`、`actions/download-artifact@v7`、`softprops/action-gh-release@v3`、`oven-sh/setup-bun@v2`、`dtolnay/rust-toolchain@stable`、`Swatinem/rust-cache@v2`、`tauri-apps/tauri-action@v1`;不追 `main`

### 2.5 prerelease 判定

`prerelease: ${{ contains(github.ref_name, '-') }}` —— `v0.2.0-beta.1` 自动标 prerelease,正式版不带 `-`。

## 3. ci.yml

```yaml
on:
  push: { branches: [main] }
  pull_request: { branches: [main] }
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true
jobs:
  frontend:  # ubuntu-latest
    bun install --frozen-lockfile → bun run lint → bun run test → bun run build(vue-tsc -b 含 Node 侧类型检查)
  rust:      # matrix: windows-latest, ubuntu-24.04;working-directory: src-tauri
    (ubuntu 装 apt deps) → cargo fmt --check → cargo clippy --all-targets -- -D warnings → cargo test
```

- 后端 job 需要 `dist/` 存在吗?——`tauri-build` 的 `build.rs` 在 `frontendDist` 不存在时会报错。处理:rust job 先 `mkdir -p dist`(空目录即可满足 build.rs 检查),不必真跑前端构建
- Rust 双平台的理由:Windows 覆盖真实 Win32 代码,Ubuntu 覆盖 `stub.rs`,对应 spec 中「保证任何平台都能编译」的约束
- `-D warnings` 与 spec「clippy 无警告」一致

## 4. 发布脚本

### 4.1 接口

```
bun run release 0.2.0              # 一键发布:检查 → 写版本 → commit → tag → push
bun run release patch|minor|major  # 基于 package.json 当前版本递增
bun run release 0.2.0 --dry-run    # 只打印计划,不改文件不跑 git
bun run release 0.2.0 --no-push    # 本地 commit + tag 后停下
bun run version:check              # 三处一致性(CI / 本地自检)
bun run version:check v0.2.0       # 三处一致 且 == tag 去 v
```

package.json:
```json
"release": "bun run scripts/release.ts",
"version:check": "bun run scripts/version.ts check",
"build": "vue-tsc -b && vite build",
"format": "prettier --write src/ scripts/"
```

### 4.2 release.ts 流程

```
解析参数(版本 / patch|minor|major / --dry-run / --no-push)
  → 安全检查(全部通过才继续,失败中文报错 exit 1)
      git status --porcelain 为空        「工作区有未提交改动」
      git branch --show-current == main  「请在 main 分支发布」
      git fetch + rev-parse 比对 origin/main「本地与远端不同步,请先 pull/push」
      git tag -l vX.Y.Z 为空             「tag 已存在」
  → 计算目标版本(patch/minor/major 从 package.json 当前版本递增)
  → version.ts 写三处 + 刷新 Cargo.lock
  → git add -A 上述 4 个文件(精确列出,不 add 其他)
  → git commit -m "chore(release): vX.Y.Z"
  → git tag vX.Y.Z
  → git push --follow-tags origin main   (--no-push 时跳过并提示手动命令)
  → 打印 Actions 地址 https://github.com/<owner>/<repo>/actions
```

- 子进程用 `execFileSync("git", [...])`(数组参数,不拼 shell 字符串)
- `--dry-run` 下所有写文件 / git 写操作替换为打印,只读的 git 查询照常执行,确保 dry-run 输出真实
- 不引 `bumpp` 等工具的原因:它们对 `Cargo.toml` / `tauri.conf.json` 是全文盲替换版本字符串,依赖里同版本号会误改,且不处理 `Cargo.lock`;自写可精确定位且保留 JSON5 注释

### 4.3 version.ts 实现要点

- **文本正则替换而非 JSON 重序列化**:`tauri.conf.json` 是带注释的 JSON5,`JSON.parse` 会丢注释;`Cargo.toml` 亦是 TOML。三处统一用「锁定行」的正则:
  - `package.json`:`/^(\s*"version":\s*")([^"]+)(")/m`(第一处匹配即根 version;devDependencies 里的 `"version"` 键不会出现在行首缩进 2 空格的位置——为稳妥,限定只替换**首个**匹配)
  - `Cargo.toml`:只在 `[package]` 段内替换 `^version = "..."`(截取到下一个 `[` 段头)
  - `tauri.conf.json`:`/^(\s*"version":\s*")([^"]+)(")/m` 首个匹配
- **读取**(check)用同样的正则抓取,不依赖解析器,保证读写一致
- **Cargo.lock 刷新**:改完 `Cargo.toml` 后执行 `cargo metadata --format-version 1 --no-deps --offline`(cwd `src-tauri`),cargo 会更新 lock 中本 crate 的版本且不编译;若 cargo 不可用则打印警告提示手动处理,不中断
- semver 校验:`/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/`
- 退出码:成功 0,校验失败 1,输出中文;所有 `console.log/error` 用中文
- 用 Node 内置 `fs`/`child_process`,不加依赖;`@types/node` 已在 devDependencies

### 4.4 TS 工程化

- `scripts/*.ts` 此前不在任何 tsconfig 内(`tsconfig.json` 只含 `src/**`,`tsconfig.node.json` 只含 `vite.config.ts`),bun 直跑不做类型检查,编辑器提示也会退化
- 修改 `tsconfig.node.json`:`include` 加 `scripts/**/*.ts`(及 `vitest.config.ts`);`compilerOptions` 补 `strict: true`、`types: ["node"]`、`noEmit: true`、`target: ES2022`、`lib: ["ES2022"]`(脚本无 DOM)
- **最终方案(`542e57f`,取代最初的 `typecheck:node` 独立脚本)**:对齐 Vite 新模板结构——根 `tsconfig.json` 改为 `files: []` + `references: [tsconfig.app.json, tsconfig.node.json]` 的壳,`build` 改为 `vue-tsc -b && vite build`,`-b` 顺着 references 一次检查浏览器侧与 Node 侧,CI 不需要额外步骤
  - 最初实现时曾因根 `tsconfig.json` 既含 `include` 又含 `references`,被迫保留 `composite: true` 且不能在文件里写 `noEmit`(`vue-tsc --noEmit` 报 TS6306 / TS6310);改成壳结构后 TS 5.6 的 `-b` 不再要求被引用项目开 `composite`,`noEmit` 可直接写回配置
  - `-b` 会写增量产物,两份配置的 `tsBuildInfoFile` 都指到 `node_modules/.tmp/`,不在仓库根落 tsbuildinfo
- 只用 Node 内置 API + `@types/node`,不装 `bun-types`,脚本 node / bun 皆可运行
- oxlint `ignorePatterns` 未排除 `scripts/`,lint 自然覆盖;prettier `format` 脚本范围补上 `scripts/`

### 4.5 可选简化(本轮不采纳,记录)

Tauri 2 允许省略 `tauri.conf.json` 的 `version`,自动继承 `Cargo.toml`。可把三处减为两处,但会让配置文件少一个「显式版本」注释锚点,与现有配置注释风格不一致;脚本已统一处理,不值得为此改动。

## 5. 发布流程(用户视角)

```bash
bun run release 0.2.0      # 或 bun run release minor
# → 脚本完成 bump / commit / tag / push
# → Actions 自动构建 4 平台 → Release 出现在 GitHub
```

业界对照:Tauri 官方模板是「手动 tag 触发」;Vue 生态常用 `bumpp` 一键 bump+tag+push;有 PR 流程的团队用 `release-please` / `changesets` 全自动。本项目选「自写一键脚本」,兼顾单命令体验与多文件精确改写。

## 6. 风险与应对

| 风险 | 应对 |
|---|---|
| macOS 首次构建可能因未签名产生 warning | 仅 warning 不阻断;签名列入非目标 |
| Linux 构建缺依赖 | 使用官方依赖清单;首次失败按日志补包 |
| `bun.lock` 与 `--frozen-lockfile` 冲突 | 当前 lock 已提交,CI 与本地版本一致即可;若失败提示本地重新 `bun install` |
| Windows runner 编译慢(~10min 冷启) | rust-cache 缓存 `target/`,以 `Cargo.lock` 为 key |
| tag 打在未过门禁的 commit | verify job 跑 lint/test 兜底;fmt/clippy 依赖 ci.yml |
