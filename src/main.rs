use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::f64::consts::TAU;
use std::time::{Duration, Instant};
use svg::node::element::path::Data;
use svg::node::element::{Circle, Path};
use svg::Document;
use vector2d::Vector2D;

type V2 = Vector2D<f64>;
type Result<T> = std::result::Result<T, Error>;
const MAZE_RADIUS: f64 = 500.0;
const MIN_SPACING: f64 = 2.0 / 50.0 * MAZE_RADIUS;
const TUBE_RADIUS: f64 = 0.01 * MAZE_RADIUS;
const COMPUTE_TIME: Duration = Duration::from_secs(3);

#[derive(Debug)]
pub struct Error(String);

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
enum Index {
    Start,
    Index(usize),
    End,
}
impl std::cmp::PartialOrd for Index {
    fn partial_cmp(&self, other: &Index) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl std::cmp::Ord for Index {
    fn cmp(&self, rhs: &Self) -> Ordering {
        match (self, rhs) {
            (&Index::Start, _) => Ordering::Less,
            (&Index::End, _) => Ordering::Greater,
            (_, &Index::Start) => Ordering::Greater,
            (_, &Index::End) => Ordering::Less,
            (&Index::Index(a), Index::Index(b)) => a.cmp(b),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Node {
    point: V2,
    index: Index,
}

fn main() -> Result<()> {
    let mut rng = rand::thread_rng();
    let mut nodes: Vec<Node> = Vec::new();
    let start = Instant::now();
    let mut tries = 0;
    while Instant::now() - start < COMPUTE_TIME {
        let radians: f64 = rng.gen::<f64>() * TAU;
        let radius: f64 = rng.gen::<f64>() * (MAZE_RADIUS - TUBE_RADIUS * (2f64).sqrt() * 2.0);
        let point = V2 {
            x: radians.cos() * radius,
            y: radians.sin() * radius,
        };
        tries += 1;
        if nodes
            .iter()
            .cloned()
            .all(|Node { point: a, .. }| (a - point).length() > MIN_SPACING)
        {
            nodes.push(Node {
                point,
                index: Index::Index(nodes.len()),
            });
        }
    }
    eprintln!("scanned {} points.", tries);
    let mut document = Document::new().set(
        "viewBox",
        (
            -MAZE_RADIUS,
            -MAZE_RADIUS,
            2.0 * MAZE_RADIUS,
            2.0 * MAZE_RADIUS,
        ),
    );
    let start: Node = Node {
        index: Index::Start,
        point: Pol {
            a: -TAU / 2.0,
            r: MAZE_RADIUS + TUBE_RADIUS * 10.0,
        }
        .into(),
    };
    let end: Node = Node {
        index: Index::End,
        point: Pol {
            a: 0.0,
            r: MAZE_RADIUS + TUBE_RADIUS * 10.0,
        }
        .into(),
    };

    let mut visited: HashSet<Index> = Default::default();
    let mut edges: HashSet<(Index, Index)> = Default::default();
    let start_point: Node = get_nearest_k(&nodes, start, 1)[0];
    let end_point: Node = get_nearest_k(&nodes, end, 1)[0];
    dfs(&mut rng, start, &mut edges, &mut visited, &nodes);
    document = document.add(
        Circle::new()
            .set("r", MAZE_RADIUS)
            .set("cx", 0.0)
            .set("cy", 0.0)
            .set("fill", "black"),
    );
    for Node {
        point: V2 { x, y }, ..
    } in nodes.iter()
    {
        document = document.add(
            Circle::new()
                .set("r", TUBE_RADIUS)
                .set("cx", *x)
                .set("cy", *y)
                .set("fill", "white"),
        );
    }
    for (a, b) in edges {
        if let (Index::Index(a), Index::Index(b)) = (a, b) {
            document = add_edge(document, nodes[a].point, nodes[b].point, "white");
        }
    }
    document = add_edge(document, start.point, start_point.point, "white");
    document = add_edge(document, end.point, end_point.point, "white");
    svg::save("image.svg", &document)?;
    Ok(())
}

fn get_nearest_k(nodes: &[Node], cur: Node, k: usize) -> Vec<Node> {
    let mut nodes: Vec<Node> = nodes.to_vec();
    nodes.sort_by(|a, b| {
        let a_dist: f64 = (a.point - cur.point).length_squared();
        let b_dist: f64 = (b.point - cur.point).length_squared();
        a_dist.partial_cmp(&b_dist).unwrap()
    });
    nodes.truncate(k);
    nodes
}

fn dfs(
    rng: &mut impl Rng,
    current: Node,
    edges: &mut HashSet<(Index, Index)>,
    visited: &mut HashSet<Index>,
    nodes: &Vec<Node>,
) {
    //
    let mut nearest_nodes = get_nearest_k(nodes, current, 8);
    nearest_nodes.shuffle(rng);
    for node in nearest_nodes {
        if !visited.contains(&node.index) {
            visited.insert(node.index);
            let (a, b): (&Node, &Node) = if current.index < node.index {
                (&current, &node)
            } else {
                (&node, &current)
            };
            edges.insert((a.index, b.index));
            dfs(rng, node, edges, visited, nodes);
        }
    }
}
fn add_edge(document: Document, start: V2, end: V2, color: &str) -> Document {
    eprintln!("[add_edge] start={start:?} end={end:?}");
    let data = Data::new()
        .move_to((start.x, start.y))
        .line_to((end.x, end.y));
    let path = Path::new()
        .set("fill", color)
        .set("stroke", color)
        .set("stroke-width", TUBE_RADIUS * 2.0)
        .set("d", data);
    document.add(path)
}
#[derive(Debug, Clone, Copy)]
struct Pol {
    pub a: f64,
    pub r: f64,
}

impl From<Pol> for V2 {
    fn from(p: Pol) -> Self {
        Self {
            x: p.a.cos() * p.r,
            y: p.a.sin() * p.r,
        }
    }
}
