use anyhow::{Context, Result};
use derive_more::From;
use serde::Deserialize;
use swayipc::{Connection, NodeLayout, NodeType};

use crate::util::NodeExt;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct LayoutFile {
    pub layouts: Vec<Layout>,
}

#[derive(Debug, Deserialize)]
pub struct Layout {
    pub output: Option<String>,
    pub workspace: Option<WorkspaceIdent>,
    pub content: Node,
}

impl Layout {
    pub fn is_correct(
        &self,
        con: &mut Connection,
        workspace: Option<&WorkspaceIdent>,
    ) -> Result<bool> {
        let tree = con.get_tree()?;
        let mut windows = tree.windows().collect();
        let Some(expected_layout) = self.content.clone().reduce(&mut windows) else {return Ok(true);};
        let workspace = if let Some(workspace) = workspace.or(self.workspace.as_ref()) {
            let Some(workspace) = ( match workspace {
                WorkspaceIdent::Name(name) => tree.find_as_ref(|node| {
                    node.node_type == NodeType::Workspace && node.name.as_ref() == Some(name)
                }),
                &WorkspaceIdent::Number(num) => tree.find_as_ref(|node| {
                    node.node_type == NodeType::Workspace && node.num == Some(num)
                }),
            } ) else {return Ok(windows.is_empty());};
            workspace
        } else {
            tree.find_focused_as_ref(|node| node.node_type == NodeType::Workspace)
                .or_else(|| tree.find_as_ref(|node| node.node_type == NodeType::Workspace))
                .context("there is no workspace")?
        };

        Ok(expected_layout.matches(workspace))
    }
}

#[derive(Debug, Deserialize, From, Clone)]
#[serde(untagged)]
pub enum WorkspaceIdent {
    Name(String),
    Number(i32),
}

#[derive(Debug, Deserialize, Clone)]
pub enum Node {
    Window {
        name: Option<String>,
        app_id: Option<String>,
        class: Option<String>,
    },
    Container {
        layout: ContainerLayout,
        content: Vec<Node>,
    },
}

impl Node {
    fn reduce(self, windows: &mut Vec<&swayipc::Node>) -> Option<Self> {
        match self {
            // Would be nicer with drain_filter
            Node::Window {
                app_id,
                name,
                class,
            } => windows
                .iter()
                .position(|window| window.matches(&name, &app_id, &class))
                .map(|idx| {
                    windows.remove(idx);
                    Self::Window {
                        app_id,
                        name,
                        class,
                    }
                }),
            Node::Container { content, layout } => {
                let mut content: Vec<_> = content
                    .into_iter()
                    .filter_map(|node| node.reduce(windows))
                    .collect();

                match content.len() {
                    0 => None,
                    1 => Some(content.swap_remove(0)),
                    _ => Some(Self::Container { layout, content }),
                }
            }
        }
    }

    fn matches(&self, node: &swayipc::Node) -> bool {
        if node.nodes.len() == 1 {
            self.matches(&node.nodes[0])
        } else {
            match self {
                Node::Window {
                    name,
                    app_id,
                    class,
                } => node.matches(name, app_id, class),
                Node::Container { layout, content } => {
                    !node.is_window()
                        && layout == &node.layout
                        && content.len() == node.nodes.len()
                        && content
                            .iter()
                            .zip(node.nodes.iter())
                            .all(|(this, node)| this.matches(node))
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ContainerLayout {
    SplitH,
    SplitV,
    Stacked,
    Tabbed,
}

impl PartialEq<NodeLayout> for ContainerLayout {
    fn eq(&self, other: &NodeLayout) -> bool {
        matches!(
            (self, other),
            (ContainerLayout::SplitH, NodeLayout::SplitH)
                | (ContainerLayout::SplitV, NodeLayout::SplitV)
                | (ContainerLayout::Stacked, NodeLayout::Stacked)
                | (ContainerLayout::Tabbed, NodeLayout::Tabbed)
        )
    }
}
