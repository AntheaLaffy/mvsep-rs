# 练习：Git 命令行

对应课程第 5 讲的练习（节选，按主题归入本 skill），复习用。

1. **考古**。克隆课程网站仓库，把版本历史可视化成图。谁最后修改了 README.md？（提示：`git log` 带文件参数）`_config.yml` 里 `collections:` 这一行的最后一次修改对应什么提交信息？（提示：`git blame` 和 `git show`）
2. **从历史里删文件**。学 Git 的常见错误是提交了不该管的大文件或敏感信息。试着往仓库添加一个文件、做几次提交，然后把它从整个历史中删掉（不只是最新提交）。
3. **stash**。从 GitHub 克隆一个仓库，修改其中一个已有文件：`git stash` 会发生什么？`git log --all --oneline` 看到什么？`git stash pop` 恢复。什么场景下这有用？
4. **git graph 别名**。在 `~/.gitconfig` 里建一个别名，让 `git graph` 输出 `git log --all --graph --decorate --oneline` 的结果（直接编辑文件或用 `git config` 命令都行）。
5. **全局 gitignore**。运行 `git config --global core.excludesfile ~/.gitignore_global` 设定全局忽略文件的位置，然后手动创建该文件，忽略 `.DS_Store` 之类的 OS/编辑器临时文件。
6. **fork + PR**。Fork 课程网站仓库，找一个 typo 或可改进处，在 GitHub 上提交 PR——规范见 `contributing-upstream`；只提有用的 PR，找不到改进点就跳过。
7. **合并冲突模拟**。`git init` 建新仓库，创建 recipe.txt 写几行（比如一个简单食谱）。提交后建两个分支：`git branch salty` 和 `git branch sweet`。在 salty 里改某行（如 "1 cup sugar" → "1 cup salt"）并提交；在 sweet 里把同一行改成别的（如 "2 cups sugar"）并提交。切回 master，依次 `git merge salty`、`git merge sweet`——发生什么？recipe.txt 里的 `<<<<<<<`、`=======`、`>>>>>>>` 标记是什么意思？编辑文件保留想要的内容、删掉标记，用 `git add` + `git commit`（或 `git merge --continue`）完成合并；最后用 `git log --graph --oneline` 看刚造出的合并历史。

> 改编自 MIT The Missing Semester 课程 Lecture 5: Version Control and Git（讲义 + 口播稿，CC BY-NC-SA 4.0）：https://creativecommons.org/licenses/by-nc-sa/4.0/ · 课程站点：https://missing.csail.mit.edu/ · 讲座视频：https://www.youtube.com/watch?v=9K8lB61dl3Y
