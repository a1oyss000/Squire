# Squire Implementation Plan

## Status: pending approval

## Requirements Summary
构建 Squire — 一个基于 Rust+Tauri 的 Windows 桌面游戏自动化应用。通过 OpenCV 模板匹配和 Tesseract OCR 识别游戏界面，结合 Windows API 和 ADB 双模式模拟操作，自动完成 NIKKE/明日方舟等手游日常任务。用户通过 YAML 配置文件定义任务流程，GUI 管理执行。

## RALPLAN-DR Summary

### Principles
1. **模块化分层** — 识别引擎、执行引擎、编排系统严格解耦，通过 trait 接口通信
2. **配置驱动** — 所有任务逻辑由 YAML 定义，核心代码不含游戏特定逻辑
3. **安全 FFI** — OpenCV/Tesseract C 库调用封装在独立 crate 中，unsafe 代码集中管理；RAII 包装所有 C 句柄；所有 FFI 调用通过 `spawn_blocking` 隔离
4. **渐进式 MVP** — 先跑通单个 NIKKE 任务端到端，再扩展功能
5. **可观测性** — 每步操作有日志和可选截图，便于调试和回放
6. **面向提取设计** — Engine API 采用 channel-based 消息传递风格，为未来进程隔离预留机械化重构路径

### Decision Drivers
1. **FFI 复杂度管理** — OpenCV/Tesseract 的 C 绑定是最大技术风险
2. **开发效率** — Tauri v2 + React 前后端分离，各自独立开发迭代
3. **用户体验** — YAML 配置 + GUI 排序的平衡点：灵活但不过度复杂

### Viable Options

#### Option A: Monorepo + Workspace Crates (Chosen)
- Rust workspace 包含多个 crate: `squire-core`(识别+执行), `squire-tasks`(编排), `squire-app`(Tauri)
- 前端 React 在 `src-ui/` 目录
- Pros: 代码共享方便、统一构建、类型安全跨 crate
- Cons: 初始配置稍复杂、编译时间较长

#### Option B: 独立进程 + IPC
- 核心引擎作为独立 CLI 进程，Tauri 通过 stdin/stdout 或 socket 通信
- Pros: 引擎可独立测试和使用、进程隔离更安全、FFI 崩溃不影响 GUI
- Cons: IPC 开销、状态同步复杂、部署需多个二进制
- **Invalidation**: MVP 阶段过度工程化；通过 channel-based API 设计保留未来提取路径作为折中

## Acceptance Criteria
- [ ] 能截取指定窗口/模拟器画面（Windows GDI/DXGI）
- [ ] 模板匹配定位界面元素，匹配率阈值可配置
- [ ] OCR 识别界面文字，支持中英文
- [ ] Windows API SendInput 模拟点击指定坐标
- [ ] ADB 模拟点击模拟器内坐标
- [ ] 解析 YAML 任务配置并按步骤顺序执行
- [ ] 可配置失败策略（重试N次/跳过/暂停）
- [ ] 任务结束检查成功标志（模板或OCR）
- [ ] GUI 显示任务列表、启停、状态
- [ ] GUI 拖拽调整任务顺序
- [ ] GUI 配置每任务独立参数
- [ ] 端到端完成 NIKKE 一个简单日常任务
- [ ] 执行过程有结构化日志

## Implementation Steps

### Phase 1: 项目脚手架 (Day 1-2)
1. 初始化 Rust workspace (`Cargo.toml` workspace members)
2. 创建 crate 结构:
   - `crates/squire-error/` — 统一错误类型 (thiserror)
   - `crates/squire-vision/` — 屏幕捕获 + OpenCV + Tesseract FFI
   - `crates/squire-input/` — Windows API + ADB 点击模拟
   - `crates/squire-engine/` — 任务编排引擎 (channel-based API)
   - `src-tauri/` — Tauri v2 应用入口
   - `src-ui/` — React + TypeScript 前端
3. 配置 Tauri v2 项目 (`tauri.conf.json`)
4. 配置 OpenCV/Tesseract C 库的 build.rs 链接脚本 + 运行时 DLL 检测
5. 设置基本 CI: `cargo check` + `cargo clippy`

### Phase 2: 视觉识别引擎 (Day 3-6)
1. `squire-vision/src/capture.rs` — 窗口截屏 (Windows GDI/DXGI)
   - `fn capture_window(hwnd: HWND) -> Result<Image>`
   - `fn find_window(title: &str) -> Result<HWND>`
   - 内存管理: Image 使用 Arc 引用计数，避免高频截屏内存泄漏
2. `squire-vision/src/matcher.rs` — OpenCV 模板匹配 FFI
   - `fn match_template(screen: &Image, template: &Template) -> Result<MatchResult>`
   - `struct MatchResult { point: Point, confidence: f64 }`
   - RAII: `CvMat` wrapper with `Drop` impl 释放 OpenCV Mat 内存
   - 所有 FFI 调用通过 `tokio::task::spawn_blocking` 派发
3. `squire-vision/src/ocr.rs` — Tesseract OCR FFI
   - `fn recognize_text(image: &Image, region: Rect) -> Result<String>`
   - `fn init_tesseract(lang: &str, data_path: &Path) -> Result<OcrEngine>`
   - RAII: `OcrEngine` wrapper with `Drop` (调用 `TessDeleteBaseAPI`)
   - 线程安全: `OcrEngine` 标记为 `!Send + !Sync`，通过专用 worker thread 访问
   - 所有 FFI 调用通过 `spawn_blocking` 派发
4. `squire-vision/src/lib.rs` — 运行时 DLL 检测
   - 启动时检查 opencv/tesseract DLL 是否存在，缺失时返回友好错误
5. 单元测试: 用静态截图验证匹配和 OCR 准确性

### Phase 3: 操作执行引擎 (Day 5-7)
1. `squire-input/src/winapi.rs` — Windows API 点击
   - `fn click(x: i32, y: i32) -> Result<()>`
   - `fn move_to(x: i32, y: i32) -> Result<()>`
   - `fn drag(from: Point, to: Point) -> Result<()>`
2. `squire-input/src/adb.rs` — ADB 点击
   - `fn adb_tap(x: i32, y: i32, device: &str) -> Result<()>`
   - `fn adb_swipe(from: Point, to: Point, duration_ms: u32) -> Result<()>`
   - `fn list_devices() -> Result<Vec<Device>>`
3. `squire-input/src/lib.rs` — 统一 trait
   - `trait InputBackend { fn click(&self, point: Point) -> Result<()>; ... }`
4. 集成测试: 在记事本窗口验证点击准确性

### Phase 4: 任务编排引擎 (Day 7-10)
1. `squire-engine/src/config.rs` — YAML 配置解析 + 验证
   - `struct TaskDefinition { name, steps, config, success_marker }`
   - `struct StepDef { action, target, timeout, on_fail }`
   - `enum FailStrategy { Retry(u32), Skip, Pause }`
   - `fn validate_task(task: &TaskDefinition) -> Result<()>` — 加载时验证模板文件存在、action 枚举合法、timeout > 0
2. `squire-engine/src/runner.rs` — 任务执行器 (channel-based)
   - `async fn run_task(task: &TaskDefinition, ctx: &mut Context) -> TaskResult`
   - 步骤循环: 识别 → 执行 → 等待 → 验证
   - 通过 `mpsc::channel` 与 vision/input crate 通信（消息传递风格）
   - 支持 `CancellationToken` 中止长时间运行的任务
   - 所有 vision/input 调用通过 channel 派发到 spawn_blocking worker
3. `squire-engine/src/scheduler.rs` — 任务调度
   - `fn run_queue(tasks: Vec<TaskDefinition>) -> Vec<TaskResult>`
   - 按顺序执行启用的任务，收集结果
   - 支持取消整个队列
4. `squire-engine/src/state.rs` — 运行时状态机
   - `enum TaskState { Pending, Running, Success, Failed, Skipped, Cancelled }`
5. YAML schema 示例文件: `tasks/nikke-daily-example.yaml`

### Phase 5: Tauri 集成 + GUI (Day 9-14)
1. Tauri commands (IPC 接口):
   - `get_tasks` / `update_task_order` / `toggle_task`
   - `start_execution` / `stop_execution` / `get_status`
   - `get_task_config` / `update_task_config`
   - `get_logs`
2. React 前端页面:
   - `TaskList` — 可拖拽任务列表 (react-dnd)
   - `TaskConfig` — 每任务参数编辑面板
   - `ExecutionStatus` — 实时执行状态/日志
   - `Settings` — 全局配置 (ADB路径、OCR语言等)
3. 状态管理: Zustand (轻量，适合 Tauri)
4. UI 组件库: shadcn/ui (Tailwind-based)

### Phase 6: 端到端集成 + NIKKE 任务 (Day 13-16)
1. 创建 NIKKE 日常任务 YAML 配置 (如: 领取邮件奖励)
2. 准备 NIKKE 界面模板图片 (按钮截图)
3. 端到端测试: 启动模拟器 → 运行任务 → 验证完成
4. 调试匹配阈值和等待时间
5. 完善错误处理和重试逻辑

### Phase 7: 日志与打磨 (Day 15-17)
1. 结构化日志 (tracing crate)
2. 关键步骤截图保存
3. GUI 日志查看器
4. 错误提示优化
5. 基本文档: README + YAML 配置说明

## Risks and Mitigations
| Risk | Impact | Mitigation |
|------|--------|------------|
| OpenCV FFI 编译/链接问题 | 高 | 使用 vcpkg 管理 C 依赖; 提前验证 build.rs; 运行时 DLL 检测 |
| FFI 崩溃 (segfault) 导致整个应用退出 | 高 | RAII 包装所有 C 句柄; channel-based API 预留进程提取路径; panic=abort 时保存状态 |
| Tesseract 中文识别准确率 | 中 | 支持 PaddleOCR 作为备选; 限定 ROI 区域; 可配置匹配阈值 |
| 模拟器窗口截屏黑屏 | 中 | DXGI 优先，GDI 降级; BitBlt 兼容模式 |
| async runtime 被 FFI 阻塞 | 高 | 所有 FFI 调用强制通过 spawn_blocking; Tesseract 专用 worker thread |
| ADB 连接不稳定 | 低 | 自动重连 + 超时重试 |
| 游戏更新导致模板失效 | 低 | 用户可自行更新模板图片 |
| 高频截屏内存压力 | 中 | Image 使用 Arc 引用计数; 截屏间隔可配置; 旧帧及时释放 |

## Verification Steps
1. `cargo build` 全 workspace 编译通过
2. `cargo test` 单元测试通过 (视觉引擎用静态图片)
3. `cargo clippy` 无 warning
4. Tauri dev 模式 GUI 可正常启动
5. 手动验证: 截屏 → 模板匹配 → 点击 流程跑通
6. 端到端: NIKKE 模拟器中完成一个简单任务

## ADR: Architecture Decision Record

### Decision
采用 Rust workspace monorepo 架构，核心功能拆分为 3 个 crate (vision/input/engine)，通过 Tauri v2 IPC 暴露给 React 前端。

### Drivers
- FFI 安全性需要集中管理 unsafe 代码
- 前后端独立开发迭代的需求
- MVP 快速验证的时间压力

### Alternatives Considered
- **独立进程 + IPC**: 过度工程化，MVP 阶段不需要进程隔离
- **Python 子进程**: 引入额外运行时依赖，部署复杂度增加
- **纯 Rust GUI (egui/iced)**: 生态不如 Web 前端成熟，拖拽等交互实现困难

### Why Chosen
Monorepo workspace 在保持模块化的同时避免了 IPC 开销，Tauri v2 提供了成熟的前后端桥接，React 生态有丰富的拖拽/UI 组件可用。Engine API 采用 channel-based 消息传递设计，使未来提取为独立进程成为机械化重构而非架构重写。

### Consequences
- 编译时间较长（全量构建需要编译 OpenCV 绑定）
- 需要在 Windows 上配置 vcpkg 或手动管理 C 库路径
- 前端开发者需要理解 Tauri command 约定
- FFI 崩溃仍会影响整个进程（MVP 接受此风险，channel API 为未来隔离预留路径）

### Follow-ups
- 评估是否需要增量编译优化 (sccache)
- 考虑未来是否需要插件系统支持第三方任务扩展
- 评估 PaddleOCR 作为 Tesseract 备选的可行性
- 稳定后考虑将 engine 提取为独立子进程以实现崩溃隔离

## Revision Changelog
1. 新增 Principle 6: 面向提取设计 (channel-based API)
2. 新增 `crates/squire-error/` 统一错误类型
3. Phase 2: 增加 RAII wrappers (CvMat/OcrEngine with Drop)
4. Phase 2: OcrEngine 标记 !Send+!Sync，专用 worker thread
5. Phase 2: 所有 FFI 调用强制 spawn_blocking
6. Phase 2: 运行时 DLL 检测 + 友好错误
7. Phase 2: Image 内存管理 (Arc 引用计数)
8. Phase 4: YAML 加载时 schema 验证 (validate_task)
9. Phase 4: Engine runner 采用 channel 消息传递风格
10. Phase 4: 新增 CancellationToken 支持任务取消
11. Option B 评估更新: 承认崩溃隔离优势，synthesis 为 channel API
12. 风险表新增: FFI 崩溃、async 阻塞、内存压力
