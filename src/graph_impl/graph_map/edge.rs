use generic::{EdgeTrait, MutEdgeTrait};

#[derive(Debug, PartialEq, Clone)]
pub struct Edge {
    start: usize,
    target: usize,
    label: Option<usize>,
}


impl Edge {
    pub fn new(start: usize, target: usize, label: Option<usize>) -> Self {
        Edge {
            start,
            target,
            label,
        }
    }
}

impl EdgeTrait for Edge {
    fn get_start(&self) -> usize {
        self.start
    }

    fn get_target(&self) -> usize {
        self.target
    }

    fn get_label(&self) -> Option<usize> {
        self.label
    }
}

impl MutEdgeTrait for Edge {
    fn set_label(&mut self, label: usize) {
        self.label = Some(label);
    }
}
