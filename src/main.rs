use crate::knn::PointCloud;
use rand::Rng;
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

fn dist(a: &V2, b: &V2) -> f64 {
    (a - b).length_squared()
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}

fn main() -> Result<()> {
    let mut rng = rand::thread_rng();
    let mut points: Vec<V2> = Vec::new();
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
        if points
            .iter()
            .cloned()
            .all(|a| (a - point).length() > MIN_SPACING)
        {
            points.push(point);
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
    let start: V2 = Pol {
        a: -TAU / 3.0,
        r: MAZE_RADIUS + TUBE_RADIUS * 10.0,
    }
    .into();
    let end: V2 = Pol {
        a: -2.0 * TAU / 3.0,
        r: MAZE_RADIUS + TUBE_RADIUS * 10.0,
    }
    .into();

    let edges: HashSet<(usize, usize)> = Default::default();
    let mut cloud = PointCloud::new(dist);
    points.iter().for_each(|p| cloud.add_point(p));
    let Some(&(_, &start_point)) = cloud.get_nearest_k(&start, 1).first() else {
        panic!("no start point!?");
    };
    let Some(&(_, &end_point)) = cloud.get_nearest_k(&end, 1).first() else {
        panic!("no start point!?");
    };

    document = document.add(
        Circle::new()
            .set("r", MAZE_RADIUS)
            .set("cx", 0.0)
            .set("cy", 0.0)
            .set("fill", "black"),
    );
    for V2 { x, y } in points {
        document = document.add(
            Circle::new()
                .set("r", TUBE_RADIUS)
                .set("cx", x)
                .set("cy", y)
                .set("fill", "white"),
        );
    }
    document = add_edge(document, start, start_point);
    document = add_edge(document, end, end_point);
    svg::save("image.svg", &document)?;
    Ok(())
}

fn add_edge(document: Document, start: V2, end: V2) -> Document {
    eprintln!("[add_edge] start={start:?} end={end:?}");
    let data = Data::new()
        .move_to((start.x, start.y))
        .line_to((end.x, end.y));
    let path = Path::new()
        .set("fill", "red")
        .set("stroke", "#ff000077")
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
