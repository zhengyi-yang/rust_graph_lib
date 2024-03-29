extern crate rust_graph;
extern crate time;

#[macro_use]
extern crate serde_derive;

use std::fs::create_dir_all;
use std::hash::Hash;
use std::marker::PhantomData;
use std::path::Path;

use time::PreciseTime;

use rust_graph::graph_impl::{EdgeVec, TypedStaticGraph};
use rust_graph::io::serde::{Deserialize, Deserializer};
use rust_graph::io::write_to_csv;
use rust_graph::map::SetMap;
use rust_graph::prelude::*;
use rust_graph::{UnGraphMap, UnStaticGraph};

fn main() {
    let args: Vec<_> = std::env::args().collect();

    let in_file = Path::new(&args[1]);
    let out_dir = Path::new(&args[2]);

    let start = PreciseTime::now();

    println!("Loading {:?}", &in_file);
    let g = Deserializer::import::<InnerUnlabeledGraph<u32, Undirected>, _>(in_file)
        .expect("Deserializer error")
        .to_static_graph::<Void, Void>();

    println!("{:?}", g.get_node_label_map());
    println!("{:?}", g.get_edge_label_map());

    if !out_dir.exists() {
        create_dir_all(out_dir).unwrap();
    }

    println!("Exporting to {:?}...", &out_dir);

    write_to_csv(&g, out_dir.join("nodes.csv"), out_dir.join("edges.csv")).unwrap();

    let end = PreciseTime::now();

    println!("Finished in {} seconds.", start.to(end));
}

#[derive(Clone, Serialize, Deserialize)]
struct OldEdgeVec<Id: IdType> {
    offsets: Vec<Id>,
    edges: Vec<Id>,
    labels: Option<Vec<Id>>,
}

impl<Id: IdType> OldEdgeVec<Id> {
    pub fn to_edge_vec(self) -> EdgeVec<Id> {
        if self.labels.is_some() {
            EdgeVec::with_labels(
                self.offsets.into_iter().map(|x| x.id()).collect(),
                self.edges,
                self.labels.unwrap(),
            )
        } else {
            EdgeVec::new(
                self.offsets.into_iter().map(|x| x.id()).collect(),
                self.edges,
            )
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct InnerUnlabeledGraph<Id: IdType, Ty: GraphType> {
    num_nodes: usize,
    num_edges: usize,
    edge_vec: OldEdgeVec<Id>,
    in_edge_vec: Option<OldEdgeVec<Id>>,
    labels: Option<Vec<Id>>,
    graph_type: PhantomData<Ty>,
}

impl<Id: IdType, Ty: GraphType> InnerUnlabeledGraph<Id, Ty> {
    pub fn to_static_graph<NL: Hash + Eq, EL: Hash + Eq>(self) -> TypedStaticGraph<Id, NL, EL, Ty> {
        TypedStaticGraph::from_raw(
            self.num_nodes,
            self.num_edges,
            self.edge_vec.to_edge_vec(),
            self.in_edge_vec.map(|x| x.to_edge_vec()),
            self.labels,
            SetMap::new(),
            SetMap::new(),
        )
    }
}
