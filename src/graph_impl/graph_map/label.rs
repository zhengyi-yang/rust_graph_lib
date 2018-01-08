use std::hash::Hash;

use ordermap::OrderSet;

use generic::MapTrait;
use generic::Iter;

pub struct LabelMap<L> {
    labels: OrderSet<L>
}

impl<L> LabelMap<L> {
    pub fn new() -> Self {
        LabelMap {
            labels: OrderSet::<L>::new()
        }
    }
}

impl<L: Hash + Eq> MapTrait<L> for LabelMap<L> {
    fn add_item(&mut self, item: L) -> usize {
        if self.labels.contains(&item) {
            self.labels.get_full(&item).unwrap().0
        } else {
            self.labels.insert(item);
            self.len() - 1
        }
    }

    fn find_item(&self, id: usize) -> Option<&L> {
        self.labels.get_index(id)
    }

    fn contains(&self, item: L) -> bool {
        self.labels.contains(&item)
    }

    fn items<'a>(&'a self) -> Iter<'a, &L> {
        Iter::new(Box::new(self.labels.iter()))
    }

    fn len(&self) -> usize {
        self.labels.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_find_item() {
        let mut label_map = LabelMap::<String>::new();

        assert_eq!(label_map.len(), 0);

        assert_eq!(label_map.add_item(String::from("zero")), 0);
        assert_eq!(label_map.add_item(String::from("first")), 1);
        assert_eq!(label_map.add_item(String::from("zero")), 0);
        assert_eq!(label_map.add_item(String::from("first")), 1);

        assert_eq!(label_map.len(), 2);
        assert_eq!(label_map.find_item(0), Some(&String::from("zero")));
        assert_eq!(label_map.find_item(1), Some(&String::from("first")));

        assert_eq!(label_map.find_item(2), None);
    }
}