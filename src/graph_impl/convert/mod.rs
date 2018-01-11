use std::hash::Hash;

use generic::{GraphTrait, DiGraphTrait, UnGraphTrait, GraphLabelTrait};
use generic::{NodeTrait, EdgeTrait};
use generic::GraphType;

use generic::MapTrait;

use graph_impl::{DiGraphMap, UnGraphMap, GraphMap, DiStaticGraph, UnStaticGraph, StaticGraph};
use graph_impl::graph_map::node::NodeMapTrait;
use graph_impl::graph_map::LabelMap;
use graph_impl::static_graph::EdgeVec;

/// Marker for None label
pub const END: usize = ::std::usize::MAX;

/// Map node id to a continuous range (sort by degree)
fn get_node_id_map<L, Ty>(g: &GraphMap<L, Ty>) -> LabelMap<usize>
    where L: Hash + Eq, Ty: GraphType {
    let mut node_degree: Vec<_> = g.nodes().map(|n| { (n.get_id(), n.degree()) }).collect();
    node_degree.sort_unstable_by_key(|&(_, d)| { d });


    let mut node_id_map = LabelMap::<usize>::new();
    for (n, _) in node_degree {
        node_id_map.add_item(n);
    }
    node_id_map
}

/// Re-assign node label id sorted by its frequency
fn get_node_label_id_map<L, Ty>(g: &GraphMap<L, Ty>) -> LabelMap<usize>
    where L: Hash + Eq, Ty: GraphType {
    let mut label_counter: Vec<_> = g.get_node_label_id_counter().into_iter().filter(|&(_, f)| { f > 0 }).collect();
    label_counter.sort_unstable_by_key(|&(_, f)| { f });

    let mut label_map = LabelMap::<usize>::new();
    for (n, _) in label_counter {
        label_map.add_item(n);
    }
    label_map
}

/// Re-assign edge label id sorted by its frequency
fn get_edge_label_id_map<L, Ty>(g: &GraphMap<L, Ty>) -> LabelMap<usize>
    where L: Hash + Eq, Ty: GraphType {
    let mut label_counter: Vec<_> = g.get_edge_label_id_counter().into_iter().filter(|&(_, f)| { f > 0 }).collect();
    label_counter.sort_unstable_by_key(|&(_, f)| { f });

    let mut label_map = LabelMap::<usize>::new();
    for (n, _) in label_counter {
        label_map.add_item(n);
    }
    label_map
}

fn get_node_labels<L, Ty>(g: &GraphMap<L, Ty>, node_map: &LabelMap<usize>, label_map: &LabelMap<usize>)
                          -> Option<Vec<usize>> where L: Hash + Eq, Ty: GraphType {
    if g.node_labels().next().is_none() {
        return None;
    }

    let mut labels: Vec<usize> = Vec::with_capacity(g.node_count());

    for node_id in node_map.items() {
        labels.push(
            match g.get_node(*node_id).unwrap().get_label() {
                Some(label) => label_map.find_index(&label).unwrap(),
                None => END,
            });
    }

    Some(labels)
}

fn get_edge_vec<L, Ty>(g: &GraphMap<L, Ty>, node_map: &LabelMap<usize>, label_map: &LabelMap<usize>)
                       -> EdgeVec where L: Hash + Eq, Ty: GraphType {
    let has_edge_label = g.edge_labels().next().is_some();
    let offset_len = g.node_count();
    let edge_len = if g.is_directed() {
        g.edge_count()
    } else {
        2 * g.edge_count()
    };

    let mut offset: usize = 0;
    let mut offset_vec: Vec<usize> = Vec::with_capacity(offset_len);
    let mut edge_vec: Vec<usize> = Vec::with_capacity(edge_len);

    let mut edge_labels: Option<Vec<usize>> = if has_edge_label {
        Some(Vec::with_capacity(edge_len))
    } else {
        None
    };

    for node_id in node_map.items() {
        offset_vec.push(offset);

        let mut neighbors: Vec<_> = g.neighbor_indices(*node_id).map(|i| node_map.find_index(&i).unwrap()).collect();

        neighbors.sort();
        offset += neighbors.len();

        for neighbor in neighbors {
            edge_vec.push(neighbor);

            if let Some(ref mut labels) = edge_labels {
                labels.push(match g.find_edge(*node_id, *node_map.find_item(neighbor).unwrap()).unwrap().get_label() {
                    Some(label) => label_map.find_index(&label).unwrap(),
                    None => END,
                });
            }
        }
    }

    match edge_labels {
        Some(labels) => EdgeVec::with_labels(offset_vec, edge_vec, labels),
        None => EdgeVec::new(offset_vec, edge_vec)
    }
}

fn get_in_edge_vec<L>(g: &DiGraphMap<L>, m: &LabelMap<usize>) -> EdgeVec
    where L: Hash + Eq {
    let offset_len = g.node_count();
    let edge_len = 2 * g.edge_count();

    let mut offset: usize = 0;
    let mut offset_vec: Vec<usize> = Vec::with_capacity(offset_len);
    let mut edge_vec: Vec<usize> = Vec::with_capacity(edge_len);

    for node_id in m.items() {
        offset_vec.push(offset);

        let mut neighbors: Vec<_> = g.in_neighbor_indices(*node_id).map(|i| m.find_index(&i).unwrap()).collect();

        neighbors.sort();
        offset += neighbors.len();

        for neighbor in neighbors {
            edge_vec.push(neighbor);
        }
    }

    EdgeVec::new(offset_vec, edge_vec)
}


impl<L: Hash + Eq> From<UnGraphMap<L>> for UnStaticGraph {
    fn from(g: UnGraphMap<L>) -> Self {
        let node_id_map = get_node_id_map(&g);
        let node_label_map = get_node_label_id_map(&g);
        let edge_label_map = get_edge_label_id_map(&g);
        let edge_vec = get_edge_vec(&g, &node_id_map, &node_label_map);
        let node_labels = get_node_labels(&g, &node_id_map, &edge_label_map);

        match node_labels {
            Some(labels) => UnStaticGraph::with_labels(g.node_count(), edge_vec, None, labels),
            None => UnStaticGraph::new(g.node_count(), edge_vec, None),
        }
    }
}

impl<L: Hash + Eq> From<DiGraphMap<L>> for DiStaticGraph {
    fn from(g: DiGraphMap<L>) -> Self {
        let node_id_map = get_node_id_map(&g);
        let node_label_map = get_node_label_id_map(&g);
        let edge_label_map = get_edge_label_id_map(&g);
        let edge_vec = get_edge_vec(&g, &node_id_map, &node_label_map);
        let node_labels = get_node_labels(&g, &node_id_map, &edge_label_map);

        let in_edge_vec = get_in_edge_vec(&g, &node_id_map);

        match node_labels {
            Some(labels) => DiStaticGraph::with_labels(g.node_count(), edge_vec, Some(in_edge_vec), labels),
            None => DiStaticGraph::new(g.node_count(), edge_vec, Some(in_edge_vec)),
        }
    }
}


//#[cfg(test)]
//mod tests {
//    use super::*;
//    use prelude::*;
//
//    #[test]
//    fn test_convert() {
//        let mut g = DiGraphMap::<u8>::new();
//        g.add_node(0, Some(0));
//        g.add_node(1, Some(1));
//        g.add_node(2, Some(2));
//        g.add_node(3, Some(3));
//        g.add_node(4, Some(4));
//        g.add_node(5, Some(5));
//
//        g.add_edge(0, 1, Some(0));
//        g.add_edge(1, 0, Some(1));
//        g.add_edge(1, 2, Some(2));
//        g.add_edge(1, 3, Some(3));
//        g.add_edge(4, 5, Some(4));
//
//        StaticGraph::from(g);
//    }
//}