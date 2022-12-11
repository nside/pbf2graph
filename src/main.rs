use csv;
use osmpbf::{Element, IndexedReader};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

struct RoadGraph {
    nodes: HashMap<i64, [f64; 2]>,
    edges: Vec<(i64, i64)>,
}

impl RoadGraph {
    fn new() -> RoadGraph {
        RoadGraph {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    fn add_node(&mut self, id: i64, lat: f64, lon: f64) {
        self.nodes.insert(id, [lat, lon]);
    }

    fn add_edge(&mut self, id1: i64, id2: i64) {
        self.edges.push((id1, id2));
    }

    fn write_csv(&self, dir: &Path) -> Result<(), Box<dyn Error>> {
        // Create directory if it doesn't exist
        std::fs::create_dir_all(dir)?;

        // Create node CSV file and writer
        let node_path = dir.join("nodes.csv");
        let node_file = std::fs::File::create(node_path)?;
        let mut node_writer = csv::Writer::from_writer(node_file);

        // Create edge CSV file and writer
        let edge_path = dir.join("edges.csv");
        let edge_file = std::fs::File::create(edge_path)?;
        let mut edge_writer = csv::Writer::from_writer(edge_file);

        // Write nodes to CSV file
        for (id, coords) in self.nodes.iter() {
            let lat = coords[0];
            let lon = coords[1];
            node_writer.serialize((id, lat, lon))?;
        }

        // Write edges to CSV file
        for (id1, id2) in self.edges.iter() {
            edge_writer.serialize((id1, id2))?;
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Read command line argument and create IndexedReader
    let arg = std::env::args_os()
        .nth(1)
        .ok_or("need a *.osm.pbf file as argument")?;
    let mut reader = IndexedReader::from_path(&arg)?;

    let mut graph = RoadGraph::new();

    // Read file and add nodes and edges to road graph
    reader.read_ways_and_deps(
        |way| {
            // Filter ways. Return true if tags contain "highway"
            way.tags().any(|key_value| key_value.0 == "highway")
        },
        |element| {
            match element {
                Element::Way(way) => {
                    let node_ids = way.refs().collect::<Vec<_>>();
                    for i in 0..node_ids.len() - 1 {
                        graph.add_edge(node_ids[i], node_ids[i + 1]);
                    }
                }
                Element::Node(node) => {
                    graph.add_node(node.id(), node.lat(), node.lon());
                }
                Element::DenseNode(dense_node) => {
                    graph.add_node(dense_node.id(), dense_node.lat(), dense_node.lon());
                }
                Element::Relation(_) => {} // should not occur
            }
        },
    )?;

    let csv_dir = Path::new("data");
    graph.write_csv(csv_dir)?;

    Ok(())
}
