use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(String);

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}
fn main() -> Result<()> {
    let data = Data::new()
        .move_to((10, 10))
        .line_by((0, 50))
        .line_by((50, 0))
        .line_by((0, -50))
        .close();
    let path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 3)
        .set("d", data);

    let document = Document::new().set("viewbox", (0, 0, 70, 70)).add(path);
    svg::save("image.svg", &document)?;
    Ok(())
}
