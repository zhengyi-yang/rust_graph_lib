extern crate rust_graph;
extern crate time;
extern crate itertools;

use time::PreciseTime;
use itertools::Itertools;

use rust_graph::io::serde::{Deserialize, Deserializer};
use rust_graph::prelude::*;
use rust_graph::{UnGraphMap, UnStaticGraph};

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();

    let start = PreciseTime::now();

    for arg in args {
        println!("------------------------------");
        println!("Loading {}", &arg);

        //        let g: UnGraphMap<String> = Deserializer::import(arg).unwrap();
        let g: UnStaticGraph<String> = Deserializer::import(arg).unwrap();

        let node_labels = g.get_node_label_counter();
        let edge_labels = g.get_edge_label_counter();

        println!("Node labels:");

        for (label, count) in node_labels.into_iter().sorted_by_key(|&(_, v)| v) {
            println!("- {} : {}", label, count);
        }

        println!();
        println!("Edge labels:");

        for (label, count) in edge_labels.into_iter().sorted_by_key(|&(_, v)| v) {
            println!("- {} : {}", label, count);
        }

        println!("------------------------------");
    }

    let end = PreciseTime::now();

    println!("Finished in {} seconds.", start.to(end));
}
