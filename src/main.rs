use anyhow::{bail, ensure, Context, Result};
use layout::{Layout, WorkspaceIdent};
use swayipc::*;

mod cli;
mod layout;
mod util;

macro_rules! cmd {
    ($con:expr,$($format:tt)*) => {
        $con.run_command(format!($($format)*))?
    };
}

fn main() -> Result<()> {
    let layout = Layout {
        output: None,
        workspace: Some(WorkspaceIdent::from(4)),
        content: layout::Node::Container {
            layout: layout::ContainerLayout::SplitH,
            content: vec![
                layout::Node::Window {
                    name: None,
                    app_id: Some("org.wezfurlong.wezterm".to_owned()),
                    class: None,
                },
                layout::Node::Window {
                    name: None,
                    app_id: Some("org.wezfurlong.wezterm".to_owned()),
                    class: None,
                },
            ],
        },
    };

    let mut con = Connection::new()?;

    dbg!(layout.is_correct(&mut con, None)?);

    // clean_up_5(&mut con)?;

    Ok(())
}

fn chat_layout_correct(workspace: &Node) -> bool {
    workspace.nodes.len() == 2
        && workspace.layout == NodeLayout::SplitH
        && workspace.nodes.len() == 2
        && workspace
            .nodes
            .iter()
            .all(|node| node.layout == NodeLayout::SplitV)
        && workspace.nodes.iter().any(|node| node.nodes.len() == 2)
}

fn find_all(node: &Node, condition: fn(&Node) -> bool) -> Vec<&Node> {
    node.nodes
        .iter()
        .flat_map(move |node| find_all(node, condition))
        .chain(condition(node).then_some(node))
        .collect()
}

fn clean_up_5(con: &mut Connection) -> Result<()> {
    let root = con.get_tree()?;
    let focused = root
        .find_focused_as_ref(|node| node.focused)
        .map(|node| node.id);
    let chat_workspace = root
        .find(|node| node.node_type == NodeType::Workspace && node.num == Some(5))
        .context("Did not find chat workspace")?;
    if chat_workspace.nodes.is_empty() {
        bail!("Empty chat workspace")
    }
    if chat_layout_correct(&chat_workspace) {
        return Ok(());
    }

    let mut windows = find_all(&chat_workspace, |node| node.pid.is_some());
    ensure!(windows.len() > 3, "requires 3 windows to make sense");

    for window in &windows {
        cmd!(con, "[con_id={}] move to scratchpad", window.id);
    }

    windows.reverse();

    let window = windows.pop().unwrap().id;
    cmd!(
        con,
        "[con_id={window}] move to workspace 5, floating disable, split v"
    );
    let window = windows.pop().unwrap().id;
    cmd!(
        con,
        "[con_id={window}] move to workspace 5, floating disable"
    );
    let window = windows.pop().unwrap().id;
    cmd!(
        con,
        "[con_id={window}] move to workspace 5, floating disable, move left, split v"
    );

    for window in &windows {
        cmd!(
            con,
            "[con_id={}] move to workspace 5, floating disable",
            window.id
        );
    }
    //
    // dbg!(chat_workspace.id);
    // // for node in &chat_workspace.nodes {
    // //     dbg!(&node.name);
    // //     dbg!(node.node_type);
    // //     dbg!(node.id);
    // // }
    // dbg!(chat_workspace.nodes.len());
    // if chat_workspace.nodes.len() != 1 {
    //     {
    //         let first = &chat_workspace.nodes[0];
    //         let id = chat_workspace.nodes[0].id;
    //         if first.pid.is_some() {
    //             cmd!(con, "[con_id={id}] split v");
    //             cmd!(con, "[con_id={id}] focus parent");
    //         } else {
    //             cmd!(con, "[con_id={id}] focus");
    //         }
    //         cmd!(con, "mark chat_parent");
    //     }
    //     for node in get_all_recursively(chat_workspace)
    //         .iter()
    //         .map(|node| node.id)
    //     {
    //         cmd!(con, "[con_id={node}] move to mark chat_parent");
    //     }
    // }
    //
    // // ensure!(
    // //     chat_workspace.nodes.len() == 1,
    // //     "there should be one node per workspace"
    // // );
    // // chat_workspace.nodes.first().unwrap();
    //
    // // wh
    // //     .into_iter()
    // //     .find(|ws| ws.num == 5)
    // //     .context("now chat workspace")?;
    // // dbg!(&tree);
    //
    // // if chat_workspace.layout != "splitv" {
    // //     // con.run_command("q")
    // // }
    //

    if let Some(focused) = focused {
        con.run_command(format!("[con_id={focused}] focus"))?;
    }
    Ok(())
}
