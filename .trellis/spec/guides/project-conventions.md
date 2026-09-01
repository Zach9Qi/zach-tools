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
