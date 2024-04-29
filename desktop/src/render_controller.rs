use std::{rc::Rc, sync::Arc};

use anyhow::anyhow;
use ruffle_render_wgpu::{
    backend::request_adapter_and_device, clap::PowerPreference, descriptors::Descriptors,
};
use url::Url;
use winit::window::Window;

use crate::{
    movie::{MovieView, MovieViewRenderer},
    player::PlayerController,
};

pub struct RenderController {
    window: Rc<Window>,
    descriptors: Arc<Descriptors>,
    movie_view_renderer: Arc<MovieViewRenderer>,
    size: winit::dpi::PhysicalSize<u32>,
}

impl RenderController {
    pub fn new(window: Rc<Window>) -> anyhow::Result<Self> {
        let (instance, backend) = create_wgpu_instance()?;
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(window.as_ref())?)
        }?;
        let (adapter, device, queue) = futures::executor::block_on(request_adapter_and_device(
            backend,
            &instance,
            Some(&surface),
            PowerPreference::High.into(),
            None,
        ))
        .map_err(|e| anyhow!("请求适配器和设备失败: {:?}", e))?;
        let adapter_info = adapter.get_info();
        dbg!(adapter_info);
        // tracing::info!(
        //     "Using graphics API {} on {} (type: {:?})",
        //     adapter_info.backend.to_str(),
        //     adapter_info.name,
        //     adapter_info.device_type
        // );
        let surface_format = surface
            .get_capabilities(&adapter)
            .formats
            .first()
            .cloned()
            .expect("至少有一个支持表面格式");
        let size = window.inner_size();
        let descriptors = Descriptors::new(instance, adapter, device, queue);
        let movie_view_renderer = Arc::new(MovieViewRenderer::new(
            &descriptors.device,
            surface_format,
            size.height,
            window.scale_factor(),
        ));
        Ok(RenderController {
            window,
            descriptors: Arc::new(descriptors),
            movie_view_renderer,
            size,
        })
    }

    pub fn descriptors(&self) -> Arc<Descriptors> {
        self.descriptors.clone()
    }

    pub fn create_movie(&mut self, player_controller: &mut PlayerController, movie_url: Url) {
        let movie_view = MovieView::new(
            self.movie_view_renderer.clone(),
            &self.descriptors.device,
            self.size.width,
            self.size.height,
        );
        player_controller.create(&movie_url, movie_view);
    }
}

fn create_wgpu_instance() -> anyhow::Result<(wgpu::Instance, wgpu::Backends)> {
    for backend in wgpu::Backends::all() {
        if let Some(instance) = try_wgpu_backend(backend) {
            // tracing::info!(
            //     "渲染后端 {}",
            //     format_list(&get_backend_names(backend), "and")
            // );
            return Ok((instance, backend));
        }
    }
    Err(anyhow!("没有找到可用的渲染后端"))
}
fn try_wgpu_backend(backend: wgpu::Backends) -> Option<wgpu::Instance> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: backend,
        flags: wgpu::InstanceFlags::default().with_env(),
        ..Default::default()
    });
    if instance.enumerate_adapters(backend).is_empty() {
        None
    } else {
        Some(instance)
    }
}
