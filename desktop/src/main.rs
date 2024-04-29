mod app;
mod movie;
mod player;
mod render_controller;
use app::App;
use anyhow::Error;

fn main()->Result<(),Error> {
    App::new().run()?;
    Ok(())
}
