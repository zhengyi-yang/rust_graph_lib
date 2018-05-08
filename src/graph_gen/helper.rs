use std::hash::Hash;

use rand::{Rng, ThreadRng};
use itertools::Itertools;

use generic::IdType;
use generic::GraphType;
use generic::MapTrait;
use generic::Iter;

use graph_impl::TypedGraphMap;

pub fn complete_edge_pairs<'a, Ty>(n: usize) -> Iter<'a, (usize, usize)>
where
    Ty: 'a + GraphType,
{
    if Ty::is_directed() {
        Iter::new(Box::new(
            (0..n)
                .tuple_combinations()
                .flat_map(|(s, d)| vec![(s, d), (d, s)]),
        ))
    } else {
        Iter::new(Box::new((0..n).tuple_combinations()))
    }
}

pub fn random_node_label<Id, L, Ty>(rng: &mut ThreadRng, g: &TypedGraphMap<Id, L, Ty>) -> Option<L>
where
    Id: IdType,
    L: Hash + Eq + Clone,
    Ty: GraphType,
{
    let labels = g.get_node_label_map();

    if labels.is_empty() {
        return None;
    }

    let random_index = rng.gen_range(0, labels.len());
    labels.get_item(random_index).cloned()
}

pub fn random_edge_label<Id, L, Ty>(rng: &mut ThreadRng, g: &TypedGraphMap<Id, L, Ty>) -> Option<L>
where
    Id: IdType,
    L: Hash + Eq + Clone,
    Ty: GraphType,
{
    let labels = g.get_edge_label_map();

    if labels.is_empty() {
        return None;
    }

    let random_index = rng.gen_range(0, labels.len());
    labels.get_item(random_index).cloned()
}