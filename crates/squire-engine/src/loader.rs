use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::{Context, Result};
use serde_yaml::Value;

use crate::schema::{
    FlowDocument, IncludeRef, NodeDef, OptionDef, RecognizeCondition, TaskDocument,
};
use crate::schema::task::ResourceValue;

// ---- Public output type -------------------------------------------------------

#[derive(Debug, Clone)]
pub struct LoadedTask {
    pub name: String,
    pub label: String,
    pub entry: String,
    pub nodes: HashMap<String, NodeDef>,
    pub disabled_nodes: HashSet<String>,
}

// ---- Entry point --------------------------------------------------------------

/// Load and resolve a task from its YAML files.
///
/// - `task_path`: path to the task.yaml file
/// - `flow_path`: path to the flow.yaml file
/// - `shared_dir`: directory containing shared/*.flow.yaml templates
/// - `user_options`: map of option name → selected value (from UI)
pub fn load_task(
    task_path: &Path,
    flow_path: &Path,
    shared_dir: Option<&Path>,
    user_options: &HashMap<String, Value>,
) -> Result<LoadedTask> {
    let task_doc = parse_task_yaml(task_path)?;
    let flow_doc = parse_flow_yaml(flow_path)?;

    // Merge shared includes into nodes
    let mut nodes = flow_doc.nodes;
    if let Some(includes) = &flow_doc.includes {
        for inc in includes {
            let template_nodes = load_shared_template(inc, shared_dir, flow_path)?;
            for (k, v) in template_nodes {
                nodes.entry(k).or_insert(v);
            }
        }
    }

    // Build variables from user options + resources table
    let variables = resolve_variables(&task_doc, user_options);

    // Replace $variable in all node string fields
    let nodes = substitute_variables_in_nodes(nodes, &variables);

    // Compute disabled_nodes from bindings
    let disabled_nodes = compute_disabled_nodes(&task_doc, user_options);

    Ok(LoadedTask {
        name: task_doc.name,
        label: task_doc.label,
        entry: flow_doc.entry,
        nodes,
        disabled_nodes,
    })
}

// ---- YAML parsers -------------------------------------------------------------

fn parse_task_yaml(path: &Path) -> Result<TaskDocument> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading task file: {}", path.display()))?;
    serde_yaml::from_str(&text)
        .with_context(|| format!("parsing task yaml: {}", path.display()))
}

fn parse_flow_yaml(path: &Path) -> Result<FlowDocument> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading flow file: {}", path.display()))?;
    serde_yaml::from_str(&text)
        .with_context(|| format!("parsing flow yaml: {}", path.display()))
}

// ---- Shared template loading --------------------------------------------------

fn load_shared_template(
    inc: &IncludeRef,
    shared_dir: Option<&Path>,
    flow_path: &Path,
) -> Result<HashMap<String, NodeDef>> {
    // Resolve path: shared_dir / <use_path>.flow.yaml
    // Fall back to flow_path's parent / <use_path>.flow.yaml
    let base = shared_dir
        .map(|d| d.to_path_buf())
        .unwrap_or_else(|| {
            flow_path.parent().unwrap_or(Path::new(".")).to_path_buf()
        });

    let file_name = format!("{}.flow.yaml", inc.use_path);
    let template_path = base.join(&file_name);

    let text = std::fs::read_to_string(&template_path)
        .with_context(|| format!("reading shared template: {}", template_path.display()))?;

    // Templates are minimal flow yamls: just a `nodes:` map (no entry/includes required)
    // Parse as a FlowDocument (entry field may be absent — use a lenient approach)
    let doc: serde_yaml::Mapping = serde_yaml::from_str(&text)
        .with_context(|| format!("parsing shared template: {}", template_path.display()))?;

    let nodes_val = doc.get("nodes").cloned().unwrap_or(Value::Mapping(Default::default()));
    let mut raw_nodes: HashMap<String, NodeDef> = serde_yaml::from_value(nodes_val)
        .with_context(|| format!("parsing nodes in shared template: {}", template_path.display()))?;

    // Apply `with` substitutions to all string fields in the template nodes
    if let Some(params) = &inc.with {
        let str_params: HashMap<String, String> = params
            .iter()
            .map(|(k, v)| (k.clone(), value_to_string(v)))
            .collect();
        raw_nodes = substitute_variables_in_nodes(raw_nodes, &str_params);
    }

    Ok(raw_nodes)
}

// ---- Variable resolution -----------------------------------------------------

/// Build a flat variables map from user option selections + resources table.
/// Multiple resource tables can contribute; later keys don't overwrite earlier ones.
fn resolve_variables(
    task: &TaskDocument,
    user_options: &HashMap<String, Value>,
) -> HashMap<String, String> {
    let mut vars: HashMap<String, String> = HashMap::new();

    let resources = match &task.resources {
        Some(r) => r,
        None => return vars,
    };

    // For each option the user has selected, look up matching resource table
    if let Some(options) = &task.options {
        for (opt_name, opt_def) in options {
            let selected_case = match opt_def {
                OptionDef::Select(_) => {
                    user_options
                        .get(opt_name)
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .or_else(|| {
                            if let OptionDef::Select(s) = opt_def {
                                Some(s.default.clone())
                            } else {
                                None
                            }
                        })
                }
                // checkbox and switch don't drive resource selection directly
                _ => None,
            };

            if let Some(case) = selected_case {
                if let Some(resource_map) = resources.get(&case) {
                    for (var_name, res_val) in resource_map {
                        vars.entry(var_name.clone()).or_insert_with(|| resource_value_to_string(res_val));
                    }
                }
            }
        }
    }

    vars
}

fn resource_value_to_string(rv: &ResourceValue) -> String {
    match rv {
        ResourceValue::Single(s) => s.clone(),
        ResourceValue::List(list) => {
            // Serialize list back to YAML sequence representation for template fields
            // that expect a list (e.g. multi-template matching)
            serde_yaml::to_string(list).unwrap_or_default().trim().to_string()
        }
        ResourceValue::Number(n) => n.to_string(),
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => serde_yaml::to_string(v).unwrap_or_default().trim().to_string(),
    }
}

// ---- $variable substitution in NodeDef fields --------------------------------

fn substitute_variables_in_nodes(
    nodes: HashMap<String, NodeDef>,
    vars: &HashMap<String, String>,
) -> HashMap<String, NodeDef> {
    nodes
        .into_iter()
        .map(|(name, node)| (name, substitute_node(node, vars)))
        .collect()
}

fn substitute_node(mut node: NodeDef, vars: &HashMap<String, String>) -> NodeDef {
    if let Some(rec) = node.recognize {
        node.recognize = Some(substitute_recognize(rec, vars));
    }
    node
}

fn substitute_recognize(rec: RecognizeCondition, vars: &HashMap<String, String>) -> RecognizeCondition {
    match rec {
        RecognizeCondition::Template(mut t) => {
            t.template = substitute_str(&t.template, vars);
            RecognizeCondition::Template(t)
        }
        RecognizeCondition::Ocr(mut o) => {
            o.ocr = substitute_str(&o.ocr, vars);
            RecognizeCondition::Ocr(o)
        }
        other => other,
    }
}

/// Replace all `$varname` occurrences in a string with values from vars.
/// Undefined variables → empty string + warn.
fn substitute_str(s: &str, vars: &HashMap<String, String>) -> String {
    if !s.contains('$') {
        return s.to_string();
    }

    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            // Collect identifier: alphanumeric + underscore
            let mut var_name = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    var_name.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            if var_name.is_empty() {
                result.push('$');
            } else if let Some(val) = vars.get(&var_name) {
                result.push_str(val);
            } else {
                tracing::warn!(variable = %var_name, "undefined $variable in flow — substituting empty string");
            }
        } else {
            result.push(ch);
        }
    }

    result
}

// ---- Bindings → disabled_nodes -----------------------------------------------

fn compute_disabled_nodes(
    task: &TaskDocument,
    user_options: &HashMap<String, Value>,
) -> HashSet<String> {
    let mut disabled = HashSet::new();

    let bindings = match &task.bindings {
        Some(b) => b,
        None => return disabled,
    };
    let options = match &task.options {
        Some(o) => o,
        None => return disabled,
    };

    for binding in bindings {
        let opt_name = &binding.enabled_by;
        let opt_def = match options.get(opt_name) {
            Some(o) => o,
            None => continue,
        };

        match opt_def {
            OptionDef::Checkbox(_) => {
                // User's selected values for this checkbox
                let selected: HashSet<String> = user_options
                    .get(opt_name)
                    .and_then(|v| v.as_sequence())
                    .map(|seq| {
                        seq.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                for node_name in &binding.nodes {
                    if !selected.contains(node_name) {
                        disabled.insert(node_name.clone());
                    }
                }
            }
            OptionDef::Switch(_) => {
                let enabled = user_options
                    .get(opt_name)
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                if !enabled {
                    for node_name in &binding.nodes {
                        disabled.insert(node_name.clone());
                    }
                }
            }
            OptionDef::Select(_) => {
                // Select options don't drive bindings per spec
            }
        }
    }

    disabled
}
