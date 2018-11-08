use std::borrow::Cow;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::replace;

use bincode::Result;
use serde;

use generic::map::MapTrait;
use generic::Iter;
use generic::{
    DefaultId, DefaultTy, DiGraphTrait, Directed, EdgeType, GeneralGraph, GraphLabelTrait,
    GraphTrait, GraphType, IdType, NodeType, UnGraphTrait, Undirected,
};
use graph_impl::static_graph::mmap::graph_mmap::StaticGraphMmapAux;
use graph_impl::static_graph::node::StaticNode;
use graph_impl::static_graph::static_edge_iter::StaticEdgeIndexIter;
use graph_impl::static_graph::{EdgeVec, EdgeVecTrait};
use graph_impl::{Edge, Graph};
use io::mmap::dump;
use io::serde::{Serialize, Serializer};
use map::SetMap;

pub type TypedUnStaticGraph<Id, NL, EL = NL, L = Id> = TypedStaticGraph<Id, NL, EL, Undirected, L>;
pub type TypedDiStaticGraph<Id, NL, EL = NL, L = Id> = TypedStaticGraph<Id, NL, EL, Directed, L>;
pub type StaticGraph<NL, EL, Ty = DefaultTy, L = DefaultId> =
    TypedStaticGraph<DefaultId, NL, EL, Ty, L>;
pub type UnStaticGraph<NL, EL = NL, L = DefaultId> = StaticGraph<NL, EL, Undirected, L>;
pub type DiStaticGraph<NL, EL = NL, L = DefaultId> = StaticGraph<NL, EL, Directed, L>;

/// `StaticGraph` is a memory-compact graph data structure.
/// The labels of both nodes and edges, if exist, are encoded as `Integer`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TypedStaticGraph<Id: IdType, NL: Hash + Eq, EL: Hash + Eq, Ty: GraphType, L: IdType = Id>
{
    num_nodes: usize,
    num_edges: usize,
    edge_vec: EdgeVec<Id, L>,
    in_edge_vec: Option<EdgeVec<Id, L>>,
    // Maintain the node's labels, whose index is aligned with `offsets`.
    labels: Option<Vec<L>>,
    // A marker of thr graph type, namely, directed or undirected.
    graph_type: PhantomData<Ty>,
    // A map of node labels.
    node_label_map: SetMap<NL>,
    // A map of edge labels.
    edge_label_map: SetMap<EL>,
}

impl<Id: IdType, NL: Hash + Eq, EL: Hash + Eq, Ty: GraphType, L: IdType>
    TypedStaticGraph<Id, NL, EL, Ty, L>
{
    pub fn new(num_nodes: usize, edges: EdgeVec<Id, L>, in_edges: Option<EdgeVec<Id, L>>) -> Self {
        if Ty::is_directed() {
            assert!(in_edges.is_some());
            let num_of_in_edges = in_edges.as_ref().unwrap().num_edges();
            let num_of_out_edges = edges.num_edges();
            if num_of_in_edges != num_of_out_edges {
                panic!(
                    "Unequal length: {} out edges but {} in edges.",
                    num_of_out_edges, num_of_in_edges
                );
            }
        }

        TypedStaticGraph {
            num_nodes,
            num_edges: if Ty::is_directed() {
                edges.num_edges()
            } else {
                edges.num_edges() >> 1
            },
            edge_vec: edges,
            in_edge_vec: in_edges,
            labels: None,
            node_label_map: SetMap::<NL>::new(),
            edge_label_map: SetMap::<EL>::new(),
            graph_type: PhantomData,
        }
    }

    pub fn with_labels(
        num_nodes: usize,
        edges: EdgeVec<Id, L>,
        in_edges: Option<EdgeVec<Id, L>>,
        labels: Vec<L>,
        node_label_map: SetMap<NL>,
        edge_label_map: SetMap<EL>,
    ) -> Self {
        if Ty::is_directed() {
            assert!(in_edges.is_some());
            let num_of_in_edges = in_edges.as_ref().unwrap().num_edges();
            let num_of_out_edges = edges.num_edges();
            if num_of_in_edges != num_of_out_edges {
                panic!(
                    "Unequal length: {} out edges but {} in edges.",
                    num_of_out_edges, num_of_in_edges
                );
            }
        }
        if num_nodes != labels.len() {
            panic!(
                "Unequal length: there are {} nodes, but {} labels",
                num_nodes,
                labels.len()
            );
        }

        TypedStaticGraph {
            num_nodes,
            num_edges: if Ty::is_directed() {
                edges.num_edges()
            } else {
                edges.num_edges() >> 1
            },
            edge_vec: edges,
            in_edge_vec: in_edges,
            labels: Some(labels),
            node_label_map,
            edge_label_map,
            graph_type: PhantomData,
        }
    }

    pub fn from_raw(
        num_nodes: usize,
        num_edges: usize,
        edge_vec: EdgeVec<Id, L>,
        in_edge_vec: Option<EdgeVec<Id, L>>,
        labels: Option<Vec<L>>,
        node_label_map: SetMap<NL>,
        edge_label_map: SetMap<EL>,
    ) -> Self {
        if Ty::is_directed() {
            assert!(in_edge_vec.is_some());
            let num_of_in_edges = in_edge_vec.as_ref().unwrap().num_edges();
            let num_of_out_edges = edge_vec.num_edges();
            if num_of_in_edges != num_of_out_edges {
                panic!(
                    "Unequal length: {} out edges but {} in edges.",
                    num_of_out_edges, num_of_in_edges
                );
            }
            if num_edges != edge_vec.num_edges() {
                panic!(
                    "Directed: num_edges {}, edge_vec {} edges",
                    num_edges,
                    edge_vec.num_edges()
                );
            }
        } else {
            if num_edges != edge_vec.num_edges() >> 1 {
                warn!(
                    "undirected: num_edges {}, edge_vec {} edges, graph may contain self loop.",
                    num_edges,
                    edge_vec.num_edges()
                );
            }
        }
        if labels.is_some() {
            let num_of_labels = labels.as_ref().unwrap().len();
            if num_nodes != num_of_labels {
                panic!(
                    "Unequal length: there are {} nodes, but {} labels",
                    num_nodes, num_of_labels
                );
            }
        }

        TypedStaticGraph {
            num_nodes,
            num_edges,
            edge_vec,
            in_edge_vec,
            labels,
            node_label_map,
            edge_label_map,
            graph_type: PhantomData,
        }
    }

    pub fn get_edge_vec(&self) -> &EdgeVec<Id, L> {
        &self.edge_vec
    }

    pub fn get_in_edge_vec(&self) -> &Option<EdgeVec<Id, L>> {
        &self.in_edge_vec
    }

    pub fn get_labels(&self) -> &Option<Vec<L>> {
        &self.labels
    }

    pub fn get_node_label_map(&self) -> &SetMap<NL> {
        &self.node_label_map
    }

    pub fn get_edge_label_map(&self) -> &SetMap<EL> {
        &self.edge_label_map
    }

    pub fn get_edge_vec_mut(&mut self) -> &mut EdgeVec<Id, L> {
        &mut self.edge_vec
    }

    pub fn get_in_edge_vec_mut(&mut self) -> &mut Option<EdgeVec<Id, L>> {
        &mut self.in_edge_vec
    }

    pub fn get_labels_mut(&mut self) -> &mut Option<Vec<L>> {
        &mut self.labels
    }

    pub fn get_node_label_map_mut(&mut self) -> &mut SetMap<NL> {
        &mut self.node_label_map
    }

    pub fn get_edge_label_map_mut(&mut self) -> &mut SetMap<EL> {
        &mut self.edge_label_map
    }

    pub fn remove_node_labels(&mut self) {
        self.labels = None;
        self.node_label_map = SetMap::new();
    }

    pub fn remove_edge_labels(&mut self) {
        self.edge_vec.remove_labels();
        self.in_edge_vec.as_mut().map(|ref mut e| e.remove_labels());
        self.edge_label_map = SetMap::new();
    }

    pub fn remove_labels(&mut self) {
        self.remove_node_labels();
        self.remove_edge_labels();
    }

    pub fn shrink_to_fit(&mut self) {
        self.edge_vec.shrink_to_fit();
        if let Some(ref mut in_edge_vec) = self.in_edge_vec {
            in_edge_vec.shrink_to_fit();
        }
        if let Some(ref mut labels) = self.labels {
            labels.shrink_to_fit();
        }
    }

    pub fn find_edge_index(&self, start: Id, target: Id) -> Option<usize> {
        self.edge_vec.find_edge_index(start, target)
    }

    pub fn to_int_label(mut self) -> TypedStaticGraph<Id, L, L, Ty, L> {
        TypedStaticGraph {
            num_nodes: self.num_nodes,
            num_edges: self.num_edges,
            edge_vec: replace(&mut self.edge_vec, EdgeVec::default()),
            in_edge_vec: self.in_edge_vec.take(),
            labels: self.labels.take(),
            node_label_map: (0..self.node_label_map.len()).map(L::new).collect(),
            edge_label_map: (0..self.edge_label_map.len()).map(L::new).collect(),
            graph_type: PhantomData,
        }
    }
}

impl<Id: IdType + Copy, NL: Hash + Eq, EL: Hash + Eq, Ty: GraphType>
    TypedStaticGraph<Id, NL, EL, Ty>
where
    NL: serde::Serialize + Clone,
    EL: serde::Serialize + Clone,
{
    pub fn dump_mmap(&self, prefix: &str) -> Result<()> {
        let edges_prefix = format!("{}_OUT", prefix);
        let in_edges_prefix = format!("{}_IN", prefix);
        let label_file = format!("{}.labels", prefix);

        let aux_map_file = format!("{}_aux.bin", prefix);

        self.edge_vec.dump_mmap(&edges_prefix)?;
        if let Some(ref in_edges) = self.in_edge_vec {
            in_edges.dump_mmap(&in_edges_prefix)?;
        }

        if let Some(ref labels) = self.labels {
            unsafe {
                dump(labels, ::std::fs::File::create(label_file)?)?;
            }
        }

        let aux_file = StaticGraphMmapAux::new(
            self.num_nodes,
            self.num_edges,
            self.node_label_map.clone(),
            self.edge_label_map.clone(),
        );

        Serializer::export(&aux_file, &aux_map_file)?;

        Ok(())
    }
}

impl<Id: IdType, NL: Hash + Eq, EL: Hash + Eq, Ty: GraphType, L: IdType> GraphTrait<Id, L>
    for TypedStaticGraph<Id, NL, EL, Ty, L>
{
    fn get_node(&self, id: Id) -> NodeType<Id, L> {
        if !self.has_node(id) {
            return NodeType::None;
        }

        match self.labels {
            Some(ref labels) => NodeType::StaticNode(StaticNode::new_static(id, labels[id.id()])),
            None => NodeType::StaticNode(StaticNode::new(id, None)),
        }
    }

    fn get_edge(&self, start: Id, target: Id) -> EdgeType<Id, L> {
        if !self.has_edge(start, target) {
            return None;
        }

        let _label = self.edge_vec.find_edge_label_id(start, target);
        match _label {
            Some(label) => Some(Edge::new_static(start, target, *label)),
            None => Some(Edge::new(start, target, None)),
        }
    }

    fn has_node(&self, id: Id) -> bool {
        id.id() < self.num_nodes
    }

    fn has_edge(&self, start: Id, target: Id) -> bool {
        self.edge_vec.has_edge(start, target)
    }

    fn node_count(&self) -> usize {
        self.num_nodes
    }

    fn edge_count(&self) -> usize {
        self.num_edges
    }

    #[inline(always)]
    fn is_directed(&self) -> bool {
        Ty::is_directed()
    }

    fn node_indices(&self) -> Iter<Id> {
        Iter::new(Box::new((0..self.num_nodes).map(|x| Id::new(x))))
    }

    fn edge_indices(&self) -> Iter<(Id, Id)> {
        Iter::new(Box::new(StaticEdgeIndexIter::new(
            Box::new(&self.edge_vec),
            self.is_directed(),
        )))
    }

    fn nodes(&self) -> Iter<NodeType<Id, L>> {
        match self.labels {
            None => {
                let node_iter = self
                    .node_indices()
                    .map(|i| NodeType::StaticNode(StaticNode::new(i, None)));

                Iter::new(Box::new(node_iter))
            }
            Some(ref labels) => {
                let node_iter = self
                    .node_indices()
                    .zip(labels.iter())
                    .map(|n| NodeType::StaticNode(StaticNode::new_static(n.0, *n.1)));

                Iter::new(Box::new(node_iter))
            }
        }
    }

    /// In `StaticGraph`, an edge is an attribute (as adjacency list) of a node.
    /// Thus, we return an iterator over the labels of all edges.
    fn edges(&self) -> Iter<EdgeType<Id, L>> {
        let labels = self.edge_vec.get_labels();
        if labels.is_empty() {
            Iter::new(Box::new(
                self.edge_indices().map(|i| Some(Edge::new(i.0, i.1, None))),
            ))
        } else {
            Iter::new(Box::new(
                self.edge_indices()
                    .zip(labels.iter())
                    .map(|e| Some(Edge::new_static((e.0).0, (e.0).1, *e.1))),
            ))
        }
    }

    fn degree(&self, id: Id) -> usize {
        self.edge_vec.degree(id)
    }

    fn neighbors_iter(&self, id: Id) -> Iter<Id> {
        let neighbors = self.edge_vec.neighbors(id);

        Iter::new(Box::new(neighbors.iter().map(|x| *x)))
    }

    fn neighbors(&self, id: Id) -> Cow<[Id]> {
        self.edge_vec.neighbors(id).into()
    }

    fn max_seen_id(&self) -> Option<Id> {
        Some(Id::new(self.node_count() - 1))
    }

    fn max_possible_id(&self) -> Id {
        Id::max_value()
    }

    fn implementation(&self) -> Graph {
        Graph::StaticGraph
    }
}

impl<Id: IdType, NL: Hash + Eq, EL: Hash + Eq, Ty: GraphType, L: IdType>
    GraphLabelTrait<Id, NL, EL, L> for TypedStaticGraph<Id, NL, EL, Ty, L>
{
    fn get_node_label_map(&self) -> &SetMap<NL> {
        &self.node_label_map
    }

    fn get_edge_label_map(&self) -> &SetMap<EL> {
        &self.edge_label_map
    }
}

impl<Id: IdType, NL: Hash + Eq, EL: Hash + Eq, L: IdType> UnGraphTrait<Id, L>
    for TypedUnStaticGraph<Id, NL, EL, L>
{}

impl<Id: IdType, NL: Hash + Eq, EL: Hash + Eq, L: IdType> DiGraphTrait<Id, L>
    for TypedDiStaticGraph<Id, NL, EL, L>
{
    fn in_degree(&self, id: Id) -> usize {
        self.in_neighbors(id).len()
    }

    fn in_neighbors_iter(&self, id: Id) -> Iter<Id> {
        let in_neighbors = self.in_edge_vec.as_ref().unwrap().neighbors(id);

        Iter::new(Box::new(in_neighbors.iter().map(|x| *x)))
    }

    fn in_neighbors(&self, id: Id) -> Cow<[Id]> {
        self.in_edge_vec.as_ref().unwrap().neighbors(id).into()
    }
}

impl<Id: IdType, NL: Hash + Eq, EL: Hash + Eq, L: IdType> GeneralGraph<Id, NL, EL, L>
    for TypedUnStaticGraph<Id, NL, EL, L>
{
    fn as_graph(&self) -> &GraphTrait<Id, L> {
        self
    }

    fn as_labeled_graph(&self) -> &GraphLabelTrait<Id, NL, EL, L> {
        self
    }
}

impl<Id: IdType, NL: Hash + Eq, EL: Hash + Eq, L: IdType> GeneralGraph<Id, NL, EL, L>
    for TypedDiStaticGraph<Id, NL, EL, L>
{
    fn as_graph(&self) -> &GraphTrait<Id, L> {
        self
    }

    fn as_labeled_graph(&self) -> &GraphLabelTrait<Id, NL, EL, L> {
        self
    }

    fn as_digraph(&self) -> Option<&DiGraphTrait<Id, L>> {
        Some(self)
    }
}
