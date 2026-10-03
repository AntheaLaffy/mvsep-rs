---
name: code-quality-tooling
description: >
  代码质量工具：格式化器（Prettier/Black/gofmt/EditorConfig）、linter（规则/预设/禁用/自动修复）、
  semgrep 语义 grep 自定义规则。Use when the user asks to set up a formatter or linter, fix lint
  errors, or write custom lint rules. 触发于「配格式化/linter/修 lint 错误/写 lint 规则」。
---

# 代码质量工具

主线：把表层琐事交给机器，人专注深层问题。格式化器统一风格（' 还是 "、运算符空格、import 排序、行长），linter 静态分析找反模式，semgrep 在 AST 层按你写的模式搜代码。工具随语言不同，概念通用。

## 格式化

- 自动格式化把风格争论变成机器决定。Prettier 高度可配置——配置文件要提交进版本控制；Black/gofmt 刻意少配置或不配置，反自行车棚。格式化器通常有 `--check`（只报 diff 不改码）与直接修两种模式，前者正是 CI 的用法——每个项目都该配一个。
- IDE 集成：输入时/保存时自动格式化；EditorConfig 向 IDE 传达项目级设置（如缩进大小），一并提交。
- 格式化管「表层」；语言服务器里的质量功能（见 `dev-environment`）与 linter 看得更深。

## Lint

- 静态分析（不运行代码）找反模式：内置规则 + 项目级预设；误报可按文件/按行禁用。
- 好 linter 自带教学：Ruff 内置约 800 条规则，每条文档解释查什么、为什么坏、替代写法（如 EM101：raise 里用字符串字面量会让错误信息在回溯里重复出现——先赋给变量）。
- 自动修复分两档：`--fix` 只做保语义的安全转换，`--unsafe-fixes` 能改更多但可能破坏行为；剩下的只能人修。误报按行禁用如 `# noqa: S602`（shell=True 调用）——压掉前想清楚为什么要例外。
- 让 AI 修 lint 错误时，配好环境让它能自己跑 linter 看结果、进入迭代循环（见 `agentic-coding`），并审查它有没有改坏。
- semgrep：AST 层的「语义 grep」，多语言，可写自定义规则——如 `semgrep -l python -e "subprocess.Popen(..., shell=True, ...)"` 拦危险调用；项目级禁令（禁 pickle、禁 exec）同样一行规则搞定。grep 正则会被换行、加参数等小改动打破（打地鼠），AST 匹配不受影响。

## 练习

学习材料在 `exercises.md`。

> 改编自 MIT The Missing Semester 课程 Lecture 9: Code Quality（讲义 + 口播稿，CC BY-NC-SA 4.0）：https://creativecommons.org/licenses/by-nc-sa/4.0/ · 课程站点：https://missing.csail.mit.edu/ · 讲座视频：https://www.youtube.com/watch?v=XBiLUNx84CQ
