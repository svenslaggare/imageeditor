use glfw::{Action, Key, Modifiers, MouseButton};

use cgmath::{Matrix3, Transform, Matrix4};

use crate::editor::editor::{LayeredImageOperation, LayerState};
use crate::program::{RIGHT_SIDE_PANEL_WIDTH, LAYER_BUFFER, LAYER_SPACING, Renders};
use crate::rendering::prelude::{Position, Rectangle, Color4, blend, Size};
use crate::editor::Editor;
use crate::editor::tools::EditorWindow;
use crate::editor::image_operation::ImageSource;
use crate::rendering::texture::Texture;

pub struct LayersManager {

}

impl LayersManager {
    pub fn new() -> LayersManager {
        LayersManager {

        }
    }

    pub fn process_gui_event(&mut self,
                             window: &mut dyn EditorWindow,
                             image_area_transform: &Matrix3<f32>,
                             view_width: u32,
                             event: &glfw::WindowEvent,
                             editor: &mut Editor) {
        match event {
            glfw::WindowEvent::Key(Key::N, _, Action::Press, modifier) => {
                if modifier == &(Modifiers::Shift | Modifiers::Control) {
                    editor.image_mut().add_layer();
                }
            }
            glfw::WindowEvent::Key(Key::Delete, _, Action::Press, modifier) => {
                if modifier == &(Modifiers::Shift | Modifiers::Control) {
                    if editor.num_alive_layers() > 1 {
                        editor.apply_layer_op(
                            LayeredImageOperation::SetLayerState(
                                editor.active_layer_index(),
                                LayerState::Deleted
                            )
                        );
                    }
                }
            }
            glfw::WindowEvent::MouseButton(button, Action::Release, _) => {
                let mouse_position = window.get_cursor_pos();
                let mouse_position = Position::new(mouse_position.0 as f32, mouse_position.1 as f32);

                let mut layer_offset = LAYER_BUFFER;
                let layer_width = RIGHT_SIDE_PANEL_WIDTH as f32 - LAYER_BUFFER;

                let mut active_layer_index = None;
                let mut layer_ops = Vec::new();
                for (layer_index, (state, image)) in editor.image_mut().layers_mut().iter_mut().enumerate() {
                    if state != &LayerState::Deleted {
                        let position = Position::new(view_width as f32 + LAYER_BUFFER, layer_offset);
                        let position = image_area_transform.transform_point(position);
                        let layer_height = layer_width * (image.height() as f32 / image.width() as f32);

                        let bounding_rectangle = Rectangle::new(position.x, position.y, layer_width, layer_height);
                        if bounding_rectangle.contains(&mouse_position) {
                            match button {
                                MouseButton::Button1 => {
                                    active_layer_index = Some(layer_index);
                                }
                                MouseButton::Button2 => {
                                    if state == &LayerState::Visible {
                                        layer_ops.push(LayeredImageOperation::SetLayerState(layer_index, LayerState::Hidden));
                                    } else if state == &LayerState::Hidden {
                                        layer_ops.push(LayeredImageOperation::SetLayerState(layer_index, LayerState::Visible));
                                    }
                                }
                                _ => {}
                            }
                        }

                        layer_offset += layer_height + LAYER_SPACING;
                    }
                }

                if let Some(active_layer_index) = active_layer_index {
                    layer_ops.push(LayeredImageOperation::SetActiveLayer(active_layer_index));
                }

                for layer_op in layer_ops {
                    editor.apply_layer_op(layer_op);
                }

            }
            _ => {}
        }
    }

    pub fn render(&self,
                  transform: &Matrix4<f32>,
                  renders: &Renders,
                  editor: &Editor,
                  view_width: u32,
                  background_transparent_texture: &Texture,
                  image_area_transform: &Matrix4<f32>) {
        let mut layer_offset = LAYER_BUFFER;
        let layer_width = RIGHT_SIDE_PANEL_WIDTH as f32 - LAYER_BUFFER;

        let active_layer_index = editor.active_layer_index();
        for (layer_index, (state, image)) in editor.image().layers().iter().enumerate() {
            if state != &LayerState::Deleted {
                let position = Position::new(view_width as f32 + LAYER_BUFFER, layer_offset);
                let layer_height = layer_width * (image.height() as f32 / image.width() as f32);

                let mut layer_color = None;
                if active_layer_index == layer_index {
                    layer_color = Some(Color4::new(0, 148, 255, 64));
                }

                if state == &LayerState::Hidden {
                    match layer_color {
                        Some(current_layer_color) => {
                            layer_color = Some(blend(&current_layer_color, &Color4::new(255, 0, 0, 64)));
                        }
                        None => {
                            layer_color = Some(Color4::new(255, 0, 0, 64));
                        }
                    }
                }

                if let Some(layer_color) = layer_color {
                    renders.solid_rectangle_render.render(
                        renders.solid_rectangle_render.shader(),
                        &(transform * image_area_transform),
                        Position::new(position.x - LAYER_BUFFER, position.y - LAYER_BUFFER),
                        Size::new(layer_width + LAYER_BUFFER, layer_height + LAYER_BUFFER * 2.0),
                        layer_color
                    );
                }

                renders.texture_render.render_sized(
                    renders.texture_render.shader(),
                    &(transform * image_area_transform),
                    background_transparent_texture,
                    position,
                    layer_width,
                    layer_height,
                    Some(Rectangle::new(
                        0.0,
                        0.0,
                        layer_width,
                        layer_height
                    ))
                );

                renders.texture_render.render_sized(
                    renders.texture_render.shader(),
                    &(transform * image_area_transform),
                    image.get_texture(),
                    position,
                    layer_width,
                    layer_height,
                    None
                );

                layer_offset += layer_height + LAYER_SPACING;
            }
        }
    }
}