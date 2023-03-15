extern crate csv;
extern crate osmpbf;
use osmpbf::{Element, IndexedReader};
use std::error::Error;
use std::path::Path;

use ordered_float::OrderedFloat;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Copy, Clone, PartialEq, Eq)]
struct NodeWithDistance {
    node: i64,
    distance: OrderedFloat<f64>,
}

impl Ord for NodeWithDistance {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.cmp(&self.distance)
    }
}

impl PartialOrd for NodeWithDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct RoadGraph {
    nodes: HashMap<i64, [f64; 2]>,
    edges: Vec<(i64, i64)>,
}

impl RoadGraph {
    pub fn new() -> RoadGraph {
        RoadGraph {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, id: i64, lat: f64, lon: f64) {
        self.nodes.insert(id, [lat, lon]);
    }

    pub fn add_edge(&mut self, id1: i64, id2: i64) {
        self.edges.push((id1, id2));
    }
    pub fn shortest_path(&self, start: i64, end: i64) -> Option<Vec<i64>> {
        let mut distances: HashMap<i64, f64> = HashMap::new();
        let mut visited: HashMap<i64, bool> = HashMap::new();
        let mut prev: HashMap<i64, i64> = HashMap::new();
        let mut queue = BinaryHeap::new();

        distances.insert(start, 0.0);
        queue.push(NodeWithDistance {
            node: start,
            distance: OrderedFloat(0.0),
        });

        while let Some(NodeWithDistance { node, distance }) = queue.pop() {
            if *visited.get(&node).unwrap_or(&false) {
                continue;
            }

            visited.insert(node, true);

            if node == end {
                let mut path = Vec::new();
                let mut current = end;

                while current != start {
                    path.push(current);
                    current = *prev.get(&current)?;
                }

                path.push(start);
                path.reverse();
                return Some(path);
            }

            for edge in &self.edges {
                if edge.0 == node {
                    let neighbor = edge.1;
                    let edge_length = self.distance(edge.0, edge.1);
                    let new_distance = distance + edge_length;

                    if new_distance
                        < OrderedFloat(*distances.get(&neighbor).unwrap_or(&f64::INFINITY))
                    {
                        distances.insert(neighbor, *new_distance);
                        prev.insert(neighbor, node);
                        queue.push(NodeWithDistance {
                            node: neighbor,
                            distance: OrderedFloat(*new_distance),
                        });
                    }
                }
            }
        }

        None
    }

    fn distance(&self, node1: i64, node2: i64) -> f64 {
        let coords1 = self.nodes.get(&node1).unwrap();
        let coords2 = self.nodes.get(&node2).unwrap();
        let lat_diff = coords1[0] - coords2[0];
        let lon_diff = coords1[1] - coords2[1];
        (lat_diff.powi(2) + lon_diff.powi(2)).sqrt()
    }
}

pub fn write_csv(graph: &RoadGraph, dir: &Path) -> Result<(), Box<dyn Error>> {
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
    for (id, coords) in graph.nodes.iter() {
        let lat = coords[0];
        let lon = coords[1];
        node_writer.serialize((id, lat, lon))?;
    }

    // Write edges to CSV file
    for (id1, id2) in graph.edges.iter() {
        edge_writer.serialize((id1, id2))?;
    }

    Ok(())
}

pub fn from_pbf(pbf_path: &Path) -> Result<RoadGraph, Box<dyn Error>> {
    // Create a new road graph
    let mut graph = RoadGraph::new();

    // Create an IndexedReader for the PBF file
    let mut reader = IndexedReader::from_path(pbf_path)?;

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

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_road_graph() -> RoadGraph {
        let mut graph = RoadGraph::new();

        graph.add_node(1, 0.0, 0.0);
        graph.add_node(2, 1.0, 1.0);
        graph.add_node(3, 2.0, 2.0);
        graph.add_node(4, 0.0, 2.0);
        graph.add_node(5, 2.0, 0.0);

        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 4);
        graph.add_edge(4, 1);
        graph.add_edge(1, 5);
        graph.add_edge(5, 3);

        graph
    }

    #[test]
    fn test_shortest_path() {
        let graph = create_test_road_graph();

        let path = graph.shortest_path(1, 3);
        assert_eq!(path, Some(vec![1, 2, 3]));

        let path = graph.shortest_path(1, 4);
        assert_eq!(path, Some(vec![1, 2, 3, 4]));

        let path = graph.shortest_path(2, 5);
        assert_eq!(path, Some(vec![2, 3, 4, 1, 5]));

        let path = graph.shortest_path(1, 6);
        assert_eq!(path, None);
    }
}
