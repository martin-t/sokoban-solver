use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    fs,
    hash::BuildHasher,
    process::Command,
};

use dot::{self, Edges, GraphWalk, Id, LabelText, Labeller, Nodes};

use crate::{map::Map, state::State};

type Nd<'a> = &'a State;
type Ed<'a> = (Nd<'a>, Nd<'a>);

struct Graph<'a, H: BuildHasher> {
    map: &'a dyn Map,
    prevs: &'a HashMap<&'a State, &'a State, H>,
    ids: RefCell<HashMap<&'a State, u32>>,
    counter: Cell<u32>,
}

impl<'a, H: BuildHasher> GraphWalk<'a, Nd<'a>, Ed<'a>> for Graph<'a, H> {
    fn nodes(&'a self) -> Nodes<'a, Nd<'a>> {
        self.prevs.keys().map(|&key| key).collect()
    }

    fn edges(&'a self) -> Edges<'a, Ed<'a>> {
        self.prevs.iter().map(|(&k, &v)| (k, v)).collect()
    }

    fn source(&'a self, e: &Ed<'a>) -> Nd<'a> {
        e.1
    }

    fn target(&'a self, e: &Ed<'a>) -> Nd<'a> {
        e.0
    }
}

impl<'a, H: BuildHasher> Labeller<'a, Nd<'a>, Ed<'a>> for Graph<'a, H> {
    fn graph_id(&'a self) -> Id<'a> {
        Id::new("G").unwrap()
    }

    fn node_id(&'a self, n: &Nd<'a>) -> Id<'a> {
        Id::new(format!(
            "N{}",
            self.ids.borrow_mut().entry(*n).or_insert_with(|| {
                let i = self.counter.get();
                self.counter.set(self.counter.get() + 1);
                i
            })
        ))
        .unwrap()
    }

    // TODO shape, color
    //fn node_shape(&'a self, _: &Nd<'a>) -> Option<LabelText<'a>> {
    //    Some(LabelText::LabelStr("plaintext".into()))
    //}

    fn node_label(&'a self, n: &Nd<'a>) -> LabelText<'a> {
        LabelText::EscStr(format!("{}", self.map.xsb_with_state(n)).into())
    }

    //fn node_color(&'a self, _: &Nd<'a>) -> Option<LabelText<'a>>{
    //    Some(LabelText::LabelStr("gray75".into()))
    //}
}

crate fn draw_states<H: BuildHasher>(map: &dyn Map, prevs: &HashMap<&State, &State, H>) {
    let graph = Graph {
        map,
        prevs,
        ids: RefCell::new(HashMap::new()),
        counter: Cell::new(0),
    };

    let mut writer = Vec::new();
    dot::render(&graph, &mut writer).unwrap();
    let s = String::from_utf8(writer).unwrap();
    let s = s.replace("digraph G {", "digraph G {\n    graph [fontname = \"hack\"];\n    node [fontname = \"hack\"];\n    edge [fontname = \"hack\"];");
    fs::write("state-space.dot", &s).unwrap();

    let status = Command::new("dot")
        .args(&["-Tsvg", "-O", "state-space.dot"])
        .status()
        .unwrap();
    assert!(status.success());
}
