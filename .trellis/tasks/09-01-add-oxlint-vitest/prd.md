# 接入 oxlint 与 Vitest

## 背景

前端质量门禁目前只有 `vue-tsc`(类型)+ Prettier(格式),缺 lint 与测试。用户决策:

- lint 选 **oxlint**(不引入 ESLint)。已知情接受的缺口:Vue 模板层规则(oxlint 只 lint SFC 的 script 块,上游 Language Plugins RFC 进行中);类型感知模式(tsgolint)要求 TS 7,项目在 TS ~5.6,暂不启用
- 测试选 **Vitest**,首批只覆盖 `lib/` 纯函数,组件测试(@vue/test-utils)是后续独立决策

## 目标

1. `bun run lint` 可用:oxlint 检查 `src/`,并把「tools 禁止 import `@/launcher/*`」的依赖边界从 spec 文字升级为 lint 强制(`no-restricted-imports` + `src/tools/**` overrides)
2. `bun run test` 可用:Vitest 跑通首批纯函数用例(`tools/clipboard/lib/time.ts` 的相对时间分段、`tools/match.ts` 的输入形态判断)
3. spec 同步:质量规范的校验命令、测试现状、禁止模式表更新

## 约束

- 不引入 ESLint、不装 oxlint-plugin-vize(alpha)、不开 typeAware
- 测试文件与被测源码同目录放置(`xxx.test.ts`),沿用 `@/` 别名导入
- Vitest 用独立 `vitest.config.ts`,不改动 `vite.config.ts`(避免影响 Tauri dev 配置)
- 存量代码如被 lint 命中,能改则改;确属项目既定约定的(如错误处理里的 console.error)配置放行并注释理由

## 验收标准

- [x] `bun run lint` 零错误通过(顺带修复 useLauncher 单元素 Promise.all;.pi 等脚手架目录已排除)
- [x] 在 tools 内临时添加 `@/launcher` 导入能被 lint 报错(验证后移除;注意 gitignore 通配 `*` 只匹配一层,须用 `**`)
- [x] `bun run test` 全部用例通过(11 例:time 相对时间分段边界、match 输入形态判断)
- [x] `bun run build` 不受影响
- [x] spec(quality-guidelines、directory-structure、index)已同步,lint 已知缺口如实记录
