use std::rc::Rc;

use url::Url;
use winit::{
    error::EventLoopError,
    event::{self, Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};
use anyhow::Error;
use crate::{player::PlayerController, render_controller::RenderController};
const MOVIE_CLIP_URL: &str = "desktop/swf_file/swf_files/spirit2471src.swf";

pub struct App {
    movie_clip_url: Url,
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
        let render_controller = RenderController::new(window.clone()).unwrap();
        let player_controller = PlayerController::new(
            window.clone(),
            render_controller.descriptors(),
        );
        Self {
            movie_clip_url: Url::from_file_path(current_dir.join(MOVIE_CLIP_URL)).unwrap(),
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
