use std::error::Error;

mod app;
mod triangle;
mod cube;
mod camera;


fn main() -> Result<(), Box<dyn Error>> {
    app::run()?;
    Ok(())
}
