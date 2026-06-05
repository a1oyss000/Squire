use std::time::Instant;

const TRACE_CAP: usize = 10_000;

#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub timestamp: Instant,
    pub from_node: String,
    pub to_node: String,
    pub condition_result: Option<bool>,
    pub stack_depth: usize,
    pub hit_count_at_transition: u32,
}

pub struct TraceLog {
    entries: Vec<TraceEntry>,
    head: usize,
    full: bool,
}

impl TraceLog {
    pub fn new() -> Self {
        TraceLog {
            entries: Vec::with_capacity(TRACE_CAP),
            head: 0,
            full: false,
        }
    }

    pub fn push(&mut self, entry: TraceEntry) {
        if self.full {
            self.entries[self.head] = entry;
            self.head = (self.head + 1) % TRACE_CAP;
        } else {
            self.entries.push(entry);
            if self.entries.len() == TRACE_CAP {
                self.full = true;
                self.head = 0;
            }
        }
    }

    pub fn dump_jsonl(&self) -> String {
        let ordered = self.iter_ordered();
        let mut out = String::new();
        for e in ordered {
            out.push_str(&format!(
                "{{\"from\":{:?},\"to\":{:?},\"cond\":{:?},\"depth\":{},\"hits\":{}}}\n",
                e.from_node, e.to_node, e.condition_result, e.stack_depth, e.hit_count_at_transition
            ));
        }
        out
    }

    pub fn iter_ordered(&self) -> Vec<&TraceEntry> {
        if !self.full {
            self.entries.iter().collect()
        } else {
            let mut v: Vec<&TraceEntry> = Vec::with_capacity(TRACE_CAP);
            v.extend(self.entries[self.head..].iter());
            v.extend(self.entries[..self.head].iter());
            v
        }
    }
}
