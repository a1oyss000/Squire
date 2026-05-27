# Deep Interview Spec: Squire - Game Automation App

## Metadata
- Interview ID: squire-game-auto-001
- Rounds: 10
- Final Ambiguity Score: 16.2%
- Type: greenfield
- Generated: 2026-05-27
- Threshold: 0.2
- Threshold Source: default
- Initial Context Summarized: no
- Status: PASSED

## Clarity Breakdown
| Dimension | Score | Weight | Weighted |
|-----------|-------|--------|----------|
| Goal Clarity | 0.92 | 0.40 | 0.368 |
| Constraint Clarity | 0.82 | 0.30 | 0.246 |
| Success Criteria | 0.72 | 0.30 | 0.224 |
| **Total Clarity** | | | **0.838** |
| **Ambiguity** | | | **16.2%** |

## Topology
| Component | Status | Description | Coverage |
|-----------|--------|-------------|----------|
| 屏幕捕获与识别引擎 | active | OpenCV模板匹配+OCR文字识别 | 技术方案已定(Rust FFI) |
| 操作执行引擎 | active | 模拟鼠标/键盘/ADB操作 | 双模式已确认 |
| 任务编排系统 | active | YAML配置驱动的任务流程 | 核心逻辑已明确 |
| 用户界面 | active | Tauri+React桌面应用 | 功能范围已定 |
| 日志与监控 | active | 执行记录与状态报告 | 基本需求明确 |
| 系统配置 | active | 全局设置与参数管理 | 基本需求明确 |

## Goal
构建一个 Windows 桌面应用（Squire），通过 OpenCV 模板匹配和 Tesseract OCR 识别游戏界面元素，结合 Windows API 和 ADB 双模式模拟用户操作，自动完成手游（首批支持 NIKKE、明日方舟）的日常任务。用户通过 YAML 配置文件定义任务流程，通过 GUI 管理任务执行顺序和参数。

## Constraints
- 开发语言：Rust（后端/核心引擎）+ TypeScript/React（前端 GUI）
- 桌面框架：Tauri v2
- 图像识别：通过 Rust FFI 调用 OpenCV C 库
- 文字识别：通过 Rust FFI 调用 Tesseract C 库
- 点击模拟：Windows API (SendInput) + ADB 双模式
- 任务配置：YAML 格式，一个任务一个文件
- 目标平台：Windows 10/11
- 目标游戏：NIKKE、明日方舟（通过模拟器运行），架构可扩展
- 失败策略：每个任务可配置（重试N次/跳过/暂停等待）
- 成功判定：任务结束时检查特定标志（文字或图标模板匹配）

## Non-Goals
- 不做移动端原生应用
- 不做云端/远程控制
- 不做游戏内存读取或注入（纯视觉识别）
- 第一版不做可视化流程编辑器（拖拽节点连线）
- 第一版不做录制回放功能
- 不做反检测/绕过游戏安全机制

## Acceptance Criteria
- [ ] 能截取指定窗口/模拟器画面
- [ ] 能通过模板匹配定位界面元素（按钮、图标）
- [ ] 能通过 OCR 识别界面文字
- [ ] 能通过 Windows API 模拟点击指定坐标
- [ ] 能通过 ADB 模拟点击模拟器内坐标
- [ ] 能解析 YAML 任务配置文件并按步骤执行
- [ ] 支持可配置的失败策略（重试/跳过/暂停）
- [ ] 任务结束时能检查成功标志
- [ ] GUI 能显示任务列表、启停执行、查看状态
- [ ] GUI 能拖拽调整任务执行顺序
- [ ] GUI 能配置每个任务的独立参数
- [ ] 能实际完成 NIKKE 的一个简单日常任务流程
- [ ] 执行过程有日志记录

## Assumptions Exposed & Resolved
| Assumption | Challenge | Resolution |
|------------|-----------|------------|
| 目标是PC游戏 | 实际是什么平台？ | 手游通过模拟器运行，同时支持PC窗口 |
| 需要硬编码游戏脚本 | 是否通用？ | 用户自定义YAML配置，架构可扩展 |
| Python是唯一选择 | 为什么不用Rust？ | 用户选择Rust+Tauri，FFI调C库 |
| 只需要简单点击 | 失败怎么办？ | 可配置失败策略（重试/跳过/暂停） |
| 需要复杂的可视化编辑器 | MVP需要吗？ | 第一版用YAML配置+GUI排序，不做节点编辑器 |

## Technical Context
- **架构**: Tauri v2 桌面应用（Rust 后端 + React/TS 前端）
- **图像处理**: Rust FFI → OpenCV C 库（模板匹配）
- **文字识别**: Rust FFI → Tesseract C 库
- **截屏**: Windows GDI/DXGI 截取指定窗口
- **点击模拟**: Windows API SendInput（PC窗口）+ ADB（模拟器）
- **任务定义**: YAML 文件，每任务一个文件，包含步骤序列和参数
- **状态管理**: 前端 React state + Rust 后端任务状态机
- **进程通信**: Tauri IPC（前端↔后端）

## Ontology (Key Entities)
| Entity | Type | Fields | Relationships |
|--------|------|--------|---------------|
| GameWindow | core | handle, title, rect, type(PC/emulator) | 被 ScreenCapture 截取 |
| Template | core | image_path, name, threshold, region | 被 Matcher 用于匹配 |
| OCRTarget | core | region, expected_text, language | 被 OCREngine 识别 |
| TaskFlow | core | name, steps[], config, enabled | 包含多个 Step |
| Step | core | action, target, timeout, on_fail | 属于 TaskFlow |
| TaskConfig | supporting | params, fail_strategy, retry_count | 属于 TaskFlow |
| ActionExecutor | core | mode(winapi/adb), target_coord | 执行 Step 的动作 |
| SuccessMarker | supporting | type(template/ocr), target, threshold | 验证 TaskFlow 完成 |
| ExecutionLog | supporting | timestamp, step, result, screenshot | 记录执行过程 |
| AppConfig | supporting | global_settings, adb_path, ocr_lang | 全局配置 |

## Ontology Convergence
| Round | Entity Count | New | Changed | Stable | Stability |
|-------|-------------|-----|---------|--------|-----------|
| 1 | 3 | 3 | - | - | N/A |
| 2 | 5 | 2 | 0 | 3 | 60% |
| 4 | 8 | 3 | 0 | 5 | 63% |
| 7 | 12 | 3 | 1 | 8 | 75% |
| 10 | 13 | 1 | 0 | 12 | 92% |

## Interview Transcript
<details>
<summary>Full Q&A (10 rounds)</summary>

### Round 0 - Topology
**Q:** 拓扑确认：5个组件是否正确？
**A:** 增加系统配置组件（共6个）

### Round 1
**Q:** 自动化目标是哪种场景？（PC/模拟器/两者）
**A:** PC + 模拟器都支持
**Ambiguity:** 85.5%

### Round 2
**Q:** 针对哪个游戏？还是通用框架？
**A:** 先针对明日方舟、NIKKE，架构可扩展
**Ambiguity:** 74.5%

### Round 3
**Q:** 具体要自动化哪些日常任务？
**A:** 本质是识别界面→点击按钮→界面跳转，用户自己配置流程
**Ambiguity:** 65.5%

### Round 4
**Q:** 用户如何配置任务流程？
**A:** 结构化配置文件(一任务一文件) + GUI拖拽排序/开关 + 每任务独立参数
**Ambiguity:** 59.8%

### Round 5
**Q:** 技术栈偏好？
**A:** Rust + Tauri
**Ambiguity:** 53.2%

### Round 5b
**Q:** Rust中OpenCV/OCR的实现路径？
**A:** Rust FFI 调用 C 库
**Ambiguity:** 48.7%

### Round 6
**Q:** 点击模拟方式？
**A:** Windows API + ADB 双模式
**Ambiguity:** 43.9%

### Round 7
**Q:** 如何判定任务步骤成功？
**A:** 结束时检查特定标志（文字或图标）
**Ambiguity:** 37.3%

### Round 8
**Q:** 识别失败时如何处理？
**A:** 可配置的失败策略（每任务自定义）
**Ambiguity:** 33.9%

### Round 9
**Q:** Tauri前端框架选择？
**A:** React + TypeScript
**Ambiguity:** 30.0%

### Round 10
**Q:** MVP需要达到什么程度？
**A:** 核心引擎+任务流程+基本GUI+完成NIKKE一个简单任务
**Ambiguity:** 16.2%

</details>
