use glfw::{WindowEvent, Action, Key, Modifiers, Window};
use cgmath::{Matrix3, Transform, Matrix, Matrix4};

use crate::rendering::prelude::{Position, Rectangle, Size, Color, Color4};
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_valid_rectangle, SelectionSubTool, Tools, get_transformed_mouse_position, EditorWindow};
use crate::editor::image_operation::{ImageOperation, ImageSource, add_op_sequential};
use crate::editor::image_operation_helpers::sub_image;
use crate::editor::Image;
use crate::program::Renders;

#[derive(Debug, Clone)]
pub struct Selection {
    pub start_x: i32,
    pub start_y: i32,
    pub end_x: i32,
    pub end_y: i32
}

impl Selection {
    pub fn start_position(&self) -> Position {
        Position::new(self.start_x as f32, self.start_y as f32)
    }

    pub fn end_position(&self) -> Position {
        Position::new(self.end_x as f32, self.end_y as f32)
    }

    pub fn size(&self) -> Size {
        Size::new((self.end_x - self.start_x) as f32, (self.end_y - self.start_y) as f32)
    }

    pub fn rectangle(&self) -> Rectangle {
        Rectangle::new(
            self.start_x as f32,
            self.start_y as f32,
            (self.end_x - self.start_x) as f32,
            (self.end_y - self.start_y) as f32
        )
    }
}

struct SelectState {
    is_selecting: bool,
    copied_image: Option<image::RgbaImage>
}

struct MovePixelsState {
    is_moving: bool,
    original_selection: Option<Selection>,
    move_offset: cgmath::Vector2<f32>,
    moved_pixels_image: Option<image::RgbaImage>
}

impl MovePixelsState {
    pub fn clear(&mut self) {
        self.is_moving = false;
        self.moved_pixels_image = None;
        self.original_selection = None;
    }
}

struct ResizePixelsState {
    is_resizing: bool,
    original_selection: Option<Selection>,
    resize_pixels_image: Option<image::RgbaImage>
}

impl ResizePixelsState {
    pub fn clear(&mut self) {
        self.is_resizing = false;
        self.resize_pixels_image = None;
        self.original_selection = None;
    }
}

pub struct SelectionTool {
    tool: SelectionSubTool,
    start_position: Option<Position>,
    end_position: Option<Position>,
    select_state: SelectState,
    move_pixels_state: MovePixelsState,
    resize_pixels_state: ResizePixelsState,
    transparent_image: image::RgbaImage
}

impl SelectionTool {
    pub fn new() -> SelectionTool {
        SelectionTool {
            tool: SelectionSubTool::Select,
            start_position: None,
            end_position: None,
            select_state: SelectState {
                is_selecting: false,
                copied_image: None
            },
            move_pixels_state: MovePixelsState {
                original_selection: None,
                is_moving: false,
                move_offset: cgmath::Vector2::new(0.0, 0.0),
                moved_pixels_image: None
            },
            resize_pixels_state: ResizePixelsState {
                is_resizing: false,
                original_selection: None,
                resize_pixels_image: None
            },
            transparent_image: image::open("content/ui/checkerboard.png").unwrap().into_rgba()
        }
    }

    fn selection(&self) -> Option<Selection> {
        match (self.start_position, self.end_position) {
            (Some(start_position), Some(end_position)) => {
                let (start_x, start_y, end_x, end_y) = get_valid_rectangle(&start_position, &end_position);
                Some(
                    Selection {
                        start_x,
                        start_y,
                        end_x,
                        end_y
                    }
                )
            }
            _ => None
        }
    }

    fn original_selection(&self) -> Option<Selection> {
        if let Some(selection) = self.resize_pixels_state.original_selection.as_ref() {
            return Some(selection.clone());
        }

        if let Some(selection) = self.move_pixels_state.original_selection.as_ref() {
            return Some(selection.clone());
        }

        None
    }

    fn process_event_select(&mut self,
                            window: &mut dyn EditorWindow,
                            event: &glfw::WindowEvent,
                            image_area_transform: &Matrix3<f32>,
                            image_area_rectangle: &Rectangle,
                            _command_buffer: &mut CommandBuffer,
                            image: &editor::Image) -> Option<ImageOperation> {
        let mut op = None;
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                let (mouse_x, mouse_y) = window.get_cursor_pos();
                if image_area_rectangle.contains(&Position::new(mouse_x as f32, mouse_y as f32)) {
                    if self.move_pixels_state.moved_pixels_image.is_some() {
                        add_op_sequential(&mut op, self.create_move(false));
                        self.move_pixels_state.clear();
                    }

                    if self.resize_pixels_state.resize_pixels_image.is_some() {
                        add_op_sequential(&mut op, self.create_move(false));
                        self.resize_pixels_state.clear();
                    }

                    self.start_position = Some(get_transformed_mouse_position(window, image_area_transform));
                    self.end_position = None;
                    self.select_state.is_selecting = true;
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                self.select_state.is_selecting = false;
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                let mouse_position = image_area_transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));
                if self.select_state.is_selecting {
                    self.end_position = Some(mouse_position);
                }
            }
            glfw::WindowEvent::Key(Key::A, _, Action::Press, Modifiers::Control) => {
                self.start_position = Some(Position::new(0.0, 0.0));
                self.end_position = Some(Position::new(image.width() as f32, image.height() as f32));
            }
            glfw::WindowEvent::Key(Key::Delete, _, Action::Press, _) => {
                if let Some(selection) = self.selection() {
                    op = Some(
                        ImageOperation::FillRectangle {
                            start_x: selection.start_x,
                            start_y: selection.start_y,
                            end_x: selection.end_x,
                            end_y: selection.end_y,
                            color: image::Rgba([0, 0, 0, 0]),
                            blend: false
                        }
                    );

                    self.start_position = None;
                    self.end_position = None;
                }
            }
            glfw::WindowEvent::Key(Key::C, _, Action::Press, Modifiers::Control) => {
                if let Some(selection) = self.selection() {
                    self.select_state.copied_image = Some(
                        sub_image(image, selection.start_x, selection.start_y, selection.end_x, selection.end_y)
                    );

                    self.start_position = None;
                    self.end_position = None;
                }
            }
            glfw::WindowEvent::Key(Key::V, _, Action::Press, Modifiers::Control) => {
                let mouse_position = get_transformed_mouse_position(window, image_area_transform);
                let start_x = mouse_position.x as i32;
                let start_y = mouse_position.y as i32;

                if let Some(copied_image) = self.select_state.copied_image.as_ref() {
                    op = Some(ImageOperation::SetImage { start_x, start_y, image: copied_image.clone(), blend: false });
                }
            }
            glfw::WindowEvent::Key(Key::X, _, Action::Press, Modifiers::Control) => {
                if let Some(selection) = self.selection() {
                    op = Some(
                        ImageOperation::FillRectangle {
                            start_x: selection.start_x,
                            start_y: selection.start_y,
                            end_x: selection.end_x,
                            end_y: selection.end_y,
                            color: image::Rgba([0, 0, 0, 0]),
                            blend: false
                        }
                    );

                    self.select_state.copied_image = Some(
                        sub_image(image, selection.start_x, selection.start_y, selection.end_x, selection.end_y)
                    );

                    self.start_position = None;
                    self.end_position = None;
                }
            }
            _ => {}
        }

        return op;
    }

    fn process_event_move_pixels(&mut self,
                                 window: &mut dyn EditorWindow,
                                 event: &glfw::WindowEvent,
                                 image_area_transform: &Matrix3<f32>,
                                 _image_area_rectangle: &Rectangle,
                                 _command_buffer: &mut CommandBuffer,
                                 image: &editor::Image) -> Option<ImageOperation> {
        let mut op = None;
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                self.move_pixels_state.is_moving = false;

                let current_mouse_position = get_transformed_mouse_position(window, image_area_transform);
                if let Some(selection) = self.selection() {
                    let selection_rectangle = Rectangle::from_min_and_max(&selection.start_position(), &selection.end_position());
                    if selection_rectangle.contains(&current_mouse_position) {
                        if self.move_pixels_state.moved_pixels_image.is_none() {
                            self.move_pixels_state.original_selection = Some(selection.clone());
                            self.move_pixels_state.moved_pixels_image = Some(sub_image(image, selection.start_x, selection.start_y, selection.end_x, selection.end_y));
                        }

                        self.move_pixels_state.is_moving = true;
                        self.move_pixels_state.move_offset = selection.start_position() - current_mouse_position;
                    }
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                self.move_pixels_state.is_moving = false;
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                let mouse_position = image_area_transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));

                if self.move_pixels_state.is_moving {
                    let selection = self.selection().unwrap();
                    let offset = selection.end_position() - selection.start_position();
                    let new_start_position = mouse_position + self.move_pixels_state.move_offset;

                    self.start_position = Some(new_start_position);
                    self.end_position = Some(new_start_position + offset);
                }
            }
            _ => {}
        }

        return op;
    }

    fn process_event_resize_pixels(&mut self,
                                   window: &mut dyn EditorWindow,
                                   event: &glfw::WindowEvent,
                                   image_area_transform: &Matrix3<f32>,
                                   _image_area_rectangle: &Rectangle,
                                   _command_buffer: &mut CommandBuffer,
                                   image: &editor::Image) -> Option<ImageOperation> {
        let mut op = None;
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                self.resize_pixels_state.is_resizing = false;

                let current_mouse_position = get_transformed_mouse_position(window, image_area_transform);
                if let Some(selection) = self.selection() {
                    let selection_rectangle = Rectangle::from_min_and_max(&selection.start_position(), &selection.end_position());
                    if selection_rectangle.contains(&current_mouse_position) {
                        if self.resize_pixels_state.resize_pixels_image.is_none() {
                            self.resize_pixels_state.original_selection = Some(selection.clone());
                            self.resize_pixels_state.resize_pixels_image = Some(sub_image(image, selection.start_x, selection.start_y, selection.end_x, selection.end_y));
                        }

                        self.resize_pixels_state.is_resizing = true;
                    }
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                self.resize_pixels_state.is_resizing = false;
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                if self.resize_pixels_state.is_resizing {
                    self.end_position = Some(
                        image_area_transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32))
                    );
                }
            }
            _ => {}
        }

        return op;
    }

    fn create_move(&self, preview: bool) -> Option<ImageOperation> {
        match (self.selection(), self.original_selection(), self.move_pixels_state.moved_pixels_image.as_ref()) {
            (Some(selection), Some(original_selection), Some(moved_pixels_image)) => {
                return Some(
                    ImageOperation::Sequential(
                        vec![
                            self.create_erased_area(&original_selection, preview),
                            ImageOperation::SetImage {
                                start_x: selection.start_x as i32,
                                start_y: selection.start_y as i32,
                                image: moved_pixels_image.clone(),
                                blend: true
                            }
                        ]
                    )
                );
            }
            _ => {}
        }

        None
    }

    fn create_resize(&self, preview: bool) -> Option<ImageOperation> {
        match (self.selection(), self.original_selection(), self.resize_pixels_state.resize_pixels_image.as_ref()) {
            (Some(selection), Some(original_selection), Some(resize_pixels_image)) => {
                return Some(
                    ImageOperation::Sequential(vec![
                        self.create_erased_area(&original_selection, preview),
                        ImageOperation::SetScaledImage {
                            image: resize_pixels_image.clone(),
                            start_x: selection.start_x,
                            start_y: selection.start_y,
                            scale_x: (selection.end_x - selection.start_x) as f32 / resize_pixels_image.width() as f32,
                            scale_y: (selection.end_y - selection.start_y) as f32 / resize_pixels_image.height() as f32
                        }
                    ])
                );
            }
            _ => {}
        }

        None
    }

    fn create_erased_area(&self, selection: &Selection, preview: bool) -> ImageOperation {
        if preview {
            ImageOperation::SetPseudoTransparent {
                start_x: selection.start_x,
                start_y: selection.start_y,
                end_x: selection.end_x,
                end_y: selection.end_y,
                pattern: self.transparent_image.clone()
            }
        } else {
            ImageOperation::FillRectangle {
                start_x: selection.start_x,
                start_y: selection.start_y,
                end_x: selection.end_x,
                end_y: selection.end_y,
                color: image::Rgba([0, 0, 0, 0]),
                blend: false
            }
        }
    }

    fn create_selection_gui(&self, selection: &Selection) -> ImageOperation {
        ImageOperation::Sequential(vec![
            ImageOperation::FillRectangle {
                start_x: selection.start_x,
                start_y: selection.start_y,
                end_x: selection.end_x,
                end_y: selection.end_y,
                color: image::Rgba([0, 148, 255, 64]),
                blend: true
            },
            ImageOperation::Rectangle {
                start_x: selection.start_x,
                start_y: selection.start_y,
                end_x: selection.end_x,
                end_y: selection.end_y,
                border_half_width: 0,
                color: image::Rgba([0, 0, 0, 255])
            }
        ])
    }
}

impl Tool for SelectionTool {
    fn on_active(&mut self, _window: &mut dyn EditorWindow, tool: Tools) -> Option<ImageOperation> {
        if let Tools::Selection(sub_tool) = tool {
            self.tool = sub_tool;
        }

        None
    }

    fn on_deactivate(&mut self, _command_buffer: &mut CommandBuffer) -> Option<ImageOperation> {
        let mut op = None;

        if self.move_pixels_state.moved_pixels_image.is_some() {
            add_op_sequential(&mut op, self.create_move(false));
            self.move_pixels_state.clear();
        }

        if self.resize_pixels_state.resize_pixels_image.is_some() {
            add_op_sequential(&mut op, self.create_resize(false));
            self.resize_pixels_state.clear();
        }

        if op.is_some() {
            self.start_position = None;
            self.end_position = None;
        }

        op
    }

    fn process_gui_event(&mut self,
                         window: &mut dyn EditorWindow,
                         event: &WindowEvent,
                         image_area_transform: &Matrix3<f32>,
                         image_area_rectangle: &Rectangle,
                         command_buffer: &mut CommandBuffer,
                         image: &editor::Image) -> Option<ImageOperation> {
        let mut op = match self.tool {
            SelectionSubTool::Select => self.process_event_select(window, event, image_area_transform, image_area_rectangle, command_buffer, image),
            SelectionSubTool::MovePixels => self.process_event_move_pixels(window, event, image_area_transform, image_area_rectangle, command_buffer, image),
            SelectionSubTool::ResizePixels => self.process_event_resize_pixels(window, event, image_area_transform, image_area_rectangle, command_buffer, image),
        };

        match event {
            glfw::WindowEvent::Key(Key::Enter, _, Action::Release, _) => {
                add_op_sequential(&mut op, self.create_move(false));
                add_op_sequential(&mut op, self.create_resize(false));

                self.start_position = None;
                self.end_position = None;

                self.move_pixels_state.clear();
                self.resize_pixels_state.clear();
            }
            _ => {}
        }

        op
    }

    fn preview(&mut self, _image: &editor::Image, preview_image: &mut editor::Image) -> bool {
        let mut update_op = preview_image.update_operation();
        // if let Some(selection) = self.selection() {
        //     if let Some(move_op) = self.create_move(true) {
        //         move_op.apply(&mut update_op, false);
        //     }
        //
        //     if let Some(resize_op) = self.create_resize(true) {
        //         resize_op.apply(&mut update_op, false);
        //     }
        //
        //     self.create_selection_gui(&selection).apply(&mut update_op, false);
        // }
        if let Some(move_op) = self.create_move(true) {
            move_op.apply(&mut update_op, false);
        }

        if let Some(resize_op) = self.create_resize(true) {
            resize_op.apply(&mut update_op, false);
        }

        return true;
    }

    fn render(&mut self, renders: &Renders, transform: &Matrix4<f32>, image_area_transform: &Matrix4<f32>, image: &editor::Image) {
        if let Some(mut selection) = self.selection() {
            selection.start_x = selection.start_x.max(0);
            selection.start_y = selection.start_y.max(0);
            selection.end_x = selection.end_x.min(image.width() as i32);
            selection.end_y = selection.end_y.min(image.height() as i32);

            renders.solid_rectangle_render.render(
                renders.solid_rectangle_render.shader(),
                &(transform * image_area_transform),
                selection.start_position(),
                selection.size(),
                Color4::new(0, 148, 255, 64)
            );

            renders.rectangle_render.render(
                renders.rectangle_render.shader(),
                &(transform * image_area_transform),
                &selection.rectangle(),
                Color4::new(0, 0, 0, 255)
            )
        }
    }
}