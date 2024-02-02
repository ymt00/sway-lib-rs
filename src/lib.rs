use json::{
    iterators::Members,
    JsonValue::{self, Null},
};
use regex::Regex;
use std::process::{Command, Output, Stdio};
use std::str;

const SWAYMSG_BIN: &str = "/usr/bin/swaymsg";
const BEMENU_BIN: &str = "/usr/bin/bemenu";
const WMENU_BIN: &str = "/usr/bin/wmenu";
const ECHO_BIN: &str = "/usr/bin/echo";

pub struct Node<'a> {
    representation: String,
    app_id: String,
    nodes: Members<'a>,
    floating_nodes: Members<'a>,
}

impl<'a> Node<'a> {
    pub fn new(n: &'a JsonValue) -> Self {
        let representation: String = if n["representation"] != Null {
            Regex::new(r"[H|V|S|T]\[|\]")
                .unwrap()
                .replace_all(n["representation"].as_str().unwrap(), "")
                .replace(' ', "\n")
        } else {
            "".to_string()
        };

        let app_id: String = if n["app_id"] != Null {
            n["app_id"].to_string()
        } else {
            "".to_string()
        };

        Self {
            representation,
            app_id,
            nodes: n["nodes"].members(),
            floating_nodes: n["floating_nodes"].members(),
        }
    }
}

pub fn bemenu(items: &str, args: &[&str]) -> String {
    str::from_utf8(
        &Command::new(BEMENU_BIN)
            .args(args)
            .stdin(Stdio::from(
                Command::new(ECHO_BIN)
                    .args(["-e", items])
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap()
                    .stdout
                    .unwrap(),
            ))
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim_end_matches('\n')
    .to_string()
}

pub fn menu(items: &str, args: &[&str]) -> String {
    str::from_utf8(
        &Command::new(WMENU_BIN)
            .args(args)
            .stdin(Stdio::from(
                Command::new(ECHO_BIN)
                    .args(["-e", items])
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap()
                    .stdout
                    .unwrap(),
            ))
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim_end_matches('\n')
    .to_string()
}

pub fn scratchpad_show(app: &str) {
    Command::new(SWAYMSG_BIN)
        .args([format!("[app_id=\"{app}\"]").as_str(), "scratchpad", "show"])
        .spawn()
        .expect("Command failed to execute");
}

pub fn get_workspaces() -> JsonValue {
    let output: Output = Command::new(SWAYMSG_BIN)
        .args(["-rt", "get_workspaces"])
        .output()
        .expect("Failed to execute command");

    json::parse(&String::from_utf8_lossy(&output.stdout)).unwrap()
}

pub fn get_tree() -> JsonValue {
    let output: Output = Command::new(SWAYMSG_BIN)
        .args(["-rt", "get_tree"])
        .output()
        .expect("Failed to execute command");

    json::parse(&String::from_utf8_lossy(&output.stdout)).unwrap()
}
pub fn get_apps(node: Node) -> String {
    let mut apps: String = String::new();

    if !node.representation.is_empty() {
        apps.push_str(node.representation.as_str());
    }

    recursive_node_apps(node, &mut apps);

    apps.trim_start_matches('\n').to_string()
}

fn recursive_node_apps(node: Node, apps: &mut String) {
    for n in node.nodes.chain(node.floating_nodes) {
        let nn: Node = Node::new(n);

        if !nn.app_id.is_empty() {
            apps.push_str(format!("\n{}", nn.app_id).as_str());
        }

        recursive_node_apps(nn, apps);
    }
}
