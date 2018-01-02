use generic::Iter;

pub trait ItemMap<L> {
    fn add_item(&mut self, item: L) -> usize;
    fn find_item(&self, id: usize) -> Option<&L>;

    fn items(&self) -> Iter<&L>;
    fn len(&self) -> usize;
}