use std::{path::Path, rc::Rc};

use crate::{player::PlayerController, render_controller::RenderController};
use anyhow::Error;
use url::Url;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};
const MOVIE_CLIP_URL: &str = "desktop/swf_files/spirit2471src.swf";

pub struct App {
    movie_url: Url,
    player_controller: PlayerController,
    window: Rc<Window>,
    event_loop: Option<EventLoop<()>>,
}

impl App {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().expect("获取当前目录失败");
        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new()
            .with_title("swf-player")
            .build(&event_loop)
            .unwrap();
        let window = Rc::new(window);
        let mut render_controller = RenderController::new(window.clone()).unwrap();
        let mut player_controller =
            PlayerController::new(window.clone(), render_controller.descriptors());
        let movie_url = Url::from_file_path(current_dir.join(MOVIE_CLIP_URL)).unwrap();

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
                    // window.request_redraw();
                }

                _ => {}
            }
        })?;
        Ok(())
    }
}
