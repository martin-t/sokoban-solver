use std::{borrow::Cow, fs, process::Command};

use dot::{self, Edges, GraphWalk, Id, LabelText, Labeller, Nodes, Style};
use fnv::{FnvHashMap, FnvHashSet};

use crate::{
    map::Map,
    solver::a_star::{Cost, SearchNode},
    state::State,
};

type Nd = usize;
type Ed = (usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Type {
    Queued,
    AvoidableDuplicate,
    Duplicate,
    Unique,
}

// TODO merge nodes with the same state? (make sure visited stays correct)
#[derive(Debug, Clone)]
crate struct Graph<'a, C: Cost> {
    map: &'a dyn Map,
    state_to_index: FnvHashMap<&'a State, usize>,
    nodes: Vec<Node<'a, C>>,
    edges: Vec<(usize, usize)>,
    solution_states: Vec<&'a State>,
    time: usize,
}

#[derive(Debug, Clone)]
crate struct Node<'a, C: Cost> {
    state: &'a State,
    queue_nodes: Vec<QueueNode<'a, C>>,
}

#[derive(Debug, Clone)]
crate struct QueueNode<'a, C: Cost> {
    search_node: SearchNode<'a, C>,
    add_time: usize,
    visit_time: Option<usize>,
    visit_type: Type,
}

impl<'a, C: Cost> Graph<'a, C> {
    crate fn new(map: &'a dyn Map) -> Self {
        Self {
            map,
            state_to_index: FnvHashMap::default(),
            nodes: Vec::new(),
            edges: Vec::new(),
            solution_states: Vec::new(),
            time: 0,
        }
    }

    crate fn add(&mut self, search_node: SearchNode<'a, C>, prev: Option<SearchNode<'a, C>>) {
        let index = if self.state_to_index.contains_key(&search_node.state) {
            self.state_to_index[search_node.state]
        } else {
            let index = self.nodes.len();
            self.state_to_index.insert(search_node.state, index);
            self.nodes.push(Node {
                state: search_node.state,
                queue_nodes: Vec::new(),
            });
            index
        };

        let qn = QueueNode {
            search_node,
            add_time: self.time,
            visit_time: None,
            visit_type: Type::Queued,
        };
        self.time += 1;

        self.nodes[index].queue_nodes.push(qn);

        if let Some(prev) = prev {
            let prev_index = self.state_to_index[&prev.state];
            self.edges.push((prev_index, index));
        }
    }

    crate fn mark_duplicate(&mut self, node: SearchNode<'a, C>) {
        let index = self.state_to_index[&node.state];
        for qn in &mut self.nodes[index].queue_nodes {
            qn.visit_time = Some(self.time);
            self.time += 1;
            qn.visit_type = Type::Duplicate;
        }
    }

    crate fn mark_unique(&mut self, node: SearchNode<'a, C>) {
        let index = self.state_to_index[&node.state];
        for qn in &mut self.nodes[index].queue_nodes {
            qn.visit_time = Some(self.time);
            self.time += 1;
            qn.visit_type = Type::Unique;
        }
    }

    crate fn draw_states(&mut self, solution_states: &'a [&'a State]) {
        self.solution_states = solution_states.iter().cloned().collect();

        let mut writer = Vec::new();
        dot::render(self, &mut writer).unwrap();
        let s = String::from_utf8(writer).unwrap();
        // use a monospace font
        let s = s.replace("digraph G {", "digraph G {\n    graph [fontname = \"hack\"];\n    node [fontname = \"hack\"];\n    edge [fontname = \"hack\"];");
        fs::write("state-space.dot", &s).unwrap();

        println!("Generating graph...");
        let status = Command::new("dot")
            // PNG is bigger on disk but takes less memory to load than SVG
            .args(&["-Tpng", "-O", "state-space.dot"])
            .status()
            .unwrap();
        assert!(status.success());
    }
}

impl<'a, C: Cost> GraphWalk<'a, Nd, Ed> for Graph<'a, C> {
    fn nodes(&'a self) -> Nodes<'a, Nd> {
        (0..self.nodes.len()).collect()
    }

    fn edges(&'a self) -> Edges<'a, Ed> {
        Cow::from(&self.edges)
    }

    fn source(&'a self, e: &Ed) -> Nd {
        e.0
    }

    fn target(&'a self, e: &Ed) -> Nd {
        e.1
    }
}

impl<'a, C: Cost> Labeller<'a, Nd, Ed> for Graph<'a, C> {
    fn graph_id(&'a self) -> Id<'a> {
        Id::new("G").unwrap()
    }

    fn node_id(&'a self, n: &Nd) -> Id<'a> {
        Id::new(format!("N{}", n)).unwrap()
    }

    fn node_label(&'a self, &n: &Nd) -> LabelText<'a> {
        let mut s = String::new();
        for qn in &self.nodes[n].queue_nodes {
            let v = qn
                .visit_time
                .map(|t| t.to_string())
                .unwrap_or_else(|| "-".to_owned());
            s.push_str(&format!(
                "c/v: {}/{}\nd: {}, h: {}\ncost: {}\n",
                qn.add_time,
                v,
                qn.search_node.dist,
                qn.search_node.cost - qn.search_node.dist,
                qn.search_node.cost,
            ))
        }
        s.push_str(&self.map.xsb_with_state(self.nodes[n].state).to_string());
        LabelText::EscStr(s.into())
    }

    /*fn node_style(&'a self, n: &Nd) -> Style {
        let node_type = self.nodes[*n].2;
        if node_type == Type::Queued {
            Style::Solid
        } else {
            Style::Filled
        }
    }
    
    fn node_color(&'a self, n: &Nd) -> Option<LabelText<'a>> {
        let state = self.nodes[*n].0.state;
        let node_type = self.nodes[*n].2;
        let color_name = match node_type {
            Type::Unique => {
                if self.solution_states.contains(state) {
                    "red"
                } else {
                    "gold"
                }
            }
            Type::AvoidableDuplicate => "green",
            Type::Duplicate => "gray",
            Type::Queued => return None,
        };
        Some(LabelText::LabelStr(color_name.into()))
    }*/

    fn edge_style(&'a self, e: &Ed) -> Style {
        let state0 = self.nodes[e.0].state;
        let state1 = self.nodes[e.1].state;

        let dest_pos = self.solution_states.iter().position(|&s| s == state1);
        if dest_pos.is_none() || dest_pos.unwrap() == 0 {
            return Style::Solid;
        }
        let dest_pos = dest_pos.unwrap();
        if self.solution_states[dest_pos - 1] == state0 {
            Style::Bold
        } else {
            Style::Solid
        }
    }

    fn edge_color(&'a self, e: &Ed) -> Option<LabelText<'a>> {
        let state0 = self.nodes[e.0].state;
        let state1 = self.nodes[e.1].state;

        let dest_pos = self.solution_states.iter().position(|&s| s == state1);
        if dest_pos.is_none() || dest_pos.unwrap() == 0 {
            return None;
        }
        let dest_pos = dest_pos.unwrap();
        if self.solution_states[dest_pos - 1] == state0 {
            Some(LabelText::LabelStr("red".into()))
        } else {
            None
        }
    }
}
