use crate::knn::PointCloud;
use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::f64::consts::TAU;
use std::time::{Duration, Instant};
use svg::node::element::path::Data;
use svg::node::element::{Circle, Path};
use svg::Document;
use vector2d::Vector2D;

mod heap;
mod knn;

type V2 = Vector2D<f64>;
type Result<T> = std::result::Result<T, Error>;
const MAZE_RADIUS: f64 = 500.0;
const MIN_SPACING: f64 = 2.0 / 50.0 * MAZE_RADIUS;
const TUBE_RADIUS: f64 = 0.01 * MAZE_RADIUS;
const COMPUTE_TIME: Duration = Duration::from_secs(3);

#[derive(Debug)]
pub struct Error(String);

fn dist(a: &Node, b: &Node) -> f64 {
    (a.point - b.point).length_squared()
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd)]
enum Index {
    Start,
    Index(usize),
    End,
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

    let mut _visited: HashSet<Index> = Default::default();
    let mut _edges: HashSet<(Index, Index)> = Default::default();
    let mut cloud = PointCloud::new(dist);
    nodes.iter().for_each(|node| cloud.add_point(node));
    let Some(&(_, &start_point)) = cloud.get_nearest_k(&start, 1).first() else {
        panic!("no start point!?");
    };
    let Some(&(_, &end_point)) = cloud.get_nearest_k(&end, 1).first() else {
        panic!("no start point!?");
    };
    // dfs(&mut edges, &mut visited)
    document = document.add(
        Circle::new()
            .set("r", MAZE_RADIUS)
            .set("cx", 0.0)
            .set("cy", 0.0)
            .set("fill", "black"),
    );
    for Node {
        point: V2 { x, y }, ..
    } in nodes
    {
        document = document.add(
            Circle::new()
                .set("r", TUBE_RADIUS)
                .set("cx", x)
                .set("cy", y)
                .set("fill", "white"),
        );
    }
    document = add_edge(document, start.point, start_point.point, "green");
    document = add_edge(document, end.point, end_point.point, "red");
    svg::save("image.svg", &document)?;
    Ok(())
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
