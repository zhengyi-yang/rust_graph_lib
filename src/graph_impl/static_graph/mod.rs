pub mod edge_vec;
pub mod graph;
pub mod mmap;
pub mod node;
pub mod static_edge_iter;

pub use graph_impl::static_graph::edge_vec::EdgeVec;
pub use graph_impl::static_graph::edge_vec::EdgeVecTrait;
pub use graph_impl::static_graph::graph::{DiStaticGraph, StaticGraph, UnStaticGraph};
pub use graph_impl::static_graph::graph::{
    TypedDiStaticGraph, TypedStaticGraph, TypedUnStaticGraph,
};
pub use graph_impl::static_graph::node::StaticNode;
