use std::{borrow::Cow, collections::HashMap, fs, hash::BuildHasher, process::Command};

use dot::{self, Edges, GraphWalk, Id, LabelText, Labeller, Nodes, Style};
use fnv::FnvHashMap;

use crate::{map::Map, solver::a_star::SearchNode, state::State};

type Nd = usize;
type Ed = (usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Type {
    Queued,
    Duplicate,
    Unique,
}

// TODO merge nodes with the same state?
#[derive(Debug)]
crate struct Graph<'a> {
    map: &'a dyn Map,
    node_to_index: FnvHashMap<SearchNode<'a>, usize>,
    nodes: Vec<(SearchNode<'a>, Type)>,
    edges: Vec<(usize, usize)>,
}

impl<'a> Graph<'a> {
    crate fn new(map: &'a dyn Map) -> Self {
        Self {
            map,
            node_to_index: FnvHashMap::default(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    crate fn add(&mut self, node: SearchNode<'a>, prev: Option<SearchNode<'a>>) {
        assert!(!self.node_to_index.contains_key(&node));

        let node_index = self.nodes.len();

        self.node_to_index.insert(node, node_index);
        self.nodes.push((node, Type::Queued));

        if let Some(prev) = prev {
            let prev_index = self.node_to_index[&prev];
            self.edges.push((prev_index, node_index));
        }
    }

    crate fn mark_duplicate(&mut self, node: SearchNode<'a>) {
        self.nodes[self.node_to_index[&node]].1 = Type::Duplicate;
    }

    crate fn mark_unique(&mut self, node: SearchNode<'a>) {
        self.nodes[self.node_to_index[&node]].1 = Type::Unique;
    }

    crate fn draw_states<H: BuildHasher>(&self, _prevs: &HashMap<&State, &State, H>) {
        // TODO prevs

        let mut writer = Vec::new();
        dot::render(self, &mut writer).unwrap();
        let s = String::from_utf8(writer).unwrap();
        let s = s.replace("digraph G {", "digraph G {\n    graph [fontname = \"hack\"];\n    node [fontname = \"hack\"];\n    edge [fontname = \"hack\"];");
        fs::write("state-space.dot", &s).unwrap();

        let status = Command::new("dot")
            .args(&["-Tsvg", "-O", "state-space.dot"])
            .status()
            .unwrap();
        assert!(status.success());
    }
}

impl<'a> GraphWalk<'a, Nd, Ed> for Graph<'a> {
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

impl<'a> Labeller<'a, Nd, Ed> for Graph<'a> {
    fn graph_id(&'a self) -> Id<'a> {
        Id::new("G").unwrap()
    }

    fn node_id(&'a self, n: &Nd) -> Id<'a> {
        Id::new(format!("N{}", n)).unwrap()
    }

    fn node_label(&'a self, n: &Nd) -> LabelText<'a> {
        let node = self.nodes[*n].0;
        LabelText::EscStr(
            format!(
                "d: {}, h: {}\ncost: {}\n{}",
                node.dist,
                node.cost - node.dist,
                node.cost,
                self.map.xsb_with_state(node.state)
            )
            .into(),
        )
    }

    fn node_style(&'a self, n: &Nd) -> Style {
        let node_type = self.nodes[*n].1;
        if node_type == Type::Queued {
            Style::Solid
        } else {
            Style::Filled
        }
    }

    fn node_color(&'a self, n: &Nd) -> Option<LabelText<'a>> {
        let node_type = self.nodes[*n].1;
        Some(LabelText::LabelStr(
            match node_type {
                Type::Unique => "red",
                Type::Duplicate => "gray",
                Type::Queued => return None,
            }
            .into(),
        ))
    }
}
