use std::{borrow::Cow, fs, process::Command};

use dot::{Edges, GraphWalk, Id, LabelText, Labeller, Nodes, Style};
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
#[derive(Debug)]
pub(crate) struct Graph<'a, C: Cost> {
    map: &'a dyn Map,
    node_to_index: FnvHashMap<SearchNode<'a, C>, usize>,
    /// node, visited counter, visited type
    nodes: Vec<(SearchNode<'a, C>, usize, Type)>,
    edges: Vec<(usize, usize)>,
    solution_states: FnvHashSet<&'a State>,
    visited_counter: usize,
}

impl<'a, C: Cost> Graph<'a, C> {
    pub(crate) fn new(map: &'a dyn Map) -> Self {
        Self {
            map,
            node_to_index: FnvHashMap::default(),
            nodes: Vec::new(),
            edges: Vec::new(),
            solution_states: FnvHashSet::default(),
            visited_counter: 0,
        }
    }

    pub(crate) fn add(&mut self, node: SearchNode<'a, C>, prev: Option<SearchNode<'a, C>>) {
        assert!(!self.node_to_index.contains_key(&node));

        let node_index = self.nodes.len();

        let mut node_type = Type::Queued;
        for &(search_node, _, _) in &self.nodes {
            if node.state == search_node.state && node.dist >= search_node.dist {
                node_type = Type::AvoidableDuplicate;
                break;
            }
        }

        self.node_to_index.insert(node, node_index);
        self.nodes.push((node, 0, node_type));

        if let Some(prev) = prev {
            let prev_index = self.node_to_index[&prev];
            self.edges.push((prev_index, node_index));
        }
    }

    pub(crate) fn mark_duplicate(&mut self, node: SearchNode<'a, C>) {
        let index = self.node_to_index[&node];
        if self.nodes[index].2 != Type::AvoidableDuplicate {
            self.nodes[index].2 = Type::Duplicate;
        }
        self.nodes[index].1 = self.visited_counter;
        self.visited_counter += 1;
    }

    pub(crate) fn mark_unique(&mut self, node: SearchNode<'a, C>) {
        let index = self.node_to_index[&node];
        self.nodes[index].1 = self.visited_counter;
        self.visited_counter += 1;
        self.nodes[index].2 = Type::Unique;
    }

    pub(crate) fn draw_states(&mut self, solution_states: &'a [&'a State]) {
        self.solution_states = solution_states.iter().copied().collect();

        let mut writer = Vec::new();
        dot::render(self, &mut writer).unwrap();
        let s = String::from_utf8(writer).unwrap();
        let s = s.replace("digraph G {", "digraph G {\n    graph [fontname = \"hack\"];\n    node [fontname = \"hack\"];\n    edge [fontname = \"hack\"];");
        fs::write("state-space.dot", s).unwrap();

        let status = Command::new("dot")
            // PNG is bigger on disk but takes less memory to load than SVG
            .args(["-Tpng", "-O", "state-space.dot"])
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
        Id::new(format!("N{n}")).unwrap()
    }

    fn node_label(&'a self, n: &Nd) -> LabelText<'a> {
        let node = self.nodes[*n].0;
        LabelText::EscStr(
            format!(
                "c/v: {}/{}\nd: {}, h: {}\ncost: {}\n{}",
                n,
                self.nodes[*n].1,
                node.dist,
                node.cost - node.dist,
                node.cost,
                self.map.xsb_with_state(node.state)
            )
            .into(),
        )
    }

    fn node_style(&'a self, n: &Nd) -> Style {
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
    }

    // TODO this also highlights edges to dupes
    fn edge_style(&'a self, e: &Ed) -> Style {
        let state0 = self.nodes[e.0].0.state;
        let state1 = self.nodes[e.1].0.state;
        if self.solution_states.contains(state0) && self.solution_states.contains(state1) {
            Style::Bold
        } else {
            Style::Solid
        }
    }

    fn edge_color(&'a self, e: &Ed) -> Option<LabelText<'a>> {
        let state0 = self.nodes[e.0].0.state;
        let state1 = self.nodes[e.1].0.state;
        if self.solution_states.contains(state0) && self.solution_states.contains(state1) {
            Some(LabelText::LabelStr("red".into()))
        } else {
            None
        }
    }
}
