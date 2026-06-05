use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use squire_engine::executor::{Executor, ExecutorConfig, ExecutorStatus, LoadedTask};
use squire_engine::recognize::VisionProvider;
use squire_engine::schema::flow::{NextItem, NodeDef};
use squire_error::Result;
use squire_input::{InputBackend, Point};
use squire_vision::capture::Image;
use tokio::sync::watch;

// ---- MockVisionProvider -------------------------------------------------------

struct MockVisionProvider {
    matches: HashMap<String, bool>,
}

impl MockVisionProvider {
    fn new(matches: HashMap<String, bool>) -> Self {
        MockVisionProvider { matches }
    }

    fn always_match() -> Self {
        MockVisionProvider { matches: HashMap::new() }
    }
}

impl VisionProvider for MockVisionProvider {
    fn capture(&self) -> Result<Image> {
        Ok(Image {
            width: 1,
            height: 1,
            data: Arc::new(vec![0u8; 4]),
        })
    }

    fn match_template(
        &self,
        _image: &Image,
        template_path: &str,
        _roi: Option<[i32; 4]>,
        _threshold: f64,
    ) -> Result<Option<squire_engine::recognize::MatchPosition>> {
        let matched = self.matches.get(template_path).copied().unwrap_or(false);
        if matched {
            Ok(Some(squire_engine::recognize::MatchPosition { x: 0, y: 0 }))
        } else {
            Ok(None)
        }
    }

    fn find_text(
        &self,
        _image: &Image,
        _pattern: &str,
        _roi: Option<[i32; 4]>,
    ) -> Result<Option<squire_engine::recognize::TextPosition>> {
        Ok(None)
    }

    fn check_color(
        &self,
        _image: &Image,
        _roi: [i32; 4],
        _lower: [u8; 3],
        _upper: [u8; 3],
        _count: u32,
    ) -> Result<bool> {
        Ok(false)
    }
}

// ---- MockInputBackend --------------------------------------------------------

struct MockInputBackend {
    clicks: Mutex<Vec<Point>>,
}

impl MockInputBackend {
    fn new() -> Self {
        MockInputBackend { clicks: Mutex::new(Vec::new()) }
    }
}

impl InputBackend for MockInputBackend {
    fn click(&self, point: Point) -> Result<()> {
        self.clicks.lock().unwrap().push(point);
        Ok(())
    }
    fn double_click(&self, _point: Point) -> Result<()> { Ok(()) }
    fn drag(&self, _from: Point, _to: Point) -> Result<()> { Ok(()) }
    fn key_press(&self, _key: u16) -> Result<()> { Ok(()) }
}

// ---- helpers -----------------------------------------------------------------

fn simple_node(next: Vec<NextItem>) -> NodeDef {
    NodeDef {
        recognize: None,
        action: None,
        next: if next.is_empty() { None } else { Some(next) },
        max_hit: None,
        timeout: None,
        pre_wait: None,
        post_wait: None,
    }
}

fn make_task(name: &str, entry: &str, nodes: HashMap<String, NodeDef>) -> LoadedTask {
    LoadedTask {
        name: name.to_string(),
        label: name.to_string(),
        entry: entry.to_string(),
        nodes,
        disabled_nodes: HashSet::new(),
    }
}

fn make_task_with_disabled(
    name: &str,
    entry: &str,
    nodes: HashMap<String, NodeDef>,
    disabled: HashSet<String>,
) -> LoadedTask {
    LoadedTask {
        name: name.to_string(),
        label: name.to_string(),
        entry: entry.to_string(),
        nodes,
        disabled_nodes: disabled,
    }
}

fn default_executor(task: LoadedTask) -> (Executor, watch::Sender<bool>) {
    let (cancel_tx, cancel_rx) = watch::channel(false);
    let vision = Arc::new(MockVisionProvider::always_match());
    let input = Arc::new(MockInputBackend::new());
    let exec = Executor::new(task, vision, input, cancel_rx, ExecutorConfig::default());
    (exec, cancel_tx)
}

// ---- tests -------------------------------------------------------------------

#[tokio::test]
async fn test_linear_flow() {
    // A → B → C (no recognize, unconditional), C has no next (leaf)
    let mut nodes = HashMap::new();
    nodes.insert("A".into(), simple_node(vec![NextItem::Goto("B".into())]));
    nodes.insert("B".into(), simple_node(vec![NextItem::Goto("C".into())]));
    nodes.insert("C".into(), simple_node(vec![]));

    let task = make_task("test", "A", nodes);
    let (mut exec, _cancel) = default_executor(task);
    let status = exec.run().await;
    assert_eq!(status, ExecutorStatus::Success);

    let trace = exec.dump_trace();
    let lines: Vec<&str> = trace.lines().collect();
    assert!(lines.iter().any(|l| l.contains("\"from\":\"A\"") && l.contains("\"to\":\"B\"")));
    assert!(lines.iter().any(|l| l.contains("\"from\":\"B\"") && l.contains("\"to\":\"C\"")));
}

#[tokio::test]
async fn test_jumpback() {
    // parent → child [jumpback]; child has max_hit:2
    // child should be entered twice then parent moves on to end-task
    let mut nodes = HashMap::new();
    let mut child = simple_node(vec![]);
    child.max_hit = Some(2);
    nodes.insert("parent".into(), simple_node(vec![
        NextItem::GotoWithJumpback("child".into()),
        NextItem::Goto("end-task".into()),
    ]));
    nodes.insert("child".into(), child);

    let task = make_task("test", "parent", nodes);
    let (mut exec, _cancel) = default_executor(task);
    let status = exec.run().await;
    assert_eq!(status, ExecutorStatus::Success);
}

#[tokio::test]
async fn test_max_hit_skip() {
    // node with max_hit:1 is skipped on second encounter
    let mut nodes = HashMap::new();
    let mut limited = simple_node(vec![]);
    limited.max_hit = Some(1);
    nodes.insert("start".into(), simple_node(vec![
        NextItem::Goto("limited".into()),
        NextItem::Goto("end-task".into()),
    ]));
    nodes.insert("limited".into(), limited);

    let task = make_task("test", "start", nodes);
    let (mut exec, _cancel) = default_executor(task);
    let status = exec.run().await;
    assert_eq!(status, ExecutorStatus::Success);
}

#[tokio::test]
async fn test_disabled_node_skip() {
    let mut nodes = HashMap::new();
    nodes.insert("start".into(), simple_node(vec![
        NextItem::Goto("disabled".into()),
        NextItem::Goto("end-task".into()),
    ]));
    nodes.insert("disabled".into(), simple_node(vec![]));

    let mut disabled = HashSet::new();
    disabled.insert("disabled".into());
    let task = make_task_with_disabled("test", "start", nodes, disabled);
    let (mut exec, _cancel) = default_executor(task);
    let status = exec.run().await;
    assert_eq!(status, ExecutorStatus::Success);
}

#[tokio::test]
async fn test_end_task() {
    let mut nodes = HashMap::new();
    nodes.insert("start".into(), simple_node(vec![NextItem::Goto("end-task".into())]));

    let task = make_task("test", "start", nodes);
    let (mut exec, _cancel) = default_executor(task);
    let status = exec.run().await;
    assert_eq!(status, ExecutorStatus::Success);
}

#[tokio::test]
async fn test_cancel() {
    // "blocker" has a recognize condition that never matches (template not in mock map)
    // "start" loops back to blocker — executor will retry forever until cancelled
    let mut nodes = HashMap::new();
    use squire_engine::schema::flow::RecognizeCondition;
    use squire_engine::schema::flow::TemplateCondition;
    let blocker = NodeDef {
        recognize: Some(RecognizeCondition::Template(TemplateCondition {
            template: "never_match.png".into(),
            roi: None,
            order_by: None,
            index: None,
        })),
        action: None,
        next: None,
        max_hit: None,
        timeout: None,
        pre_wait: None,
        post_wait: None,
    };
    // start's only next target is "blocker" which never matches → executor retries forever
    nodes.insert("start".into(), simple_node(vec![NextItem::Goto("blocker".into())]));
    nodes.insert("blocker".into(), blocker);

    let task = make_task("test", "start", nodes);
    let (cancel_tx, cancel_rx) = watch::channel(false);
    // Use a vision provider where "never_match.png" returns false
    let vision = Arc::new(MockVisionProvider::new(HashMap::new()));
    let input = Arc::new(MockInputBackend::new());
    let mut exec = Executor::new(task, vision, input, cancel_rx, ExecutorConfig { retry_interval_ms: 10 });

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let _ = cancel_tx.send(true);
    });

    let status = exec.run().await;
    assert_eq!(status, ExecutorStatus::Cancelled);
}
