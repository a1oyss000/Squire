<!-- Parent: ../../AGENTS.md -->
<!-- Generated: 2026-06-05 -->

# resources

## 概述
所有引擎运行时资源的根目录。

## 子目录

| 目录 | 用途 |
|------|------|
| `tasks/` | 任务元数据 YAML（`<name>.yaml`）及模板图片 |
| `flows/` | 流程节点图（`<name>.flow.yaml`）及共享模板 |

## 格式

### tasks/ — 任务定义
```yaml
name: task-id
label: Human-readable Name
flow: task-id       # 对应 flows/<name>.flow.yaml
```

可选：`group`, `options`, `resources`, `bindings`

### flows/ — 流程节点图
```yaml
entry: root-node
nodes:
  node-id:
    recognize:
      template: resources/tasks/templates/xxx.png
    action: click
    post_wait: 1000
    next: [other-node, end-task]
```

- 模板路径相对于项目根目录，如 `resources/tasks/templates/nikke/mail_button.png`
- `end-task` 是特殊节点名，任务成功结束
- `entry` 节点的 action 不会执行，只作为入口
