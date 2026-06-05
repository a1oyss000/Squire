use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use squire_input::InputBackend;
use tokio::sync::{mpsc, watch};
use tracing::{debug, info, warn};

use crate::recognize::{evaluate, VisionProvider};
use crate::action;
use crate::event::EngineEvent;
use crate::schema::flow::{NextItem, NodeDef};
use crate::trace::{TraceEntry, TraceLog};

// ---- LoadedTask -------------------------------------------------------------

pub struct LoadedTask {
    pub name: String,
    pub label: String,
    pub entry: String,
    pub nodes: HashMap<String, NodeDef>,
    pub disabled_nodes: HashSet<String>,
}

// ---- ExecutorConfig ---------------------------------------------------------

pub struct ExecutorConfig {
    pub retry_interval_ms: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        ExecutorConfig { retry_interval_ms: 500 }
    }
}

// ---- ExecutorStatus ---------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutorStatus {
    Running,
    Success,
    Failed(String),
    Cancelled,
}

// ---- Frame ------------------------------------------------------------------

struct Frame {
    node_id: String,
    next_pointer: usize,
    /// When this frame was pushed, was it via a jumpback transition?
    via_jumpback: bool,
    /// Track when each target candidate was first attempted (for timeout).
    first_tried: HashMap<String, Instant>,
}

// ---- Executor ---------------------------------------------------------------

pub struct Executor {
    task: LoadedTask,
    task_name: String,
    call_stack: Vec<Frame>,
    hit_counts: HashMap<String, u32>,
    trace_log: TraceLog,
    config: ExecutorConfig,
    vision: Arc<dyn VisionProvider>,
    input: Arc<dyn InputBackend>,
    cancel: watch::Receiver<bool>,
    event_tx: mpsc::Sender<EngineEvent>,
    pub status: ExecutorStatus,
}

impl Executor {
    pub fn new(
        task: LoadedTask,
        vision: Arc<dyn VisionProvider>,
        input: Arc<dyn InputBackend>,
        cancel: watch::Receiver<bool>,
        config: ExecutorConfig,
        event_tx: mpsc::Sender<EngineEvent>,
    ) -> Self {
        let task_name = task.name.clone();
        Executor {
            task,
            task_name,
            call_stack: Vec::new(),
            hit_counts: HashMap::new(),
            trace_log: TraceLog::new(),
            config,
            vision,
            input,
            cancel,
            event_tx,
            status: ExecutorStatus::Running,
        }
    }

    fn hit_count(&self, node_id: &str) -> u32 {
        *self.hit_counts.get(node_id).unwrap_or(&0)
    }

    fn is_skippable(&self, node_id: &str, first_tried_at: Option<Instant>) -> Option<&'static str> {
        if node_id == "end-task" {
            return None;
        }
        if self.task.disabled_nodes.contains(node_id) {
            return Some("disabled by binding");
        }
        let node = match self.task.nodes.get(node_id) {
            Some(n) => n,
            None => return Some("node not found in flow"),
        };
        if let Some(max) = node.max_hit {
            if self.hit_count(node_id) >= max {
                return Some("max_hit reached");
            }
        }
        if let (Some(timeout_ms), Some(tried_at)) = (node.timeout, first_tried_at) {
            if tried_at.elapsed() >= Duration::from_millis(timeout_ms) {
                return Some("timeout");
            }
        }
        None
    }

    fn record_trace(&mut self, from: &str, to: &str, cond: Option<bool>, hit: u32) {
        let depth = self.call_stack.len();
        self.trace_log.push(TraceEntry {
            timestamp: Instant::now(),
            from_node: from.to_string(),
            to_node: to.to_string(),
            condition_result: cond,
            stack_depth: depth,
            hit_count_at_transition: hit,
        });
    }

    fn emit(&self, event: EngineEvent) {
        let _ = self.event_tx.try_send(event);
    }

    fn emit_log(&self, level: &str, message: String) {
        self.emit(EngineEvent::Log {
            task: self.task_name.clone(),
            level: level.to_string(),
            message,
        });
    }

    /// Format the call stack as "root > child > current"
    fn stack_path(&self) -> String {
        self.call_stack
            .iter()
            .map(|f| f.node_id.as_str())
            .collect::<Vec<_>>()
            .join(" > ")
    }

    pub fn dump_trace(&self) -> String {
        self.trace_log.dump_jsonl()
    }

    /// Main execution loop. Runs until Success, Failed, or Cancelled.
    pub async fn run(&mut self) -> ExecutorStatus {
        // Push the entry frame
        let entry = self.task.entry.clone();
        let disabled_count = self.task.disabled_nodes.len();
        let total_nodes = self.task.nodes.len();
        info!(task = %self.task_name, entry = %entry, nodes = total_nodes, disabled = disabled_count, "▶ 任务启动");
        self.emit_log("info", format!(
            "▶ 任务启动 | 入口: {entry} | 节点数: {total_nodes} | 禁用: {disabled_count}"
        ));
        self.push_frame(&entry, false);

        loop {
            // Check cancellation
            if *self.cancel.borrow() {
                info!(task = %self.task_name, "■ 任务被取消");
                self.emit_log("info", "■ 任务被取消".into());
                self.status = ExecutorStatus::Cancelled;
                return self.status.clone();
            }

            // Stack empty → success
            let stack_len = self.call_stack.len();
            if stack_len == 0 {
                info!(task = %self.task_name, "✔ 任务成功完成");
                self.emit_log("info", "✔ 任务成功完成".into());
                self.status = ExecutorStatus::Success;
                return self.status.clone();
            }

            let current_node_id = self.call_stack[stack_len - 1].node_id.clone();
            let path = self.stack_path();

            // "end-task" pseudo-node
            if current_node_id == "end-task" {
                self.call_stack.pop();
                info!(task = %self.task_name, "● 到达 end-task");
                self.emit_log("info", "● 到达 end-task 终止节点".into());
                self.status = ExecutorStatus::Success;
                return self.status.clone();
            }

            let node = match self.task.nodes.get(&current_node_id).cloned() {
                Some(n) => n,
                None => {
                    warn!(task = %self.task_name, node = %current_node_id, path = %path, "✗ 节点未在流程中定义");
                    self.emit_log("warn", format!(
                        "✗ 节点 [{current_node_id}] 未在流程中定义 | 调用链: {path}"
                    ));
                    self.call_stack.pop();
                    continue;
                }
            };

            let next_items = match &node.next {
                Some(items) if !items.is_empty() => items.clone(),
                _ => {
                    // No next list: node is a leaf, pop it
                    self.complete_node(&current_node_id);
                    continue;
                }
            };

            // Capture screenshot for this evaluation cycle
            let image = match self.vision.capture() {
                Ok(img) => img,
                Err(e) => {
                    let msg = format!("capture failed: {e}");
                    warn!(task = %self.task_name, node = %current_node_id, path = %path, error = %e, "✗ 截图失败");
                    self.emit_log("error", format!(
                        "✗ 截图失败 | 当前节点: [{current_node_id}] | 调用链: {path} | 错误: {e}"
                    ));
                    self.status = ExecutorStatus::Failed(msg);
                    return self.status.clone();
                }
            };

            // Try to match a next item starting from next_pointer
            let pointer = self.call_stack[stack_len - 1].next_pointer;

            // All next items exhausted — node is complete
            if pointer >= next_items.len() {
                self.complete_node(&current_node_id);
                continue;
            }

            let total_candidates = next_items.len();
            debug!(
                task = %self.task_name,
                node = %current_node_id,
                scan = format!("{}/{total_candidates}", pointer + 1),
                "→ 开始扫描后继节点"
            );
            self.emit_log("info", format!(
                "→ 扫描 [{current_node_id}] 后继节点: 从 {}/{total_candidates} 开始",
                pointer + 1
            ));

            let mut matched_idx: Option<usize> = None;
            let mut match_pos = None;
            let mut all_dead = true;

            // Timeout tracking is per-frame: timestamps persist across retry cycles
            let frame_idx = stack_len - 1;

            #[allow(clippy::needless_range_loop)]
            'scan: for idx in pointer..next_items.len() {
                let target_id = next_items[idx].node_id().to_string();
                let scan_pos = format!("{}/{}", idx + 1, total_candidates);

                let tried_at = *self.call_stack[frame_idx]
                    .first_tried
                    .entry(target_id.clone())
                    .or_insert_with(Instant::now);

                if let Some(reason) = self.is_skippable(&target_id, Some(tried_at)) {
                    let hit = self.hit_count(&target_id);
                    let event = EngineEvent::NodeSkipped {
                        task: self.task_name.clone(),
                        node: target_id.clone(),
                        reason: reason.to_string(),
                    };
                    debug!(task = %self.task_name, from = %current_node_id, to = %target_id, scan = %scan_pos, reason, "⊘ 跳过");
                    self.emit_log("info", format!(
                        "⊘ 跳过 [{target_id}] ({scan_pos}) | 原因: {reason}"
                    ));
                    self.emit(event);
                    self.record_trace(&current_node_id, &target_id, None, hit);
                    // "node not found" is permanent; timeout/max_hit/disabled also permanent
                    // → this candidate will never match
                    continue;
                }

                let target_node = match self.task.nodes.get(&target_id) {
                    Some(n) => n,
                    None if target_id == "end-task" => {
                        debug!(task = %self.task_name, from = %current_node_id, "→ end-task ({scan_pos})");
                        matched_idx = Some(idx);
                        break 'scan;
                    }
                    None => continue,
                };

                if let Some(cond) = target_node.recognize.as_ref() {
                    debug!(task = %self.task_name, from = %current_node_id, to = %target_id, scan = %scan_pos, "? 评估识别条件");
                    all_dead = false; // this candidate could match on a future retry
                    match evaluate(cond, &image, self.vision.as_ref(), &self.task.nodes) {
                        Ok(result) => {
                            self.record_trace(&current_node_id, &target_id, Some(result.matched), self.hit_count(&target_id));
                            if result.matched {
                                info!(task = %self.task_name, from = %current_node_id, to = %target_id, scan = %scan_pos, "✓ 匹配");
                                self.emit_log("info", format!(
                                    "✓ 匹配 [{target_id}] ({scan_pos}) | 从: [{current_node_id}]"
                                ));
                                match_pos = result.position;
                                matched_idx = Some(idx);
                                break 'scan;
                            } else {
                                debug!(task = %self.task_name, from = %current_node_id, to = %target_id, scan = %scan_pos, "✗ 未匹配");
                                self.emit_log("info", format!(
                                    "✗ 未匹配 [{target_id}] ({scan_pos})"
                                ));
                                break 'scan; // don't check later candidates — retry this one first
                            }
                        }
                        Err(e) => {
                            all_dead = false; // transient error — may succeed on retry
                            warn!(task = %self.task_name, from = %current_node_id, to = %target_id, scan = %scan_pos, error = %e, "⚠ 识别出错");
                            self.emit_log("warn", format!(
                                "⚠ 识别出错 [{current_node_id}] → [{target_id}] ({scan_pos}) | 错误: {e}"
                            ));
                            break 'scan; // don't check later candidates — retry this one first
                        }
                    }
                } else {
                    // No recognize condition — unconditional match
                    all_dead = false;
                    debug!(task = %self.task_name, from = %current_node_id, to = %target_id, scan = %scan_pos, "→ 无条件跳转");
                    self.emit_log("info", format!(
                        "→ 无条件跳转 [{current_node_id}] → [{target_id}] ({scan_pos})"
                    ));
                    self.record_trace(&current_node_id, &target_id, None, self.hit_count(&target_id));
                    matched_idx = Some(idx);
                    break 'scan;
                }
            }

            if let Some(idx) = matched_idx {
                let item = next_items[idx].clone();
                let target_id = item.node_id().to_string();
                let is_jumpback = matches!(item, NextItem::GotoWithJumpback(_));

                let transition = if is_jumpback { "⤴ jumpback" } else { "→ 进入" };
                self.emit(EngineEvent::NodeEntered {
                    task: self.task_name.clone(),
                    node: target_id.clone(),
                });

                // Execute the target node's action before pushing
                if let Some(target_node) = self.task.nodes.get(&target_id).cloned() {
                    if let Some(pre_wait) = &target_node.pre_wait {
                        debug!(task = %self.task_name, from = %current_node_id, to = %target_id, freeze_ms = pre_wait.freeze, "⏸ pre_wait");
                        tokio::time::sleep(Duration::from_millis(pre_wait.freeze)).await;
                    }
                    if let Some(act) = &target_node.action.clone() {
                        info!(task = %self.task_name, from = %current_node_id, to = %target_id, action = ?act, "{transition}");
                        self.emit_log("info", format!(
                            "{transition} [{target_id}] | 动作: {act:?} | 调用链: {path} > {target_id}"
                        ));
                        if let Err(e) = action::execute(act, match_pos.as_ref(), self.input.as_ref()) {
                            warn!(task = %self.task_name, from = %current_node_id, to = %target_id, error = %e, "⚠ 动作失败");
                            self.emit_log("warn", format!(
                                "⚠ 动作失败 [{current_node_id}] → [{target_id}] | 动作: {act:?} | 错误: {e}"
                            ));
                        }
                    }
                    if let Some(pw) = target_node.post_wait {
                        debug!(task = %self.task_name, to = %target_id, post_wait_ms = pw, "⏸ post_wait");
                        tokio::time::sleep(Duration::from_millis(pw)).await;
                    }
                }

                self.push_frame(&target_id, is_jumpback);
            } else if all_dead {
                // All candidates permanently dead (all skipped): no point retrying
                let msg = format!(
                    "所有候选节点均已永久跳过（超时/禁用/max_hit），无法继续 | 当前节点: [{current_node_id}] | 候选: {}",
                    next_items.iter().map(|n| n.node_id().to_string()).collect::<Vec<_>>().join(", ")
                );
                warn!(task = %self.task_name, node = %current_node_id, path = %path, "✗ 所有候选已耗尽");
                self.emit_log("error", msg.clone());
                self.status = ExecutorStatus::Failed(msg);
                return self.status.clone();
            } else {
                // No match but some candidates are still viable: retry after interval
                debug!(
                    task = %self.task_name,
                    node = %current_node_id,
                    candidates = total_candidates,
                    retry_ms = self.config.retry_interval_ms,
                    "⟳ 无匹配，等待重试"
                );
                self.emit_log("info", format!(
                    "⟳ 无匹配 [{current_node_id}] | 已扫描 {total_candidates} 个候选 | {}ms 后重试",
                    self.config.retry_interval_ms
                ));
                tokio::time::sleep(Duration::from_millis(self.config.retry_interval_ms)).await;
            }
        }
    }

    fn push_frame(&mut self, node_id: &str, via_jumpback: bool) {
        self.call_stack.push(Frame {
            node_id: node_id.to_string(),
            next_pointer: 0,
            via_jumpback,
            first_tried: HashMap::new(),
        });
        let depth = self.call_stack.len();
        debug!("push frame: {node_id} depth={depth}");
    }

    fn complete_node(&mut self, node_id: &str) {
        let via_jumpback = self.call_stack.last().map(|f| f.via_jumpback).unwrap_or(false);
        let count = self.hit_counts.entry(node_id.to_string()).or_insert(0);
        *count += 1;
        let new_hit_count = *count;
        self.call_stack.pop();

        let path = self.stack_path();
        let jb = if via_jumpback { " [jumpback]" } else { "" };
        info!(task = %self.task_name, node = %node_id, hit_count = new_hit_count, depth = self.call_stack.len(), "● 节点完成");
        self.emit_log("info", format!(
            "● 节点完成 [{node_id}]{jb} (第{new_hit_count}次) | 返回: [{path}]"
        ));

        self.emit(EngineEvent::NodeCompleted {
            task: self.task_name.clone(),
            node: node_id.to_string(),
            hit_count: new_hit_count,
        });

        if let Some(parent) = self.call_stack.last_mut() {
            if via_jumpback {
                parent.next_pointer = 0;
                parent.first_tried.clear(); // reset timeout timers on jumpback
            } else {
                parent.next_pointer += 1;
            }
        }
    }
}
