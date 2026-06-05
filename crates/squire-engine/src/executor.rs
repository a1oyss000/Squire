use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use squire_input::InputBackend;
use tokio::sync::watch;
use tracing::{debug, warn};

use crate::recognize::{evaluate, VisionProvider};
use crate::action;
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
    entered_at: Instant,
    /// When this frame was pushed, was it via a jumpback transition?
    via_jumpback: bool,
}

// ---- Executor ---------------------------------------------------------------

pub struct Executor {
    task: LoadedTask,
    call_stack: Vec<Frame>,
    hit_counts: HashMap<String, u32>,
    trace_log: TraceLog,
    config: ExecutorConfig,
    vision: Arc<dyn VisionProvider>,
    input: Arc<dyn InputBackend>,
    cancel: watch::Receiver<bool>,
    pub status: ExecutorStatus,
}

impl Executor {
    pub fn new(
        task: LoadedTask,
        vision: Arc<dyn VisionProvider>,
        input: Arc<dyn InputBackend>,
        cancel: watch::Receiver<bool>,
        config: ExecutorConfig,
    ) -> Self {
        Executor {
            task,
            call_stack: Vec::new(),
            hit_counts: HashMap::new(),
            trace_log: TraceLog::new(),
            config,
            vision,
            input,
            cancel,
            status: ExecutorStatus::Running,
        }
    }

    fn hit_count(&self, node_id: &str) -> u32 {
        *self.hit_counts.get(node_id).unwrap_or(&0)
    }

    fn is_skippable(&self, node_id: &str, first_tried_at: Option<Instant>) -> bool {
        if node_id == "end-task" {
            return false;
        }
        if self.task.disabled_nodes.contains(node_id) {
            return true;
        }
        let node = match self.task.nodes.get(node_id) {
            Some(n) => n,
            None => return true,
        };
        if let Some(max) = node.max_hit {
            if self.hit_count(node_id) >= max {
                return true;
            }
        }
        if let (Some(timeout_ms), Some(tried_at)) = (node.timeout, first_tried_at) {
            if tried_at.elapsed() >= Duration::from_millis(timeout_ms) {
                return true;
            }
        }
        false
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

    pub fn dump_trace(&self) -> String {
        self.trace_log.dump_jsonl()
    }

    /// Main execution loop. Runs until Success, Failed, or Cancelled.
    pub async fn run(&mut self) -> ExecutorStatus {
        // Push the entry frame
        let entry = self.task.entry.clone();
        self.push_frame(&entry, false);

        loop {
            // Check cancellation
            if *self.cancel.borrow() {
                self.status = ExecutorStatus::Cancelled;
                return self.status.clone();
            }

            // Stack empty → success
            let stack_len = self.call_stack.len();
            if stack_len == 0 {
                self.status = ExecutorStatus::Success;
                return self.status.clone();
            }

            let current_node_id = self.call_stack[stack_len - 1].node_id.clone();

            // "end-task" pseudo-node
            if current_node_id == "end-task" {
                self.call_stack.pop();
                self.status = ExecutorStatus::Success;
                return self.status.clone();
            }

            let node = match self.task.nodes.get(&current_node_id).cloned() {
                Some(n) => n,
                None => {
                    warn!("Node not found: {}", current_node_id);
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
                    self.status = ExecutorStatus::Failed(format!("capture failed: {e}"));
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

            let mut matched_idx: Option<usize> = None;
            let mut match_pos = None;

            // Track first-tried timestamps per candidate node for timeout
            let mut first_tried: HashMap<String, Instant> = HashMap::new();

            'scan: for idx in pointer..next_items.len() {
                let target_id = next_items[idx].node_id();

                let tried_at = *first_tried.entry(target_id.to_string()).or_insert_with(Instant::now);

                if self.is_skippable(target_id, Some(tried_at)) {
                    continue;
                }

                let target_node = match self.task.nodes.get(target_id) {
                    Some(n) => n,
                    None if target_id == "end-task" => {
                        matched_idx = Some(idx);
                        break 'scan;
                    }
                    None => continue,
                };

                if let Some(cond) = &target_node.recognize.clone() {
                    match evaluate(cond, &image, self.vision.as_ref(), &self.task.nodes) {
                        Ok(result) => {
                            self.record_trace(&current_node_id, target_id, Some(result.matched), self.hit_count(target_id));
                            if result.matched {
                                match_pos = result.position;
                                matched_idx = Some(idx);
                                break 'scan;
                            }
                        }
                        Err(e) => {
                            warn!("recognize error on {}: {e}", target_id);
                        }
                    }
                } else {
                    // No recognize condition — unconditional match
                    self.record_trace(&current_node_id, target_id, None, self.hit_count(target_id));
                    matched_idx = Some(idx);
                    break 'scan;
                }
            }

            if let Some(idx) = matched_idx {
                let item = next_items[idx].clone();
                let target_id = item.node_id().to_string();
                let is_jumpback = matches!(item, NextItem::GotoWithJumpback(_));

                // Execute the target node's action before pushing
                if let Some(target_node) = self.task.nodes.get(&target_id).cloned() {
                    if let Some(pre_wait) = &target_node.pre_wait {
                        tokio::time::sleep(Duration::from_millis(pre_wait.freeze)).await;
                    }
                    if let Some(act) = &target_node.action.clone() {
                        if let Err(e) = action::execute(act, match_pos.as_ref(), self.input.as_ref()) {
                            warn!("action error on {}: {e}", target_id);
                        }
                    }
                    if let Some(pw) = target_node.post_wait {
                        tokio::time::sleep(Duration::from_millis(pw)).await;
                    }
                }

                self.push_frame(&target_id, is_jumpback);
            } else {
                // No match: interval retry
                tokio::time::sleep(Duration::from_millis(self.config.retry_interval_ms)).await;
            }
        }
    }

    fn push_frame(&mut self, node_id: &str, via_jumpback: bool) {
        self.call_stack.push(Frame {
            node_id: node_id.to_string(),
            next_pointer: 0,
            entered_at: Instant::now(),
            via_jumpback,
        });
        debug!("push frame: {node_id} depth={}", self.call_stack.len());
    }

    fn complete_node(&mut self, node_id: &str) {
        let via_jumpback = self.call_stack.last().map(|f| f.via_jumpback).unwrap_or(false);
        *self.hit_counts.entry(node_id.to_string()).or_insert(0) += 1;
        self.call_stack.pop();
        debug!("pop frame: {node_id} hit_count={}", self.hit_count(node_id));

        if let Some(parent) = self.call_stack.last_mut() {
            if via_jumpback {
                parent.next_pointer = 0;
            } else {
                parent.next_pointer += 1;
            }
        }
    }
}
