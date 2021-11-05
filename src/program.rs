use std::cell::RefCell;
use std::rc::Rc;
use std::path::Path;
use std::ops::DerefMut;
use std::collections::HashMap;

use cgmath::{Matrix3, Matrix4, Matrix, SquareMatrix};

use glfw::{Key, Action, Modifiers};

use crate::command_buffer::{CommandBuffer, Command};
use crate::{editor, ui};
use crate::rendering::shader::Shader;
use crate::rendering::prelude::{Position, Rectangle, Color, Color4, Size};
use crate::rendering::texture_render::TextureRender;
use crate::editor::tools::{Tool, create_tools, Tools, EditorWindow, get_transformed_mouse_position, SelectionSubTool};
use crate::rendering::text_render::{TextRender, TextAlignment};
use crate::rendering::solid_rectangle_render::SolidRectangleRender;
use crate::rendering::ShaderAndRender;
use crate::rendering::texture::Texture;
use crate::rendering::font::Font;
use crate::rendering::rectangle_render::RectangleRender;
use crate::editor::editor::{LayerState, EditorOperation};
use crate::ui::layers::LayersManager;
use crate::editor::LayeredImage;

pub const LEFT_SIDE_PANEL_WIDTH: u32 = 70;
pub const RIGHT_SIDE_PANEL_WIDTH: u32 = 150;
pub const SIDE_PANELS_WIDTH: u32 = LEFT_SIDE_PANEL_WIDTH + RIGHT_SIDE_PANEL_WIDTH;
pub const TOP_PANEL_HEIGHT: u32 = 40;

pub const LAYER_BUFFER: f32 = 5.0;
pub const LAYER_SPACING: f32 = 10.0;

pub struct Program {
    renders: Renders,
    pub command_buffer: CommandBuffer,
    pub editor: editor::Editor,
    ui_manager: ui::Manager,
    layers_manager: LayersManager,
    tools: Vec<Box<dyn Tool>>,
    active_tool: Tools,
    prev_tool: Option<Tools>,
    transparent_background_texture: Texture,
    preview_image: editor::Image,
    zoom: f32,
    window_width: u32,
    window_height: u32,
    view_width: u32,
    view_height: u32,
    view_x: f32,
    view_y: f32,
    pub actions: ProgramActionsManager
}

impl Program {
    pub fn new(view_width: u32,
               view_height: u32,
               editor: editor::Editor,
               ui_manager: ui::Manager) -> Program {
        let preview_image = editor.new_image_same();
        let width = editor.image().width();
        let height = editor.image().height();

        let transparent_background_image = image::open("content/ui/checkerboard.png").unwrap().into_rgba();
        let transparent_background_texture = Texture::from_image(&transparent_background_image);
        transparent_background_texture.bind();
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
            layers_manager: LayersManager::new(),
            tools,
            active_tool: Tools::Pencil,
            prev_tool: None,
            transparent_background_texture,
            preview_image,
            zoom: 1.0,
            window_width: view_width,
            window_height: view_height,
            view_width: view_width - SIDE_PANELS_WIDTH,
            view_height: view_height - TOP_PANEL_HEIGHT,
            view_x: 0.0,
            view_y: 0.0,
            actions: ProgramActionsManager::new()
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
                event => {
                    self.process_internal_events(window, &event);

                    self.ui_manager.process_gui_event(
                        window,
                        &event,
                        &mut self.command_buffer
                    );

                    self.layers_manager.process_gui_event(
                        window,
                        self.window_width - SIDE_PANELS_WIDTH,
                        &event,
                        &mut self.editor
                    );

                    let image_area_transform = self.image_area_transform(false).invert().unwrap();
                    let image_area_rectangle = self.image_area_rectangle();
                    let op = self.tools[self.active_tool.index()].process_gui_event(
                        window,
                        &event,
                        &image_area_transform,
                        &image_area_rectangle,
                        &mut self.command_buffer,
                        self.editor.active_layer()
                    );

                    if let Some(op) = op {
                        self.command_buffer.push(Command::ApplyImageOp(op));
                    }
                }
            }
        }

        self.handle_commands(window);
    }

    fn handle_commands(&mut self, window: &mut dyn EditorWindow) {
        while let Some(command) = self.command_buffer.pop() {
            match command {
                Command::NewImage(width, height) => {
                    let image = image::RgbaImage::new(width, height);
                    self.editor.apply_editor_op(EditorOperation::SetImage(LayeredImage::from_rgba(image)));
                    self.image_size_changed();
                }
                Command::SwitchImage(image) => {
                    self.editor.apply_editor_op(EditorOperation::SetImage(LayeredImage::from_rgba(image)));
                    self.image_size_changed();
                }
                Command::SetTool(tool) => {
                    self.switch_tool(window, tool);
                }
                Command::SwitchToPrevTool => {
                    if let Some(tool) = self.prev_tool.take() {
                        self.switch_tool(window, tool);
                    }
                },
                Command::ApplyImageOp(op) => {
                    self.editor.apply_image_op(op);
                }
                Command::UndoImageOp => {
                    self.editor.undo_op();
                    self.update_view_size();
                }
                Command::RedoImageOp => {
                    self.editor.redo_op();
                    self.update_view_size();
                }
                Command::NewLayer => {
                    self.editor.add_layer();
                }
                Command::DuplicateLayer => {
                    self.editor.duplicate_active_layer();
                }
                Command::DeleteLayer => {
                    self.editor.delete_active_layer();
                }
                Command::ResizeImage(new_width, new_height) => {
                    let mut image = self.editor.image().clone();
                    image.resize(new_width, new_height);

                    self.editor.apply_editor_op(EditorOperation::SetImage(image));
                    self.image_size_changed();
                }
                Command::ResizeCanvas(new_width, new_height) => {
                    let mut image = self.editor.image().clone();
                    image.resize_canvas(new_width, new_height);

                    self.editor.apply_editor_op(EditorOperation::SetImage(image));
                    self.image_size_changed();
                }
                command => {
                    if let Command::SelectAll = command {
                        self.switch_tool(window, Tools::Selection(SelectionSubTool::Select));
                    }

                    for draw_tool in &mut self.tools {
                        draw_tool.handle_command(self.editor.active_layer(), &command);
                    }

                    self.ui_manager.process_command(&command);
                }
            }
        }
    }

    fn switch_tool(&mut self, window: &mut dyn EditorWindow, tool: Tools) {
        if tool.index() != self.active_tool.index() {
            self.prev_tool = Some(self.active_tool);

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

    fn process_internal_events(&mut self, _window: &mut dyn EditorWindow, event: &glfw::WindowEvent) {
        match event {
            glfw::WindowEvent::Key(Key::Z, _, Action::Press, Modifiers::Control) => {
                self.command_buffer.push(Command::UndoImageOp);
            }
            glfw::WindowEvent::Key(Key::Y, _, Action::Press, Modifiers::Control) => {
                self.command_buffer.push(Command::RedoImageOp);
            }
            glfw::WindowEvent::Key(Key::S, _, Action::Press, Modifiers::Control) => {
                match self.editor.image().save(Path::new("output.png")) {
                    Ok(()) => {
                        println!("Saved image.");
                    }
                    Err(err) => {
                        println!("Failed to save due to: {}.", err);
                    }
                }
            }
            glfw::WindowEvent::Key(Key::O, _, Action::Press, Modifiers::Control) => {
                self.actions.trigger(ProgramActions::OpenImage);
            }
            glfw::WindowEvent::Key(Key::S, _, Action::Press, modifier) => {
                if modifier == &(Modifiers::Control | Modifiers::Shift) {
                    self.actions.trigger(ProgramActions::SaveImageAs);
                }
            }
            glfw::WindowEvent::Key(Key::R, _, Action::Press, Modifiers::Control) => {
                self.actions.trigger(ProgramActions::ResizeImage);
            }
            glfw::WindowEvent::Key(Key::R, _, Action::Press, modifier) => {
                if modifier == &(Modifiers::Control | Modifiers::Shift) {
                    self.actions.trigger(ProgramActions::ResizeCanvas);
                }
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
                self.zoom = (self.zoom + *y as f32 * 0.1).max(0.3);

                if self.zoom < 1.0 || prev_zoom < 1.0 {
                    self.view_x = self.editor.image().width() as f32 * 0.5 - (self.view_width as f32 / self.zoom) * 0.5;
                    self.view_y = self.editor.image().height() as f32 * 0.5 - (self.view_height as f32 / self.zoom) * 0.5;
                }

                self.update_view_size();
            }
            glfw::WindowEvent::Key(Key::Num0, _, Action::Press, Modifiers::Control) => {
                self.view_x = 0.0;
                self.view_y = 0.0;
                self.zoom = 1.0;
                self.update_view_size();
            }
            _ => {}
        }
    }

    fn sees_not_whole(&self) -> bool {
        let ratio_x = (self.editor.image().width() as f32 * self.zoom) / self.view_width as f32;
        let ratio_y = (self.editor.image().height() as f32 * self.zoom) / self.view_height as f32;
        ratio_x > 1.0 || ratio_y > 1.0
    }

    pub fn render(&mut self, window: &mut dyn EditorWindow, transform: &Matrix4<f32>) {
        let image_area_transform = self.image_area_transform_matrix4(true);
        let image_area_transform_full = self.image_area_transform_matrix4(false);

        self.render_image_area(
            transform,
            &image_area_transform,
            &image_area_transform_full
        );

        self.render_ui(
            window,
            transform,
            &image_area_transform_full
        );

        // let image_area_rectangle = self.image_area_rectangle();
        // self.renders.solid_rectangle_render.render(
        //     self.renders.solid_rectangle_render.shader(),
        //     &transform,
        //     image_area_rectangle.position,
        //     image_area_rectangle.size,
        //     Color4::new(255, 0, 0, 128)
        // );
    }

    fn render_image_area(&mut self,
                         transform: &Matrix4<f32>,
                         image_area_transform: &Matrix4<f32>,
                         image_area_transform_full: &Matrix4<f32>) {
        let (transparent_background_start, transparent_background_width, transparent_background_height) = self.calculate_transparent_background_rectangle();
        if transparent_background_width > 0.0 && transparent_background_height > 0.0 {
            self.renders.texture_render.render_sized(
                self.renders.texture_render.shader(),
                &(transform * image_area_transform),
                &self.transparent_background_texture,
                transparent_background_start,
                transparent_background_width,
                transparent_background_height,
                Some(Rectangle::new(0.0, 0.0, transparent_background_width, transparent_background_height))
            );
        }

        let image_crop_rectangle = Rectangle::new(
            self.view_x,
            self.view_y,
            self.view_width as f32 / self.zoom,
            self.view_height as f32 / self.zoom
        );

        let mut transparent_area = None;
        let changed = self.tools[self.active_tool.index()].preview(
            self.editor.active_layer(),
            &mut self.preview_image,
            &mut transparent_area
        );

        if changed {
            self.preview_image.clear_cpu();
        }

        for (index, (state, image)) in self.editor.image().layers().iter().enumerate() {
            if state == &LayerState::Visible {
                self.renders.texture_render.render_sub(
                    self.renders.texture_render.shader(),
                    &(transform * image_area_transform),
                    image.get_texture(),
                    Position::new(0.0, 0.0),
                    self.zoom,
                    Some(image_crop_rectangle.clone())
                );
            }

            if index == self.editor.active_layer_index() {
                if let Some(transparent_area) = transparent_area.as_ref() {
                    self.renders.texture_render.render_sized(
                        self.renders.texture_render.shader(),
                        &(transform * image_area_transform_full),
                        &self.transparent_background_texture,
                        transparent_area.position,
                        transparent_area.size.x,
                        transparent_area.size.y,
                        Some(
                            Rectangle::new(
                                0.0,
                                0.0,
                                transparent_area.size.x * self.zoom,
                                transparent_area.size.y * self.zoom
                            )
                        )
                    );
                }

                self.renders.texture_render.render_sub(
                    self.renders.texture_render.shader(),
                    &(transform * image_area_transform),
                    self.preview_image.get_texture(),
                    Position::new(0.0, 0.0),
                    self.zoom,
                    Some(image_crop_rectangle.clone())
                );
            }
        }

        self.tools[self.active_tool.index()].render_image_area(
            &self.renders,
            &transform,
            &image_area_transform_full,
            self.editor.active_layer()
        );

        self.renders.rectangle_render.render(
            self.renders.rectangle_render.shader(),
            &(transform * image_area_transform_full),
            &Rectangle::new(
                0.0,
                0.0,
                self.editor.image().width() as f32,
                self.editor.image().height() as f32
            ),
            Color4::new(0, 0, 0, 255)
        )
    }

    fn render_ui(&mut self,
                 window: &mut dyn EditorWindow,
                 transform: &Matrix4<f32>,
                 image_area_transform_full: &Matrix4<f32>) {
        let menu_color = Color4::new(255, 255, 255, 255);
        self.renders.solid_rectangle_render.render(
            self.renders.solid_rectangle_render.shader(),
            transform,
            Position::new(0.0, 0.0),
            Size::new(LEFT_SIDE_PANEL_WIDTH as f32, self.window_height as f32),
            menu_color
        );

        self.renders.solid_rectangle_render.render(
            self.renders.solid_rectangle_render.shader(),
            transform,
            Position::new(0.0, 0.0),
            Size::new(self.window_width as f32, TOP_PANEL_HEIGHT as f32),
            menu_color
        );

        self.renders.solid_rectangle_render.render(
            self.renders.solid_rectangle_render.shader(),
            transform,
            Position::new(self.window_width as f32 - RIGHT_SIDE_PANEL_WIDTH as f32, 0.0),
            Size::new(RIGHT_SIDE_PANEL_WIDTH as f32, self.window_height as f32),
            menu_color
        );

        let mouse_position = get_transformed_mouse_position(window, &self.image_area_transform(false).invert().unwrap());
        self.renders.text_render.draw_line(
            self.renders.text_render.shader(),
            transform,
            self.renders.ui_font.borrow_mut().deref_mut(),
            format!("{:.0} %, {:.0}, {:.0}", self.zoom * 100.0, mouse_position.x.round(), mouse_position.y.round()).chars().map(|c| (c, Color::new(0, 0, 0))),
            Position::new(self.window_width as f32 - RIGHT_SIDE_PANEL_WIDTH as f32 - 160.0, 10.0),
            TextAlignment::Top
        );

        self.layers_manager.render(
            transform,
            &self.renders,
            &self.editor,
            self.window_width - SIDE_PANELS_WIDTH,
            &self.transparent_background_texture,
        );

        self.ui_manager.render(&self.renders, &transform);

        self.tools[self.active_tool.index()].render_ui(
            &self.renders,
            &transform,
            &image_area_transform_full,
            self.editor.active_layer()
        );
    }

    fn calculate_transparent_background_rectangle(&self) -> (Position, f32, f32) {
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

    fn image_size_changed(&mut self) {
        self.preview_image = self.editor.new_image_same();
        self.zoom = 1.0;
        self.view_x = 0.0;
        self.view_y = 0.0;
        self.update_view_size();
    }

    fn update_view_size(&mut self) {
        self.view_width = (self.window_width - SIDE_PANELS_WIDTH).min((self.editor.image().width() as f32 * self.zoom.max(1.0)) as u32);
        self.view_height = (self.window_height - TOP_PANEL_HEIGHT).min((self.editor.image().height() as f32 * self.zoom.max(1.0)) as u32);
    }

    fn image_area_transform(&self, only_origin: bool) -> Matrix3<f32> {
        let mut origin_x = LEFT_SIDE_PANEL_WIDTH as f32;
        let mut origin_y = TOP_PANEL_HEIGHT as f32;

        let center_origin_x = self.window_width as f32 / 2.0 - self.view_width as f32 / 2.0;
        let center_origin_y = self.window_height as f32 / 2.0 - self.view_height as f32 / 2.0;

        if (origin_x + self.view_width as f32) < self.window_width as f32 - SIDE_PANELS_WIDTH as f32 {
            origin_x = center_origin_x;
        }

        if center_origin_y > origin_y {
            origin_y = center_origin_y;
        }

        let origin_transform = cgmath::Matrix3::from_cols(
            cgmath::Vector3::new(1.0, 0.0, origin_x),
            cgmath::Vector3::new(0.0, 1.0, origin_y),
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
        Rectangle::new(x, y, self.view_width as f32, self.view_height as f32)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum ProgramActions {
    OpenImage,
    SaveImageAs,
    ResizeImage,
    ResizeCanvas
}

pub struct ProgramActionsManager {
    states: HashMap<ProgramActions, bool>
}

impl ProgramActionsManager {
    pub fn new() -> ProgramActionsManager {
        ProgramActionsManager {
            states: HashMap::new()
        }
    }

    pub fn trigger(&mut self, action: ProgramActions) {
        *self.states.entry(action).or_insert(true) = true;
    }

    pub fn is_triggered(&mut self, action: &ProgramActions) -> bool {
        if let Some(state) = self.states.get_mut(action) {
            let current_state = *state;
            *state = false;
            current_state
        } else {
            false
        }
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