use std::sync::mpsc::Receiver;
use std::cell::RefCell;
use std::rc::Rc;

use cgmath::{Matrix3, Matrix4, Transform, Matrix, SquareMatrix};

use glfw::{Key, Action, Modifiers};

use crate::command_buffer::{CommandBuffer, Command};
use crate::{editor, ui};
use crate::rendering::shader::Shader;
use crate::rendering::prelude::{Position, Rectangle};
use crate::rendering::texture_render::TextureRender;
use crate::editor::image_operation::{ImageSource, ImageOperation};
use crate::editor::tools::{Tool, create_tools, Tools, EditorWindow};
use crate::rendering::text_render::TextRender;
use crate::rendering::solid_rectangle_render::SolidRectangleRender;
use crate::rendering::ShaderAndRender;
use crate::rendering::texture::Texture;
use crate::rendering::font::Font;
use crate::rendering::rectangle_render::RectangleRender;

pub const SIDE_PANEL_WIDTH: u32 = 70;
pub const TOP_PANEL_HEIGHT: u32 = 40;

pub struct Program {
    renders: Renders,
    pub command_buffer: CommandBuffer,
    editor: editor::Editor,
    ui_manager: ui::Manager,
    tools: Vec<Box<dyn Tool>>,
    active_tool: Tools,
    background_transparent_image: image::RgbaImage,
    background_transparent_texture: Texture,
    preview_image: editor::Image,
    zoom: f32,
    window_width: u32,
    window_height: u32,
    view_width: u32,
    view_height: u32,
    view_x: f32,
    view_y: f32
}

impl Program {
    pub fn new(view_width: u32,
               view_height: u32,
               editor: editor::Editor,
               ui_manager: ui::Manager) -> Program {
        let preview_image = editor.new_image_same();
        let width = editor.image().width();
        let height = editor.image().height();

        let background_transparent_image = image::open("content/ui/checkerboard.png").unwrap().into_rgba();
        let background_transparent_texture = Texture::from_image(&background_transparent_image);
        background_transparent_texture.bind();
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        }

        let renders = Renders::new();
        let tools = create_tools(&renders);

        let mut program = Program {
            renders,
            command_buffer: CommandBuffer::new(),
            editor,
            ui_manager,
            tools,
            active_tool: Tools::Pencil,
            background_transparent_image,
            background_transparent_texture,
            preview_image,
            zoom: 1.0,
            window_width: view_width,
            window_height: view_height,
            view_width: view_width - SIDE_PANEL_WIDTH,
            view_height: view_height - TOP_PANEL_HEIGHT,
            view_x: 0.0,
            view_y: 0.0
        };

        program.command_buffer.push(Command::SetImageSize(width, height));
        program.command_buffer.push(Command::SetColor(image::Rgba([255, 0, 0, 255])));
        program.command_buffer.push(Command::SetAlternativeColor(image::Rgba([0, 0, 0, 255])));

        program
    }

    pub fn update(&mut self,
                  window: &mut dyn EditorWindow,
                  events: impl Iterator<Item=glfw::WindowEvent>) {
        self.tools[self.active_tool.index()].update();

        for event in events {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    unsafe {
                        gl::Viewport(0, 0, width, height);
                        self.window_width = width as u32;
                        self.window_height = height as u32;
                        self.update_view_size();
                    }
                }
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                glfw::WindowEvent::Key(Key::Left, _, Action::Press | Action::Repeat, _) => {
                    if self.sees_not_whole() {
                        self.view_x -= 10.0;
                    }
                }
                glfw::WindowEvent::Key(Key::Right, _, Action::Press | Action::Repeat, _) => {
                    if self.sees_not_whole() {
                        self.view_x += 10.0;
                    }
                }
                glfw::WindowEvent::Key(Key::Up, _, Action::Press | Action::Repeat, _) => {
                    if self.sees_not_whole() {
                        self.view_y -= 10.0;
                    }
                }
                glfw::WindowEvent::Key(Key::Down, _, Action::Press | Action::Repeat, _) => {
                    if self.sees_not_whole() {
                        self.view_y += 10.0;
                    }
                }
                glfw::WindowEvent::Scroll(_, y) => {
                    let prev_zoom = self.zoom;
                    self.zoom = (self.zoom + y as f32 * 0.1).max(0.3);

                    if self.zoom < 1.0 || prev_zoom < 1.0 {
                        self.view_x = self.editor.image().width() as f32 * 0.5 - (self.view_width as f32 / self.zoom) * 0.5;
                        self.view_y = self.editor.image().height() as f32 * 0.5 - (self.view_height as f32 / self.zoom) * 0.5;
                    }
                }
                glfw::WindowEvent::Key(Key::Num0, _, Action::Press, Modifiers::Control) => {
                    self.view_x = 0.0;
                    self.view_y = 0.0;
                    self.zoom = 1.0;
                }
                glfw::WindowEvent::Key(Key::Tab, _, Action::Press, _) => {
                    self.editor.next_layer();
                }
                event => {
                    self.process_internal_events(&event);

                    self.ui_manager.process_gui_event(window, &event, &mut self.command_buffer);

                    let image_area_transform = self.image_area_transform(false).invert().unwrap();
                    let image_area_rectangle = self.image_area_rectangle();
                    let op = self.tools[self.active_tool.index()].process_gui_event(
                        window,
                        &event,
                        &image_area_transform,
                        &image_area_rectangle,
                        &mut self.command_buffer,
                        self.editor.active_image()
                    );

                    if let Some(op) = op {
                        self.command_buffer.push(Command::ApplyImageOp(op));
                    }
                }
            }
        }

        while let Some(command) = self.command_buffer.pop() {
            match command {
                Command::SwitchImage(image) => {
                    self.editor.set_image(editor::Image::new(image));
                    self.preview_image = self.editor.new_image_same();
                    self.update_view_size();
                }
                Command::SetTool(tool) => {
                    if tool.index() != self.active_tool.index() {
                        if let Some(op) = self.tools[self.active_tool.index()].on_deactivate(&mut self.command_buffer) {
                            self.command_buffer.push(Command::ApplyImageOp(op));
                        }
                    }

                    self.active_tool = tool;
                    if let Some(op) = self.tools[self.active_tool.index()].on_active(window, tool) {
                        self.command_buffer.push(Command::ApplyImageOp(op));
                    }

                    self.preview_image.clear_cpu();
                    self.preview_image.update_operation();
                }
                Command::ApplyImageOp(op) => {
                    self.editor.apply_op(op);
                }
                Command::UndoImageOp => {
                    self.editor.undo_op();
                }
                Command::RedoImageOp => {
                    self.editor.redo_op();
                }
                command => {
                    for draw_tool in &mut self.tools {
                        draw_tool.handle_command(&command);
                    }

                    self.ui_manager.process_command(&command);
                }
            }
        }
    }

    fn process_internal_events(&mut self, event: &glfw::WindowEvent) {
        match event {
            glfw::WindowEvent::Key(Key::Z, _, Action::Press, Modifiers::Control) => {
                self.command_buffer.push(Command::UndoImageOp);
            }
            glfw::WindowEvent::Key(Key::Y, _, Action::Press, Modifiers::Control) => {
                self.command_buffer.push(Command::RedoImageOp);
            }
            glfw::WindowEvent::Key(Key::S, _, Action::Press, Modifiers::Control) => {
                match std::fs::File::create("output.png") {
                    Ok(file) => {
                        let writer = std::io::BufWriter::new(file);
                        let encoder = image::png::PNGEncoder::new(writer);
                        let image = self.editor.active_image();
                        encoder.encode(
                            image.get_image().as_ref(),
                            image.width(),
                            image.height(),
                            image::ColorType::RGBA(8)
                        ).unwrap();
                        println!("Saved image.");
                    }
                    Err(error) => {
                        println!("Failed to save due to: {}.", error);
                    }
                }
            }
            _ => {}
        }
    }

    fn sees_not_whole(&self) -> bool {
        let ratio_x = (self.editor.image().width() as f32 * self.zoom) / self.view_width as f32;
        let ratio_y = (self.editor.image().height() as f32 * self.zoom) / self.view_height as f32;
        ratio_x > 1.0 || ratio_y > 1.0
    }

    pub fn render(&mut self, transform: &Matrix4<f32>) {
        let image_area_transform = self.image_area_transform_matrix4(true);

        let (background_transparent_start, background_transparent_width, background_transparent_height) = self.calculate_background_transparent_rectangle();
        if background_transparent_width > 0.0 && background_transparent_height > 0.0 {
            self.renders.texture_render.render_sized(
                self.renders.texture_render.shader(),
                &(transform * image_area_transform),
                &self.background_transparent_texture,
                background_transparent_start,
                background_transparent_width,
                background_transparent_height,
                Rectangle::new(0.0, 0.0, background_transparent_width, background_transparent_height)
            );
        }

        let image_crop_rectangle = Rectangle::new(
            self.view_x,
            self.view_y,
            self.editor.image().width() as f32 / self.zoom,
            self.editor.image().height() as f32 / self.zoom
        );

        for image in self.editor.image().layers() {
            self.renders.texture_render.render_sub(
                self.renders.texture_render.shader(),
                &(transform * image_area_transform),
                image.get_texture(),
                Position::new(0.0, 0.0),
                self.zoom,
                Some(image_crop_rectangle.clone())
            );
        }

        self.ui_manager.render(&self.renders, &transform);

        let changed = {
            self.tools[self.active_tool.index()].preview(self.editor.active_image(), &mut self.preview_image)
        };

        if changed {
            self.preview_image.clear_cpu();
        }

        self.renders.texture_render.render_sub(
            self.renders.texture_render.shader(),
            &(transform * image_area_transform),
            self.preview_image.get_texture(),
            Position::new(0.0, 0.0),
            self.zoom,
            Some(image_crop_rectangle.clone())
        );

        let image_area_transform_full = self.image_area_transform_matrix4(false);
        self.tools[self.active_tool.index()].render(
            &self.renders,
            &transform,
            &image_area_transform_full,
            self.editor.active_image()
        );
    }

    fn calculate_background_transparent_rectangle(&self) -> (Position, f32, f32) {
        let mut background_transparent_start = Position::new(
            -self.view_x * self.zoom,
            -self.view_y * self.zoom
        );

        let mut background_transparent_end = Position::new(
            background_transparent_start.x + self.editor.image().width() as f32 * self.zoom,
            background_transparent_start.y + self.editor.image().height() as f32 * self.zoom
        );

        if background_transparent_start.x < 0.0 {
            background_transparent_start.x = 0.0;
        }

        if background_transparent_start.y < 0.0 {
            background_transparent_start.y = 0.0;
        }

        background_transparent_end.x = background_transparent_end.x.min(self.view_width as f32);
        background_transparent_end.y = background_transparent_end.y.min(self.view_height as f32);

        let background_transparent_width = background_transparent_end.x - background_transparent_start.x;
        let background_transparent_height = background_transparent_end.y - background_transparent_start.y;

        (background_transparent_start, background_transparent_width, background_transparent_height)
    }

    fn update_view_size(&mut self) {
        self.view_width = (self.window_width - SIDE_PANEL_WIDTH).min(self.editor.image().width());
        self.view_height = (self.window_height - TOP_PANEL_HEIGHT).min(self.editor.image().height());
    }

    fn image_area_transform(&self, only_origin: bool) -> Matrix3<f32> {
        let origin_transform = cgmath::Matrix3::from_cols(
            cgmath::Vector3::new(1.0, 0.0, SIDE_PANEL_WIDTH as f32),
            cgmath::Vector3::new(0.0, 1.0, TOP_PANEL_HEIGHT as f32),
            cgmath::Vector3::new(0.0, 0.0, 1.0),
        ).transpose();

        if only_origin {
            origin_transform
        } else {
            origin_transform
                *
                cgmath::Matrix3::from_cols(
                    cgmath::Vector3::new(self.zoom, 0.0, 0.0),
                    cgmath::Vector3::new(0.0, self.zoom, 0.0),
                    cgmath::Vector3::new(0.0, 0.0, 1.0),
                ).transpose()
                *
                cgmath::Matrix3::from_cols(
                    cgmath::Vector3::new(1.0, 0.0, -self.view_x),
                    cgmath::Vector3::new(0.0, 1.0, -self.view_y),
                    cgmath::Vector3::new(0.0, 0.0, 1.0),
                ).transpose()
        }
    }

    fn image_area_transform_matrix4(&self, only_origin: bool) -> Matrix4<f32> {
        let image_area_transform = self.image_area_transform(only_origin).transpose();

        cgmath::Matrix4::from_cols(
            cgmath::Vector4::new(image_area_transform.x.x, image_area_transform.x.y, 0.0, image_area_transform.x.z),
            cgmath::Vector4::new(image_area_transform.y.x, image_area_transform.y.y, 0.0, image_area_transform.y.z),
            cgmath::Vector4::new(0.0, 0.0, 1.0, 0.0),
            cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0)
        ).transpose()
    }

    fn image_area_rectangle(&self) -> Rectangle {
        let origin_transform = self.image_area_transform(true);
        let x = origin_transform.z.x;
        let y = origin_transform.z.y;
        Rectangle::new(x, y, self.editor.image().width() as f32 + x, self.editor.image().height() as f32 + y)
    }
}

pub struct Renders {
    pub texture_render: ShaderAndRender<TextureRender>,
    pub rectangle_render: ShaderAndRender<RectangleRender>,
    pub solid_rectangle_render: ShaderAndRender<SolidRectangleRender>,
    pub text_render: ShaderAndRender<TextRender>,
    pub ui_font: Rc<RefCell<Font>>,
}

impl Renders {
    pub fn new() -> Renders {
        Renders {
            texture_render: ShaderAndRender::new(
                Shader::new("content/shaders/texture.vs", "content/shaders/texture.fs", None).unwrap(),
                TextureRender::new()
            ),
            rectangle_render: ShaderAndRender::new(
                Shader::new("content/shaders/rectangle.vs", "content/shaders/rectangle.fs", None).unwrap(),
                RectangleRender::new()
            ),
            solid_rectangle_render: ShaderAndRender::new(
                Shader::new("content/shaders/solid_rectangle.vs", "content/shaders/solid_rectangle.fs", None).unwrap(),
                SolidRectangleRender::new()
            ),
            text_render: ShaderAndRender::new(
                Shader::new("content/shaders/text.vs", "content/shaders/text.fs", None).unwrap(),
                TextRender::new()
            ),
            ui_font: Rc::new(RefCell::new(Font::new("content/fonts/NotoMono-Regular.ttf", 16).unwrap()))
        }
    }
}