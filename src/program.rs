use std::sync::mpsc::Receiver;

use cgmath::{Matrix3, Matrix4, Transform, Matrix, SquareMatrix};

use glfw::{Key, Action, Modifiers};

use crate::command_buffer::{CommandBuffer, Command};
use crate::{editor, ui};
use crate::rendering::shader::Shader;
use crate::rendering::prelude::Position;
use crate::rendering::texture_render::TextureRender;
use crate::editor::image_operation::ImageOperation;
use crate::editor::draw_tools::{DrawTool, create_draw_tools, DrawTools};

pub struct Program {
    command_buffer: CommandBuffer,
    editor: editor::Editor,
    ui_manager: ui::Manager,
    draw_tools: Vec<Box<dyn DrawTool>>,
    active_draw_tool: DrawTools,
    preview_image: editor::Image
}

impl Program {
    pub fn new(editor: editor::Editor, ui_manager: ui::Manager) -> Program {
        let preview_image = editor.new_image_same();
        let width = editor.image().width();
        let height = editor.image().height();
        let mut program = Program {
            command_buffer: CommandBuffer::new(),
            editor,
            ui_manager,
            draw_tools: create_draw_tools(),
            active_draw_tool: DrawTools::Pencil,
            preview_image
        };

        program.command_buffer.push(Command::SetImageSize(width, height));

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
            glfw::WindowEvent::Key(Key::F1, _, Action::Press, _) => {
                self.command_buffer.push(Command::SetDrawTool(DrawTools::Pencil));
            }
            glfw::WindowEvent::Key(Key::F2, _, Action::Press, _) => {
                self.command_buffer.push(Command::SetDrawTool(DrawTools::Line));
            }
            glfw::WindowEvent::Key(Key::F3, _, Action::Press, _) => {
                self.command_buffer.push(Command::SetDrawTool(DrawTools::Rectangle));
            }
            glfw::WindowEvent::Key(Key::F4, _, Action::Press, _) => {
                self.command_buffer.push(Command::SetDrawTool(DrawTools::Selection));
            }
            glfw::WindowEvent::Key(Key::F5, _, Action::Press, _) => {
                self.command_buffer.push(Command::SetDrawTool(DrawTools::Effect));
            }
            glfw::WindowEvent::Key(Key::Z, _, Action::Press, Modifiers::Control) => {
                self.command_buffer.push(Command::UndoImageOp);
            }
            glfw::WindowEvent::Key(Key::Y, _, Action::Press, Modifiers::Control) => {
                self.command_buffer.push(Command::RedoImageOp);
            }
            _ => {}
        }
    }

    pub fn update(&mut self,
                  window: &mut glfw::Window,
                  events: &Receiver<(f64, glfw::WindowEvent)>) {
        self.draw_tools[self.active_draw_tool as usize].update();

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

                    self.ui_manager.process_event(window, &event, &mut self.command_buffer);

                    let transform = self.image_area_transform().invert().unwrap();
                    let op = self.draw_tools[self.active_draw_tool as usize].process_event(
                        window,
                        &event,
                        &transform,
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
                Command::SetDrawTool(draw_tool) => {
                    self.active_draw_tool = draw_tool;
                    self.draw_tools[self.active_draw_tool as usize].on_active();
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
                    for draw_tool in &mut self.draw_tools {
                        draw_tool.handle_command(&command);
                    }
                }
            }
        }
    }

    pub fn render(&mut self, shader: &Shader, texture_render: &TextureRender, transform: &Matrix4<f32>) {
        let origin = self.image_area_transform().transform_point(Position::new(0.0, 0.0));

        texture_render.render(
            &shader,
            &transform,
            self.editor.image().get_texture(),
            origin
        );

        self.ui_manager.render(&shader, &texture_render, &transform);

        let changed = {
            self.draw_tools[self.active_draw_tool as usize].preview(self.editor.image(), &mut self.preview_image)
        };

        if changed {
            self.preview_image.clear_cpu();
        }

        texture_render.render(
            &shader,
            &transform,
            self.preview_image.get_texture(),
            origin
        );
    }
}