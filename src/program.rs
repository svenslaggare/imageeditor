use std::sync::mpsc::Receiver;

use cgmath::{Matrix3, Matrix4, Transform, Matrix, SquareMatrix};

use glfw::{Key, Action, Modifiers};

use crate::command_buffer::{CommandBuffer, Command};
use crate::{editor, ui};
use crate::rendering::shader::Shader;
use crate::rendering::prelude::Position;
use crate::rendering::texture_render::TextureRender;
use crate::editor::image_operation::{ImageSource};
use crate::editor::tools::{Tool, create_tools, Tools};
use crate::rendering::text_render::TextRender;
use crate::rendering::solid_rectangle_render::SolidRectangleRender;
use crate::rendering::ShaderAndRender;

pub struct Renders {
    pub texture_render: ShaderAndRender<TextureRender>,
    pub text_render: ShaderAndRender<TextRender>,
    pub solid_rectangle_render: ShaderAndRender<SolidRectangleRender>,
}

impl Renders {
    pub fn new() -> Renders {
        Renders {
            texture_render: ShaderAndRender::new(
                Shader::new("content/shaders/texture.vs", "content/shaders/texture.fs", None).unwrap(),
                TextureRender::new()
            ),
            solid_rectangle_render: ShaderAndRender::new(
                Shader::new("content/shaders/solid_rectangle.vs", "content/shaders/solid_rectangle.fs", None).unwrap(),
                SolidRectangleRender::new()
            ),
            text_render: ShaderAndRender::new(
                Shader::new("content/shaders/text.vs", "content/shaders/text.fs", None).unwrap(),
                TextRender::new()
            ),
        }
    }
}

pub struct Program {
    renders: Renders,
    command_buffer: CommandBuffer,
    editor: editor::Editor,
    ui_manager: ui::Manager,
    tools: Vec<Box<dyn Tool>>,
    active_tool: Tools,
    preview_image: editor::Image,
}

impl Program {
    pub fn new(editor: editor::Editor, ui_manager: ui::Manager) -> Program {
        let preview_image = editor.new_image_same();
        let width = editor.image().width();
        let height = editor.image().height();

        let mut program = Program {
            renders: Renders::new(),
            command_buffer: CommandBuffer::new(),
            editor,
            ui_manager,
            tools: create_tools(),
            active_tool: Tools::Pencil,
            preview_image
        };

        program.command_buffer.push(Command::SetImageSize(width, height));
        program.command_buffer.push(Command::SetColor(image::Rgba([255, 0, 0, 255])));
        program.command_buffer.push(Command::SetAlternativeColor(image::Rgba([0, 0, 0, 255])));

        program
    }

    fn image_area_transform(&self) -> Matrix3<f32> {
        cgmath::Matrix3::from_cols(
            cgmath::Vector3::new(1.0, 0.0, 48.0),
            cgmath::Vector3::new(0.0, 1.0, 0.0),
            cgmath::Vector3::new(0.0, 0.0, 1.0),
        ).transpose()
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
                        let image = self.editor.image();
                        encoder.encode(image.get_image().as_ref(), image.width(), image.height(), image::ColorType::RGBA(8)).unwrap();
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

    pub fn update(&mut self,
                  window: &mut glfw::Window,
                  events: &Receiver<(f64, glfw::WindowEvent)>) {
        self.tools[self.active_tool as usize].update();

        for (_, event) in glfw::flush_messages(events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    unsafe {
                        gl::Viewport(0, 0, width, height);
                    }
                }
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                event => {
                    self.process_internal_events(&event);

                    self.ui_manager.process_gui_event(window, &event, &mut self.command_buffer);

                    let transform = self.image_area_transform().invert().unwrap();
                    let op = self.tools[self.active_tool as usize].process_event(
                        window,
                        &event,
                        &transform,
                        &mut self.command_buffer,
                        self.editor.image()
                    );

                    if let Some(op) = op {
                        self.command_buffer.push(Command::ApplyImageOp(op));
                    }
                }
            }
        }

        while let Some(command) = self.command_buffer.pop() {
            match command {
                Command::SetTool(draw_tool) => {
                    if let Some(op) = self.tools[self.active_tool as usize].on_deactivate(&mut self.command_buffer) {
                        self.command_buffer.push(Command::ApplyImageOp(op));
                    }

                    self.active_tool = draw_tool;
                    if let Some(op) = self.tools[self.active_tool as usize].on_active() {
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

    pub fn render(&mut self, transform: &Matrix4<f32>) {
        let origin = self.image_area_transform().transform_point(Position::new(0.0, 0.0));

        self.renders.texture_render.render(
            self.renders.texture_render.shader(),
            &transform,
            self.editor.image().get_texture(),
            origin
        );

        self.ui_manager.render(&self.renders, &transform);

        let changed = {
            self.tools[self.active_tool as usize].preview(self.editor.image(), &mut self.preview_image)
        };

        if changed {
            self.preview_image.clear_cpu();
        }

        self.renders.texture_render.render(
            self.renders.texture_render.shader(),
            &transform,
            self.preview_image.get_texture(),
            origin
        );
    }
}