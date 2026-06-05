use std::collections::HashMap;
use std::path::PathBuf;

use squire_engine::loader::load_task;

fn benchmark_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs/benchmark/draft-new-format/final")
}

fn shared_dir() -> PathBuf {
    benchmark_dir()
}

#[test]
fn test_load_small_event() {
    let dir = benchmark_dir();
    let task_path = dir.join("small-event.task.yaml");
    let flow_path = dir.join("small-event.flow.yaml");
    let sd = shared_dir();

    let result = load_task(&task_path, &flow_path, Some(sd.as_path()), &HashMap::new());
    assert!(result.is_ok(), "load_task failed: {:?}", result.err());

    let loaded = result.unwrap();
    assert_eq!(loaded.name, "small-event");
    assert!(!loaded.entry.is_empty(), "entry should not be empty");
    assert!(!loaded.nodes.is_empty(), "nodes should not be empty");
}

#[test]
fn test_load_large_event() {
    let dir = benchmark_dir();
    let task_path = dir.join("large-event.task.yaml");
    let flow_path = dir.join("large-event.flow.yaml");
    let sd = shared_dir();

    // Provide theme + story_priority so $story_index resolves to a valid integer
    let mut user_options: HashMap<String, serde_yaml::Value> = HashMap::new();
    user_options.insert(
        "theme".to_string(),
        serde_yaml::Value::String("star-anis".to_string()),
    );
    user_options.insert(
        "story_priority".to_string(),
        serde_yaml::Value::String("story2-first".to_string()),
    );

    let result = load_task(&task_path, &flow_path, Some(sd.as_path()), &user_options);
    assert!(result.is_ok(), "load_task failed: {:?}", result.err());

    let loaded = result.unwrap();
    assert_eq!(loaded.name, "large-event");
    assert!(!loaded.nodes.is_empty(), "nodes should not be empty");
}

#[test]
fn test_bindings_disable() {
    let dir = benchmark_dir();
    let task_path = dir.join("small-event.task.yaml");
    let flow_path = dir.join("small-event.flow.yaml");
    let sd = shared_dir();

    // Provide options that only enable "challenge" — story and mission should be disabled
    let mut user_options: HashMap<String, serde_yaml::Value> = HashMap::new();
    user_options.insert(
        "content".to_string(),
        serde_yaml::Value::Sequence(vec![
            serde_yaml::Value::String("challenge".to_string()),
        ]),
    );

    let result = load_task(&task_path, &flow_path, Some(sd.as_path()), &user_options);
    assert!(result.is_ok(), "load_task failed: {:?}", result.err());

    let loaded = result.unwrap();
    assert!(
        loaded.disabled_nodes.contains("story"),
        "story should be disabled"
    );
    assert!(
        loaded.disabled_nodes.contains("mission"),
        "mission should be disabled"
    );
    assert!(
        !loaded.disabled_nodes.contains("challenge"),
        "challenge should not be disabled"
    );
}
