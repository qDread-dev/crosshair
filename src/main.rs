use std::io::Read;

use glium::Surface;
use winit::{platform::windows::WindowBuilderExtWindows, window::WindowBuilder};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop}};
use tray_icon::{menu::{Menu, PhysicalPosition}, TrayIconBuilder, TrayIconEvent};
use std::process;
use std::time::Duration;
use winapi::um::winuser::{GetForegroundWindow, GetWindowRect, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
use winapi::shared::windef::RECT;

#[macro_use]
extern crate glium;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

fn main() {

    // tray icon
    let tray_menu = Menu::new();
    let _tray_icon = TrayIconBuilder::new()
    .with_menu(Box::new(tray_menu))
    .with_tooltip("Crosshair - Right click to close")
    .build()
    .unwrap();

    let event_loop = winit::event_loop::EventLoopBuilder::new().build().expect("event loop building");
    let monitor: Option<winit::monitor::MonitorHandle> = event_loop.primary_monitor();
    let proxy = event_loop.create_proxy();

    // must make the window 1 larger to prevent windows from detecting it as fullscreen
    // if windows detects it as fullscreen transparency is disabled, and background becomes black
    let size = match monitor {
        Some(monitor) => {
            let mut size = monitor.size();
            size.width += 1;
            size.height += 1;
            size
        },
        None => {
            println!("No primary monitor found");
            return;
        }
    };
    let window_builder = WindowBuilder::new()
        .with_transparent(true)
        .with_inner_size(size)
        .with_position(winit::dpi::PhysicalPosition::new(-1, -1))
        // .with_fullscreen(Some(Fullscreen::Borderless(monitor)))
        .with_decorations(false)
        .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
        .with_skip_taskbar(true)
        .with_title("Crosshair");


    let (_window, display) = glium::backend::glutin::SimpleWindowBuilder::new().set_window_builder(window_builder).build(&event_loop);
    let _ = _window.set_cursor_hittest(false);
    let mut frame = display.draw();
    frame.clear_color(0.0, 0.0, 0.0, 0.0);
    // actual crosshair
    let crosshair: Vec<Vertex>;
    
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("crosshair.txt")
        .unwrap();
    
    // Read the file contents into a string
    let mut crosshair_file = String::new();
    file.read_to_string(&mut crosshair_file).unwrap();

    if crosshair_file == "" {
        let num_points = 100; // The number of points to generate
        let radius = 0.01; // The radius of the circle (1 is largest)

        let (height, width) = display.get_framebuffer_dimensions();
        let aspect_ratio = width as f32 / height as f32;

        let circle: Vec<Vertex> = (0..num_points).map(|i: i32| {
            let angle = i as f32 / num_points as f32 * 2.0 * std::f32::consts::PI;
            Vertex { position: [radius * angle.cos() * aspect_ratio, radius * angle.sin()] }
        }).collect();
        crosshair = circle;
    } else {
        let vertex_entries = crosshair_file.split(";");
        dbg!(&vertex_entries);
        let mut vertex_vec: Vec<Vertex> = vec![];
        for entry in vertex_entries {
            let mut vertex = entry.split(",");
            let x = vertex.next().unwrap().parse::<f32>().unwrap();
            let y = vertex.next().unwrap().parse::<f32>().unwrap();
            vertex_vec.push(Vertex { position: [x, y] });
        }
        dbg!(&vertex_vec);
        crosshair = vertex_vec;
    }
    dbg!(&crosshair);

    
    let vertex_buffer = glium::VertexBuffer::new(&display, &crosshair).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);

    let mut vertex_shader_src = r#"
        #version 140

        in vec2 position;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;
    let mut vertex_shader_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("vertex_shader.txt")
        .unwrap();
    
    // Read the file contents into a string
    let mut vertex_shader = String::new();
    vertex_shader_file.read_to_string(&mut vertex_shader).unwrap();
    
    if vertex_shader != "" {
        vertex_shader_src = vertex_shader.as_str();
    }

    let mut fragment_shader_src = r#"
    #version 140

    out vec4 color;

    void main() {
        color = vec4(0.0, 0.97, 1.0, 1.0);
    }
    "#;
    let mut frag_shader_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("frag_shader.txt")
        .unwrap();
    
    // Read the file contents into a string
    let mut frag_shader = String::new();
    frag_shader_file.read_to_string(&mut frag_shader).unwrap();
    if frag_shader != "" {
        fragment_shader_src = frag_shader.as_str();
    }

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);
    target.draw(&vertex_buffer, &indices, &program, &glium::uniform! {}, &Default::default()).unwrap();
    target.finish().unwrap();

    event_loop.run(move |ev, window_target| {
        match ev {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    window_target.exit();
                },
                _ => (),
            },
            _ => (),
        }
        
        // get tray events
        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            println!("{:?}", event);
            std::process::exit(0)
        }
    })
    .unwrap();
   

}