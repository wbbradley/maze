use crate::seg::*;
use hex_color::HexColor;
use rand::Rng;
use std::collections::HashSet;
use std::f64::consts::{PI, TAU};
use std::time::{Duration, Instant};
use svg::node::element::path::Data;
use svg::node::element::{Circle, Path};
use svg::Document;
use vector2d::Vector2D;

mod seg;

type V2 = Vector2D<f64>;
type Result<T> = std::result::Result<T, Error>;
const MAZE_RADIUS: f64 = 500.0;
const TUBE_RADIUS: f64 = 0.005 * MAZE_RADIUS;
const MIN_SPACING: f64 = TUBE_RADIUS * 3.5;
const TUBE_SHRINK: f64 = 0.15;
const COMPUTE_TIME: Duration = Duration::from_secs(2);

#[derive(Debug)]
pub struct Error(String);

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Edge(pub Index, pub Index);

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}

type Index = usize;

#[derive(Debug, Copy, Clone)]
struct Node {
    point: V2,
    index: Index,
}

fn main() -> Result<()> {
    let mut rng = rand::thread_rng();
    let mut nodes: Vec<Node> = Vec::new();
    let start: Node = Node {
        index: 0,
        point: Pol {
            a: -TAU / 2.0,
            r: MAZE_RADIUS + TUBE_RADIUS * 10.0,
        }
        .into(),
    };
    nodes.push(start);
    let end: Node = Node {
        index: 1,
        point: Pol {
            a: 0.0,
            r: MAZE_RADIUS + TUBE_RADIUS * 10.0,
        }
        .into(),
    };
    nodes.push(end);
    let start_compute = Instant::now();
    let mut tries = 0;
    while Instant::now() - start_compute < COMPUTE_TIME {
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
                index: nodes.len(),
            });
        }
    }
    eprintln!("scanned {} points, found {} points.", tries, nodes.len());
    let mut document = Document::new().set(
        "viewBox",
        (
            -MAZE_RADIUS,
            -MAZE_RADIUS,
            2.0 * MAZE_RADIUS,
            2.0 * MAZE_RADIUS,
        ),
    );

    let mut visited: HashSet<Index> = Default::default();
    let mut edges: HashSet<Edge> = Default::default();
    let start_point: Node = get_nearest_k(&nodes, start, 2)[0];
    let end_point: Node = get_nearest_k(&nodes, end, 2)[1];
    dfs(
        &mut rng,
        start.point - V2 { x: 10.0, y: 0.0 },
        start,
        &mut edges,
        &mut visited,
        &nodes,
    );
    eprintln!("created {} edges", edges.len());
    document = document.add(
        Circle::new()
            .set("r", MAZE_RADIUS)
            .set("cx", 0.0)
            .set("cy", 0.0)
            .set("fill", "blue"),
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
                .set("fill", HexColor::random_rgba().to_string().as_str()), // "white"),
        );
    }
    let drawn_nodes: HashSet<Index> = HashSet::new();

    for Edge(a, b) in edges {
        let color = HexColor::random_rgb().to_string();
        document = add_edge(document, nodes[a].point, nodes[b].point, color.as_ref()); // "white");
        if !drawn_nodes.contains(&a) {
            document = document.add(
                Circle::new()
                    .set("r", TUBE_RADIUS)
                    .set("cx", nodes[a].point.x)
                    .set("cy", nodes[a].point.y)
                    .set("fill", "white"),
            );
        }
        if !drawn_nodes.contains(&b) {
            document = document.add(
                Circle::new()
                    .set("r", TUBE_RADIUS)
                    .set("cx", nodes[b].point.x)
                    .set("cy", nodes[b].point.y)
                    .set("fill", "white"),
            );
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
    prior: V2,
    current: Node,
    edges: &mut HashSet<Edge>,
    visited: &mut HashSet<Index>,
    nodes: &Vec<Node>,
) {
    let cur_vec_angle = (current.point - prior).normalise().angle();
    let mut nearest_nodes = get_nearest_k(nodes, current, 8);
    // nearest_nodes.shuffle(rng);
    for node in nearest_nodes {
        if !visited.contains(&node.index) {
            visited.insert(node.index);
            let edge = Edge(current.index, node.index);
            let edge_vec = (node.point - current.point).normalise();
            let diff = radian_diff(edge_vec.angle(), cur_vec_angle);
            // if diff < PI * 0.4 { continue; }
            if edge_intersects(edge, edges, nodes) {
                continue;
            }

            edges.insert(edge);
            dfs(rng, current.point, node, edges, visited, nodes);
        }
    }
}
fn radian_diff(a: f64, b: f64) -> f64 {
    let mut d = a - b;
    if d > PI {
        d -= TAU;
    } else if d < -PI {
        d += TAU;
    }
    d.abs()
}
fn edge_intersects(edge: Edge, edges: &HashSet<Edge>, nodes: &[Node]) -> bool {
    let Edge(a, b) = edge;
    for &Edge(c, d) in edges {
        if intersection_with_width(
            nodes[a].point,
            nodes[b].point,
            nodes[c].point,
            nodes[d].point,
            TUBE_RADIUS,
            TUBE_SHRINK,
        ) {
            return true;
        }
    }
    false
}
fn add_edge(document: Document, start: V2, end: V2, color: &str) -> Document {
    // eprintln!("[add_edge] start={start:?} end={end:?}");
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
