use std::{path::{Path, PathBuf}, rc::Rc};

use crate::{player::PlayerController, render_controller::RenderController};
use anyhow::Error;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};
const MOVIE_CLIP_URL: &str = "desktop/swf_files/spirit2471src.swf";

pub struct App {
    movie_url: PathBuf,
    player_controller: PlayerController,
    window: Rc<Window>,
    event_loop: Option<EventLoop<()>>,
}

impl App {
    pub fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new()
            .with_title("swf-player")
            .build(&event_loop)
            .unwrap();
        let window: Rc<Window> = Rc::new(window);
        let mut render_controller = RenderController::new(window.clone()).unwrap();
        let mut player_controller =
            PlayerController::new(window.clone(), render_controller.descriptors());
        let movie_url = Path::new(MOVIE_CLIP_URL).to_path_buf();
        render_controller.create_movie(&mut player_controller, movie_url.clone());


        Self {
            movie_url,
            player_controller,
            window,
            event_loop: Some(event_loop),
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        // events loop
        let event_loop = self.event_loop.take().expect("App 已经在运行了");
        event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        dbg!("CloseRequested");
                        elwt.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        dbg!("RedrawRequested");
                        if let Some(mut player) = self.player_controller.get() {
                            player.render();
                        }
                    }
                    WindowEvent::Resized(_) => {
                        dbg!("Resized");
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    // 自己调用request_redraw()方法，不需要等待系统调用
                    // dbg!("AboutToWait");
                    // 应用程序应该总是重新绘制窗口
                    if let Some(mut player) = self.player_controller.get() {
                        // player.tick();
                    }
                }

                _ => {}
            }
        })?;
        Ok(())
    }
}
