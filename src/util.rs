use swayipc::{Node, NodeType};

pub trait NodeExt {
    fn windows(&self) -> NodeIter<'_>;

    fn is_window(&self) -> bool;

    fn matches(
        &self,
        name: &Option<String>,
        app_id: &Option<String>,
        class: &Option<String>,
    ) -> bool;
}

impl NodeExt for Node {
    fn windows(&self) -> NodeIter<'_> {
        NodeIter {
            filter: Self::is_window,
            stack: vec![self],
        }
    }

    fn is_window(&self) -> bool {
        matches!(self.node_type, NodeType::Con | NodeType::FloatingCon) && self.pid.is_some()
    }
    fn matches(
        &self,
        name: &Option<String>,
        app_id: &Option<String>,
        class: &Option<String>,
    ) -> bool {
        dbg!(
            (name.is_none() || &self.name == name)
            && (app_id.is_none()
                || dbg!(&self.app_id) == dbg!(app_id)
                || class.is_some()
                && self
                .window_properties
                .as_ref()
                .map(|props| &props.class == class)
                == Some(true))
            && (class.is_none()
                || self
                .window_properties
                .as_ref()
                .map(|props| &props.class == class)
                == Some(true))
        )
    }
}

pub struct NodeIter<'a> {
    filter: fn(node: &Node) -> bool,
    stack: Vec<&'a Node>,
}

impl<'a> Iterator for NodeIter<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().and_then(|node| {
            self.stack
                .extend(node.nodes.iter().chain(node.floating_nodes.iter()));
            (self.filter)(node).then_some(node).or_else(|| self.next())
        })
    }
}
