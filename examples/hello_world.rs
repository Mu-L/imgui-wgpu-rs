use wgpu::winit::{
    Event, WindowEvent, EventsLoop,
    KeyboardInput, VirtualKeyCode, ElementState,
    dpi::LogicalSize,
};
use imgui::*;
use imgui_wgpu::Renderer;
use imgui_winit_support;
use std::time::Instant;

fn main() {
    env_logger::init();
    
    //
    // Set up window and GPU
    //
    
    let mut events_loop = EventsLoop::new();
    let (window, instance, mut size, surface, hidpi_factor) = {
        let instance = wgpu::Instance::new();

        let version = env!("CARGO_PKG_VERSION");

        let window = wgpu::winit::Window::new(&events_loop).unwrap();
        window.set_inner_size(LogicalSize { width: 1280.0, height: 720.0 });
        window.set_title(&format!("imgui-wgpu {}", version));
        let hidpi_factor = window.get_hidpi_factor();
        let size = window
            .get_inner_size()
            .unwrap()
            .to_physical(hidpi_factor);

        let surface = instance.create_surface(&window);

        (window, instance, size, surface, hidpi_factor)
    };

    let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
        power_preference: wgpu::PowerPreference::HighPerformance,
    });

    let mut device = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    });

    //
    // Set up swap chain
    //
    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width: size.width as u32,
        height: size.height as u32,
        present_mode: wgpu::PresentMode::NoVsync,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    //
    // Set up dear imgui
    //
    let mut imgui = imgui::Context::create();
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), &window, imgui_winit_support::HiDpiMode::Default);
    imgui.set_ini_filename(None);

    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    imgui.fonts().add_font(&[
        FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            })
        }
    ]);

    //
    // Set up dear imgui wgpu renderer
    // 
    let clear_color = wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 };
    let mut renderer = Renderer::new(&mut imgui, &mut device, sc_desc.format, Some(clear_color))
        .expect("Failed to initialize renderer");

    let mut last_frame = Instant::now();
    let mut demo_open = true;

    //
    // Event loop
    //
    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            match event {
                // On resize
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    size = window
                        .get_inner_size()
                        .unwrap()
                        .to_physical(hidpi_factor);

                    sc_desc = wgpu::SwapChainDescriptor {
                        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        width: size.width as u32,
                        height: size.height as u32,
                        present_mode: wgpu::PresentMode::NoVsync
                    };

                    swap_chain = device.create_swap_chain(&surface, &sc_desc);
                }
                // On ESC / close
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                        ..
                    },
                    ..
                } |
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    running = false;
                },
                _ => (),
            }

            platform.handle_event(imgui.io_mut(), &window, &event);
        });

        let now = Instant::now();
        let delta = now - last_frame;
        let delta_s = delta.as_micros();
        last_frame = now;

        let frame = swap_chain.get_next_texture();
        platform.prepare_frame(imgui.io_mut(), &window) // step 4
            .expect("Failed to prepare frame");
        let ui = imgui.frame();

        {
            let window = imgui::Window::new(im_str!("Hello world"));
            window
                .size([300.0, 100.0], Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(im_str!("Hello world!"));
                    ui.text(im_str!("This...is...imgui-rs on WGPU!"));
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(im_str!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0],
                        mouse_pos[1]
                    ));
                });

            let window = imgui::Window::new(im_str!("Hello too"));
            window
                .size([400.0, 200.0], Condition::FirstUseEver)
                .position([400.0, 200.0], Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(im_str!("Frametime: {}us", delta_s));
                });

            ui.show_demo_window(&mut demo_open);
        }

        let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        platform.prepare_render(&ui, &window);
        renderer
            .render(ui, size.width, size.height, hidpi_factor, &mut device, &mut encoder, &frame.view)
            .expect("Rendering failed");

        device
            .get_queue()
            .submit(&[encoder.finish()]);
    }
}