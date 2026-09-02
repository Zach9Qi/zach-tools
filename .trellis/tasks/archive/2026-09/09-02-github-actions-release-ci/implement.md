# 执行计划

前置阅读:`.trellis/spec/guides/project-conventions.md`(中文注释 / 提交规范)、`.trellis/spec/tauri/quality-and-security.md`(校验命令)、`.trellis/spec/frontend/quality-guidelines.md`(前端门禁)。

## Step 0 TS 工程化配置

1. `tsconfig.node.json`:include 加 `scripts/**/*.ts`,补 `strict` / `types: ["node"]` / `noEmit` / `target`+`lib` ES2022(design.md §4.4)
2. `package.json` scripts:`format` 范围改 `src/ scripts/`
3. 类型检查覆盖 Node 侧配置与 `scripts/`:最终以 `542e57f` 落地——根 `tsconfig.json` 改壳 + `tsconfig.app.json`,`build` 改 `vue-tsc -b`(取代最初的 `typecheck:node` 独立脚本);先对现有 `vite.config.ts` 跑一次,若暴露遗留类型错误一并修

## Step 1 版本与发布脚本

1. 新建 `scripts/version.ts`:导出 `readVersions()` / `writeVersions(ver)` / `refreshCargoLock()` / `assertConsistent(tag?)`,带 `check [tag]` CLI 入口(design.md §4.3)
2. 新建 `scripts/release.ts`:参数解析 → 安全检查 → 调用 version.ts → git commit/tag/push(design.md §4.2);所有提示中文,导出函数带 `/** */` 注释
3. `package.json` 增加 `release` / `version:check` scripts
4. 本地验证:
   - `bun run version:check` → 通过;`bun run version:check v9.9.9` → 失败、中文提示、退出码 1
   - `bun run release 0.1.1 --dry-run` → 打印计划,`git status` 仍干净
   - 工作区有改动时 `bun run release 0.1.1` → 拒绝执行
   - `bun run release 0.1.1 --no-push` 完整路径:安全检查要求工作区干净且与远端同步,而实现阶段脚本本身尚未提交,因此在**系统临时目录的一次性克隆**里验证,不碰真实仓库:`git clone <本仓库> $TMP/zt` → 把新增/修改文件(`scripts/`、`package.json`、`tsconfig.node.json`)复制进去并 commit → `git init --bare $TMP/remote.git` 作临时远端、`git remote set-url origin` 指向它并 `git push -u origin main` → 在克隆里执行 `bun run release 0.1.1 --no-push` → 检查三处 + Cargo.lock 版本、commit `chore(release): v0.1.1`、tag `v0.1.1`、`git show` 确认 tauri.conf.json 注释完整 → 删除临时目录
5. `bun run lint && bun run build && bun run format` 通过

## Step 2 `.github/workflows/ci.yml`

1. 按 design.md §3 编写;所有 step 带中文 `name`,关键决策处写 `#` 注释(如 `mkdir dist` 的原因)
2. Linux apt 依赖抽成一个 step,`if: runner.os == 'Linux'`
3. 本地无法执行 Actions,用 `actionlint` 做静态校验:本机有 Go、无 docker,用 `go install github.com/rhysd/actionlint/cmd/actionlint@latest` 后执行 `$(go env GOPATH)/bin/actionlint`;若网络不可用则至少 `bun x js-yaml` 解析 yml 语法

## Step 3 `.github/workflows/release.yml`

1. 按 design.md §2 编写 verify → build(matrix include 4 项)→ publish
2. tauri-action 用法(只构建不建 Release:不传 `tagName` / `releaseName` / `releaseId`):
   ```yaml
   - uses: tauri-apps/tauri-action@v1
     env:
       GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
     with:
       args: ${{ matrix.args }}
   - uses: actions/upload-artifact@v4
     with:
       name: bundles-${{ matrix.name }}
       if-no-files-found: error
       path: |
         src-tauri/target/**/bundle/msi/*.msi
         src-tauri/target/**/bundle/nsis/*.exe
         src-tauri/target/**/bundle/dmg/*.dmg
         src-tauri/target/**/bundle/deb/*.deb
         src-tauri/target/**/bundle/rpm/*.rpm
         src-tauri/target/**/bundle/appimage/*.AppImage
   ```
   注意:不用 `steps.<id>.outputs.artifactPaths`(GitHub 表达式字符串不支持 `\n` 转义,拼不出多行 path),按扩展名 glob 收集,见 design.md §2.1
3. publish job:`actions/download-artifact@v4`(`merge-multiple: true` 拉平到一个目录)→ `softprops/action-gh-release@v2`(`files: bundles/**`,`generate_release_notes: true`,`prerelease: ${{ contains(github.ref_name, '-') }}`)
4. `permissions` 最小化:顶层 `contents: read`,publish `contents: write`
5. 同 Step 2 做静态校验

## Step 4 文档

1. 若根目录无 `README.md` 则新建,增加「发布流程」一节(`bun run release <ver>` 一条命令及 `--dry-run` / `--no-push` 说明 + 平台支持说明:Windows 完整功能,macOS/Linux 仅可编译安装、平台能力为 stub)
2. 可顺手修 `Cargo.toml` 的 `authors` / `description`(非验收项)

## Step 5 质量检查

- `bun run format && bun run lint && bun run test && bun run build` 全过
- `cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
- yml 静态校验无错误
- 全部注释 / 输出为中文

## Step 6 提交与验证(用户侧)

- 提交拆分建议:
  1. `build(scripts): 新增一键发布与版本校验脚本,scripts/ 纳入类型检查`
  2. `ci: 新增 main 分支前后端质量门禁`
  3. `ci: 新增 v* tag 触发的跨平台打包发布工作流`
  4. `docs: 补充发布流程说明`
- push 到 main 观察 ci.yml 绿;打 `v0.1.0` 试跑 release.yml。首次真实运行前可先推一个 `v0.0.1-test` 观察,验证后删除 tag 与 Release
