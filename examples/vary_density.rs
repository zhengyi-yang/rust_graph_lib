extern crate pbr;
extern crate rand;
extern crate rust_graph;
extern crate time;

use std::path::Path;

use pbr::ProgressBar;
use rand::{thread_rng, Rng};
use time::PreciseTime;

use rust_graph::graph_impl::UnGraphMap;
use rust_graph::io::serde::*;
use rust_graph::prelude::*;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    let in_graph = Path::new(&args[1]);
    let out_dir = Path::new(&args[2]);

    let average_degrees = vec![20, 50, 100, 150, 200];

    let start = PreciseTime::now();

    let mut rng = thread_rng();

    println!("Loading {:?}", &in_graph);
    let mut g: UnGraphMap<String> = Deserializer::import(in_graph).unwrap();

    let num_of_nodes = g.node_count();
    let num_of_edges = g.edge_count();

    println!("Average degree: {}", 2 * num_of_edges / num_of_nodes);
    assert_eq!(g.max_seen_id().unwrap().id(), num_of_nodes - 1);

    for d in average_degrees {
        println!("Targeting average degree {}: ", d);

        let target_num_of_edges = d * num_of_nodes / 2;

        assert!(target_num_of_edges > num_of_edges);

        let i = target_num_of_edges - num_of_edges;
        let nodes = DefaultId::new(num_of_nodes);

        let mut pb = ProgressBar::new(i as u64);

        for _ in 0..i {
            pb.inc();
            loop {
                let s = rng.gen_range(0, nodes);
                let t = rng.gen_range(0, nodes);
                if s != t && !g.has_edge(s, t) {
                    g.add_edge(s, t, None);
                    break;
                }
            }
        }

        let file_name = in_graph
            .components()
            .last()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap();
        let export_filename = format!(
            "{}_{}_{}_{}.graphmap",
            file_name,
            g.node_count(),
            g.edge_count(),
            d
        );
        let export_path = out_dir.join(export_filename);

        pb.finish_print("done");

        println!("Exporting to {:?}...", export_path);

        Serializer::export(&g, export_path).unwrap();
    }

    let end = PreciseTime::now();

    println!("Finished in {} seconds.", start.to(end));
}
