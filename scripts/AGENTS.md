<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-06-02 | Updated: 2026-06-02 -->

# scripts

## 概述
辅助工具脚本目录，包含用于处理模板图片等开发/运维任务的独立脚本。

## 关键文件

| 文件 | 说明 |
|------|------|
| `make_template_transparent.py` | 将模板图片背景转为透明（从四角 flood-fill 检测背景色） |

## AI Agent 指南

### 在此目录工作
- 脚本为独立的 Python 工具，不属于 Rust 构建流程
- 运行需要 Python 3 + `numpy` + `Pillow`

### 常见用法
```bash
python scripts/make_template_transparent.py <input.png> [output.png] [--threshold 30]
```

## 依赖

### 外部
- Python 3
- numpy — 数组操作
- Pillow — 图片读写

<!-- MANUAL: -->
