use std::collections::HashMap;
use std::os::raw::c_void;
use std::cell::{RefCell};

use cgmath::{Point2};
use std::rc::Rc;

pub struct FontCharacter {
    pub size: Point2<i32>,
    pub bearing: Point2<i32>,

    pub advance_x: f32,
    pub line_height: f32,

    pub texture_top: f32,
    pub texture_left: f32,
    pub texture_bottom: f32,
    pub texture_right: f32
}

pub struct Font {
    filename: String,
    size: u32,
    characters: Vec<u32>,
    font_map: FontMap,
    line_height: f32
}

impl Font {
    pub fn new(filename: &str, size: u32) -> Option<Font> {
        let characters: Vec<u32> = (0..255).map(|x| x as u32).collect();
        let font_map = FontMap::new(filename, size, &characters)?;

        let mut line_height = 0.0f32;
        for character in font_map.characters.values() {
            line_height = character.line_height;
            break;
        }

        Some(
            Font {
                filename: filename.to_string(),
                size,
                characters,
                font_map,
                line_height
            }
        )
    }

    pub fn texture_id(&self) -> u32 {
        return self.font_map.texture_id;
    }

    pub fn get_only(&self, character: char) -> Option<&FontCharacter> {
        self.font_map.characters.get(&character)
    }

    pub fn get(&mut self, character: char) -> Option<&FontCharacter> {
        if !self.font_map.characters.contains_key(&character) {
            self.characters.push(character as u32);
            self.font_map = FontMap::new(&self.filename, self.size, &self.characters).unwrap();
            println!("Re-created font map.");
        }

        self.get_only(character)
    }

    pub fn line_height(&self) -> f32 {
        self.line_height
    }

    pub fn line_width(&mut self, text: &str) -> f32 {
        let mut line_width = 0.0;
        for character in text.chars() {
            let font_character = self.get(character).unwrap();
            line_width += font_character.advance_x;
        }

        line_width
    }
}

pub type FontRef = Rc<RefCell<Font>>;

struct FontMap {
    texture_id: u32,
    texture_width: u32,
    texture_height: u32,
    characters: HashMap<char, FontCharacter>
}

impl FontMap {
    fn new(filename: &str, size: u32, characters: &Vec<u32>) -> Option<FontMap> {
        let library = freetype::Library::init().unwrap();
        let face = library.new_face(filename, 0).ok()?;
        face.set_pixel_sizes(0, size).unwrap();

        let mut font_characters = HashMap::new();

        let mut max_character_size = size;
        for character in characters {
            face.load_char(*character as usize, freetype::face::LoadFlag::RENDER).unwrap();
            let glyph = face.glyph();
            let bitmap = glyph.bitmap();
            max_character_size = std::cmp::max(max_character_size, bitmap.width() as u32);
            max_character_size = std::cmp::max(max_character_size, bitmap.rows() as u32);
        }

        let texture_width = max_character_size * characters.len() as u32;
        let texture_height = max_character_size;

        let mut buffer = vec![0; (texture_width * texture_height) as usize];
        let characters_per_row = characters.len();

        for (character_index, character) in characters.iter().enumerate() {
            face.load_char(*character as usize, freetype::face::LoadFlag::RENDER).unwrap();
            let glyph = face.glyph();
            let bitmap = glyph.bitmap();

            let character_width = bitmap.width() as usize;
            let character_height = bitmap.rows() as usize;

            let character_offset = character_index * max_character_size as usize;
            for y in 0..character_height {
                for x in 0..character_width {
                    buffer[y * (max_character_size as usize * characters_per_row) + x + character_offset] = bitmap.buffer()[y * character_width + x];
                }
            }

            let character_size = Point2::new(character_width as i32, character_height as i32);
            font_characters.insert(
                std::char::from_u32(*character).unwrap(),
                FontCharacter {
                    size: character_size,
                    bearing: Point2::new(glyph.bitmap_left(), glyph.bitmap_top()),

                    advance_x: glyph.advance().x as f32 / 64.0,
                    line_height: glyph.metrics().vertAdvance as f32 / 64.0,

                    texture_top: 0.0,
                    texture_left: character_offset as f32 / texture_width as f32,
                    texture_bottom: character_size.y as f32 / max_character_size as f32,
                    texture_right: (character_offset as f32 + character_size.x as f32) / texture_width as f32
                }
            );
        }

        let mut texture_id = 0;

        unsafe {
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1); // Disable byte-alignment restriction

            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RED as i32,
                texture_width as i32,
                texture_height as i32,
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                &buffer[0] as *const u8 as *const c_void);
        }

        Some(
            FontMap {
                texture_id,
                texture_width,
                texture_height,
                characters: font_characters
            }
        )
    }
}

impl Drop for FontMap {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture_id);
        }
    }
}
