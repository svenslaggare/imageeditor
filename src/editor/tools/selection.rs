use glfw::{WindowEvent, Action, Key, Modifiers};
use cgmath::{Matrix3, Transform, Matrix4, EuclideanSpace};

use crate::rendering::prelude::{Position, Rectangle, Size, Color4};
use crate::editor;
use crate::command_buffer::{Command, CommandBuffer};
use crate::editor::tools::{Tool, get_valid_rectangle, SelectionSubTool, Tools, get_transformed_mouse_position, EditorWindow, get_valid_rectangle_as_int};
use crate::editor::image_operation::{ImageOperation, ImageSource, add_op_sequential, select_latest};
use crate::editor::image_operation_helpers::sub_image;
use crate::program::Renders;
use crate::editor::Region;

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

    pub fn region(&self) -> Region {
        Region::new(
            self.start_x as i32,
            self.start_y as i32,
            (self.end_x - self.start_x) as i32,
            (self.end_y - self.start_y) as i32
        )
    }
}

struct SelectState {
    is_selecting: bool,
    copied_image: Option<image::RgbaImage>,
    triggered_resize: bool
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

struct RotatePixelsState {
    is_rotating: bool,
    original_selection: Option<Selection>,
    rotate_pixels_image: Option<image::RgbaImage>,
    rotation: f32
}

impl RotatePixelsState {
    pub fn clear(&mut self) {
        self.is_rotating = false;
        self.rotate_pixels_image = None;
        self.rotation = 0.0;
    }
}

pub struct SelectionTool {
    tool: SelectionSubTool,
    changed_selection: bool,
    start_position: Option<Position>,
    end_position: Option<Position>,
    skip_erase_original_selection: bool,
    select_state: SelectState,
    move_pixels_state: MovePixelsState,
    resize_pixels_state: ResizePixelsState,
    rotate_pixels_state: RotatePixelsState
}

impl SelectionTool {
    pub fn new() -> SelectionTool {
        SelectionTool {
            tool: SelectionSubTool::Select,
            changed_selection: false,
            start_position: None,
            end_position: None,
            // start_position: Some(Position::new(243.0, 325.0)),
            // end_position: Some(Position::new(739.0, 545.0)),
            skip_erase_original_selection: false,
            select_state: SelectState {
                is_selecting: false,
                copied_image: None,
                triggered_resize: false
            },
            move_pixels_state: MovePixelsState {
                original_selection: None,
                is_moving: false,
                move_offset: cgmath::Vector2::new(0.0, 0.0),
                moved_pixels_image: None,
            },
            resize_pixels_state: ResizePixelsState {
                is_resizing: false,
                original_selection: None,
                resize_pixels_image: None
            },
            rotate_pixels_state: RotatePixelsState {
                is_rotating: false,
                original_selection: None,
                rotate_pixels_image: None,
                rotation: 0.0
            }
        }
    }

    fn selection(&self) -> Option<Selection> {
        match (self.start_position, self.end_position) {
            (Some(start_position), Some(end_position)) => {
                let (start_x, start_y, end_x, end_y) = get_valid_rectangle_as_int(&start_position, &end_position);
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
                            command_buffer: &mut CommandBuffer,
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
                        add_op_sequential(&mut op, self.create_resize(false));
                        self.resize_pixels_state.clear();
                    }

                    if self.rotate_pixels_state.rotate_pixels_image.is_some() {
                        add_op_sequential(&mut op, self.create_rotation(false));
                        self.rotate_pixels_state.clear();
                    }

                    self.set_start_position(Some(get_transformed_mouse_position(window, image_area_transform)));
                    self.set_end_position(None);
                    self.select_state.is_selecting = true;
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                self.select_state.is_selecting = false;
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                let mouse_position = image_area_transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));
                if self.select_state.is_selecting {
                    if window.is_shift_down() {
                        let start_position = self.start_position.unwrap();
                        let distance = (mouse_position.x - start_position.x).max(mouse_position.y - start_position.y);
                        self.set_end_position(Some(Position::new(start_position.x + distance, start_position.y + distance)));
                    } else {
                        self.set_end_position(Some(mouse_position));
                    }
                }
            }
            glfw::WindowEvent::Key(Key::A, _, Action::Press, Modifiers::Control) => {
                self.select_all(image);
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

                    self.set_start_position(None);
                    self.set_end_position(None);
                }
            }
            glfw::WindowEvent::Key(Key::C, _, Action::Press, Modifiers::Control) => {
                if let Some(selection) = self.selection() {
                    let copied_image = sub_image(image, selection.start_x, selection.start_y, selection.end_x, selection.end_y);
                    command_buffer.push(Command::SetCopiedImage(copied_image.clone()));
                    self.select_state.copied_image = Some(copied_image);

                    self.set_start_position(None);
                    self.set_end_position(None);
                }
            }
            glfw::WindowEvent::Key(Key::V, _, Action::Press, Modifiers::Control) => {
                if let Some(copied_image) = self.select_state.copied_image.as_ref() {
                    if (copied_image.width() <= image.width() && copied_image.height() <= image.height()) || self.select_state.triggered_resize {
                        let copied_image = copied_image.clone();
                        self.handle_paste(copied_image);
                    } else {
                        self.select_state.triggered_resize = true;
                        command_buffer.push(Command::RequestResizeCanvas(copied_image.width(), copied_image.height()));
                    }
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

                    self.set_start_position(None);
                    self.set_end_position(None);
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
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                self.move_pixels_state.is_moving = false;

                let current_mouse_position = get_transformed_mouse_position(window, image_area_transform);
                if let Some(selection) = self.selection() {
                    let selection_rectangle = Rectangle::from_min_and_max(&selection.start_position(), &selection.end_position());
                    if selection_rectangle.contains(&current_mouse_position) {
                        if self.move_pixels_state.moved_pixels_image.is_none() {
                            self.move_pixels_state.original_selection = Some(selection.clone());
                            self.move_pixels_state.moved_pixels_image = Some(
                                sub_image(
                                    image,
                                    selection.start_x,
                                    selection.start_y,
                                    selection.end_x,
                                    selection.end_y
                                )
                            );
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
                    let (start_x, start_y, end_x, end_y) = get_valid_rectangle(&self.start_position.unwrap(), &self.end_position.unwrap());
                    let offset = Position::new(end_x, end_y) - Position::new(start_x, start_y);
                    let new_start_position = mouse_position + self.move_pixels_state.move_offset;

                    self.set_start_position(Some(new_start_position));
                    self.set_end_position(Some(new_start_position + offset));
                }
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Release, _ ) => {
                if let Some(original_selection) = self.original_selection() {
                    self.set_start_position(Some(original_selection.start_position()));
                    self.set_end_position(Some(original_selection.end_position()));
                }

                self.clear_states();
            }
            _ => {}
        }

        None
    }

    fn process_event_resize_pixels(&mut self,
                                   window: &mut dyn EditorWindow,
                                   event: &glfw::WindowEvent,
                                   image_area_transform: &Matrix3<f32>,
                                   _image_area_rectangle: &Rectangle,
                                   _command_buffer: &mut CommandBuffer,
                                   image: &editor::Image) -> Option<ImageOperation> {
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                self.resize_pixels_state.is_resizing = false;

                let current_mouse_position = get_transformed_mouse_position(window, image_area_transform);
                if let Some(mut selection) = self.selection() {
                    let selection_rectangle = Rectangle::from_min_and_max(&selection.start_position(), &selection.end_position());
                    if selection_rectangle.contains(&current_mouse_position) {
                        if let Some(original_selection) = self.move_pixels_state.original_selection.as_ref() {
                            selection = original_selection.clone();
                        }

                        if self.resize_pixels_state.resize_pixels_image.is_none() {
                            self.resize_pixels_state.original_selection = Some(selection.clone());
                            self.resize_pixels_state.resize_pixels_image = Some(
                                sub_image(
                                    image,
                                    selection.start_x,
                                    selection.start_y,
                                    selection.end_x,
                                    selection.end_y
                                )
                            );
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
                    let mouse_position = image_area_transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));

                    if window.is_shift_down() {
                        let start_position = self.start_position.unwrap();
                        let distance = (mouse_position.x - start_position.x).max(mouse_position.y - start_position.y);
                        self.set_end_position(Some(Position::new(start_position.x + distance, start_position.y + distance)));
                    } else {
                        self.set_end_position(Some(mouse_position));
                    }
                }
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Release, _ ) => {
                if let Some(original_selection) = self.original_selection() {
                    self.set_start_position(Some(original_selection.start_position()));
                    self.set_end_position(Some(original_selection.end_position()));
                }

                self.clear_states();
            }
            _ => {}
        }

        None
    }

    fn process_event_rotate_pixels(&mut self,
                                   window: &mut dyn EditorWindow,
                                   event: &glfw::WindowEvent,
                                   image_area_transform: &Matrix3<f32>,
                                   _image_area_rectangle: &Rectangle,
                                   _command_buffer: &mut CommandBuffer,
                                   image: &editor::Image) -> Option<ImageOperation> {
        match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                self.rotate_pixels_state.is_rotating = false;

                let current_mouse_position = get_transformed_mouse_position(window, image_area_transform);
                if let Some(mut selection) = self.selection() {
                    let selection_rectangle = Rectangle::from_min_and_max(&selection.start_position(), &selection.end_position());
                    if selection_rectangle.contains(&current_mouse_position) {
                        if let Some(original_selection) = self.move_pixels_state.original_selection.as_ref() {
                            selection = original_selection.clone();
                        }

                        if self.rotate_pixels_state.rotate_pixels_image.is_none() {
                            self.resize_pixels_state.original_selection = Some(selection.clone());
                            self.rotate_pixels_state.rotate_pixels_image = Some(
                                sub_image(
                                    image,
                                    selection.start_x,
                                    selection.start_y,
                                    selection.end_x,
                                    selection.end_y
                                )
                            );
                        }

                        self.rotate_pixels_state.is_rotating = true;
                    }
                }
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                self.rotate_pixels_state.is_rotating = false;
            }
            glfw::WindowEvent::CursorPos(raw_mouse_x, raw_mouse_y) => {
                if self.rotate_pixels_state.is_rotating {
                    if let (Some(start_position), Some(end_position)) = (self.start_position, self.end_position) {
                        let mouse_position = image_area_transform.transform_point(cgmath::Point2::new(*raw_mouse_x as f32, *raw_mouse_y as f32));
                        let diff = mouse_position - (start_position.to_vec() + end_position.to_vec()) * 0.5;
                        let mut angle = diff.y.atan2(diff.x);

                        if window.is_shift_down() {
                            angle = (angle / 45.0_f32.to_radians()).round() * 45.0_f32.to_radians();
                        }

                        self.rotate_pixels_state.rotation = angle;
                    }
                }
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Release, _ ) => {
                if let Some(original_selection) = self.original_selection() {
                    self.set_start_position(Some(original_selection.start_position()));
                    self.set_end_position(Some(original_selection.end_position()));
                }

                self.clear_states();
            }
            _ => {}
        }

        None
    }

    fn handle_paste(&mut self, copied_image: image::RgbaImage) {
        self.set_start_position(Some(Position::new(0.0, 0.0)));
        self.set_end_position(Some(Position::new(copied_image.width() as f32, copied_image.height() as f32)));

        self.move_pixels_state.original_selection = self.selection();
        self.move_pixels_state.moved_pixels_image = Some(copied_image);
        self.skip_erase_original_selection = true;
        self.select_state.triggered_resize = false;
    }

    fn create_move(&self, preview: bool) -> Option<ImageOperation> {
        match (self.selection(), self.original_selection(), self.move_pixels_state.moved_pixels_image.as_ref()) {
            (Some(selection), Some(original_selection), Some(moved_pixels_image)) => {
                let set_image = ImageOperation::SetImage {
                    start_x: selection.start_x as i32,
                    start_y: selection.start_y as i32,
                    image: moved_pixels_image.clone(),
                    blend: true
                };

                return if !self.skip_erase_original_selection {
                    Some(
                        ImageOperation::Sequential(
                            Some("Move pixels".to_owned()),
                            vec![
                                self.create_erased_area(&original_selection, preview),
                                set_image
                            ]
                        )
                    )
                } else {
                    Some(set_image)
                }
            }
            _ => {}
        }

        None
    }

    fn create_resize(&self, preview: bool) -> Option<ImageOperation> {
        match (self.selection(), self.original_selection(), self.resize_pixels_state.resize_pixels_image.as_ref()) {
            (Some(selection), Some(original_selection), Some(resize_pixels_image)) => {
                let set_image = ImageOperation::SetScaledImage {
                    image: resize_pixels_image.clone(),
                    start_x: selection.start_x,
                    start_y: selection.start_y,
                    scale_x: (selection.end_x - selection.start_x) as f32 / resize_pixels_image.width() as f32,
                    scale_y: (selection.end_y - selection.start_y) as f32 / resize_pixels_image.height() as f32
                };

                return if !self.skip_erase_original_selection {
                    Some(
                        ImageOperation::Sequential(
                            Some("Scale pixels".to_owned()),
                            vec![
                                self.create_erased_area(&original_selection, preview),
                                set_image
                            ]
                        )
                    )
                } else {
                    Some(set_image)
                }
            }
            _ => {}
        }

        None
    }

    fn create_rotation(&self, preview: bool) -> Option<ImageOperation> {
        match (self.selection(), self.original_selection(), self.rotate_pixels_state.rotate_pixels_image.as_ref()) {
            (Some(selection), Some(original_selection), Some(resize_pixels_image)) => {
                let set_image = ImageOperation::SetRotatedImage {
                    image: resize_pixels_image.clone(),
                    start_x: selection.start_x,
                    start_y: selection.start_y,
                    end_x: selection.end_x,
                    end_y: selection.end_y,
                    rotation: self.rotate_pixels_state.rotation
                };

                return if !self.skip_erase_original_selection {
                    Some(
                        ImageOperation::Sequential(
                            Some("Rotate pixels".to_owned()),
                            vec![
                                self.create_erased_area(&original_selection, preview),
                                set_image
                            ]
                        )
                    )
                } else {
                    Some(set_image)
                }
            }
            _ => {}
        }

        None
    }

    fn create_erased_area(&self, selection: &Selection, preview: bool) -> ImageOperation{
        if !preview {
            ImageOperation::FillRectangle {
                start_x: selection.start_x,
                start_y: selection.start_y,
                end_x: selection.end_x,
                end_y: selection.end_y,
                color: image::Rgba([0, 0, 0, 0]),
                blend: false
            }
        } else {
            ImageOperation::Empty
        }
    }

    fn select_all(&mut self, image: &editor::Image) {
        self.set_start_position(Some(Position::new(0.0, 0.0)));
        self.set_end_position(Some(Position::new(image.width() as f32, image.height() as f32)));
    }

    fn set_start_position(&mut self, position: Option<Position>) {
        self.start_position = position;
        self.changed_selection = true;
    }

    fn set_end_position(&mut self, position: Option<Position>) {
        self.end_position = position;
        self.changed_selection = true;
    }

    fn before_change_selection(&mut self) {
        self.changed_selection = false;
    }

    fn after_change_selection(&mut self, command_buffer: &mut CommandBuffer) {
        if self.changed_selection {
            command_buffer.push(Command::SetSelection(self.selection()));
            self.changed_selection = false;
        }
    }

    fn clear_states(&mut self) {
        self.move_pixels_state.clear();
        self.resize_pixels_state.clear();
        self.rotate_pixels_state.clear();
        self.skip_erase_original_selection = false;
    }
}

impl Tool for SelectionTool {
    fn on_active(&mut self, _window: &mut dyn EditorWindow, tool: Tools) -> Option<ImageOperation> {
        if let Tools::Selection(sub_tool) = tool {
            self.tool = sub_tool;
        }

        None
    }

    fn on_deactivate(&mut self, command_buffer: &mut CommandBuffer) -> Option<ImageOperation> {
        self.before_change_selection();

        let op = select_latest([
            self.create_move(false),
            self.create_resize(false),
            self.create_rotation(false)
        ]);

        if self.move_pixels_state.moved_pixels_image.is_some() {
            self.move_pixels_state.clear();
        }

        if self.resize_pixels_state.resize_pixels_image.is_some() {
            self.resize_pixels_state.clear();
        }

        if self.rotate_pixels_state.rotate_pixels_image.is_some() {
            self.rotate_pixels_state.clear();
        }

        if op.is_some() {
            self.set_start_position(None);
            self.set_end_position(None);
        }

        self.after_change_selection(command_buffer);

        op
    }

    fn process_gui_event(&mut self,
                         window: &mut dyn EditorWindow,
                         event: &WindowEvent,
                         image_area_transform: &Matrix3<f32>,
                         image_area_rectangle: &Rectangle,
                         command_buffer: &mut CommandBuffer,
                         image: &editor::Image) -> Option<ImageOperation> {
        self.before_change_selection();

        let mut op = match self.tool {
            SelectionSubTool::Select => self.process_event_select(window, event, image_area_transform, image_area_rectangle, command_buffer, image),
            SelectionSubTool::MovePixels => self.process_event_move_pixels(window, event, image_area_transform, image_area_rectangle, command_buffer, image),
            SelectionSubTool::ResizePixels => self.process_event_resize_pixels(window, event, image_area_transform, image_area_rectangle, command_buffer, image),
            SelectionSubTool::RotatePixels => self.process_event_rotate_pixels(window, event, image_area_transform, image_area_rectangle, command_buffer, image),
        };

        match event {
            glfw::WindowEvent::Key(Key::Enter, _, Action::Release, _) => {
                add_op_sequential(
                    &mut op,
                    select_latest([
                        self.create_move(false),
                        self.create_resize(false),
                        self.create_rotation(false)
                    ])
                );

                self.set_start_position(None);
                self.set_end_position(None);

                self.clear_states();
                command_buffer.push(Command::SetTool(Tools::Selection(SelectionSubTool::Select)));
            }
            _ => {}
        }

        self.after_change_selection(command_buffer);

        op
    }

    fn handle_command(&mut self, command_buffer: &mut CommandBuffer, image: &editor::Image, command: &Command) {
        self.before_change_selection();

        match command {
            Command::SelectAll => {
                self.select_all(image);
            }
            Command::ResizeCanvas(_, _) => {
                if self.select_state.triggered_resize {
                    if let Some(copied_image) = self.select_state.copied_image.as_ref() {
                        let copied_image = copied_image.clone();
                        self.handle_paste(copied_image);
                    }
                }
            }
            Command::AbortedResizeCanvas => {
                if self.select_state.triggered_resize {
                    if let Some(copied_image) = self.select_state.copied_image.as_ref() {
                        let copied_image = copied_image.clone();
                        self.handle_paste(copied_image);
                    }
                }
            }
            Command::SetClipboard(image) => {
                self.select_state.copied_image = Some(image.clone());
            }
            _ => {}
        }

        self.after_change_selection(command_buffer);
    }

    fn preview(&mut self,
               _image: &editor::Image,
               preview_image: &mut editor::Image,
               transparent_area: &mut Option<Rectangle>) -> bool {
        let mut update_op = preview_image.update_operation();

        let mut erased_area = false;
        if let Some(preview_op) = select_latest([self.create_move(true),
                                                 self.create_resize(true),
                                                 self.create_rotation(true)]) {
            erased_area = !self.skip_erase_original_selection;
            preview_op.apply(&mut update_op, false);
        }

        if erased_area {
            if let Some(selection) = self.original_selection() {
                *transparent_area = Some(Rectangle::from_min_and_max(&selection.start_position(), &selection.end_position()));
            }
        }

        return true;
    }

    fn render_image_area(&mut self, renders: &Renders, transform: &Matrix4<f32>, image_area_transform: &Matrix4<f32>, image: &editor::Image) {
        if let Some(mut selection) = self.selection() {
            let clamp_x = |x: i32| x.clamp(0, image.width() as i32 - 1);
            let clamp_y = |y: i32| y.clamp(0, image.height() as i32 - 1);

            selection.start_x = clamp_x(selection.start_x);
            selection.start_y = clamp_y(selection.start_y);
            selection.end_x = clamp_x(selection.end_x);
            selection.end_y = clamp_y(selection.end_y);

            renders.solid_rectangle_render.render(
                renders.solid_rectangle_render.shader(),
                &(transform * image_area_transform),
                &Rectangle::from_position_and_size(
                    selection.start_position(),
                    selection.size(),
                ),
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

    fn render_image_area_inactive(&mut self, renders: &Renders, transform: &Matrix4<f32>, image_area_transform: &Matrix4<f32>, image: &editor::Image) {
        self.render_image_area(
            renders,
            transform,
            image_area_transform,
            image,
        );
    }
}