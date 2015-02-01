extern crate freetype;
extern crate image;

use ::glium;

use std::option::Option;
use std::collections::{HashMap};
use std::rc::{Rc, Weak};
use std::vec::Vec;
use std::boxed::Box;
use std::default::Default;
use std::cmp::{Eq};
use std::collections::hash_map::{Entry};
use std::num::{SignedInt, Float};
use std::cell::{RefCell, Cell};

use math3d::{Matrix4, Vector3};

use glium::texture::UncompressedFloatFormat;
use glium::texture::ClientFormat;
use glium::Surface;
use glium::index_buffer::TriangleStrip;
use glium::LinearBlendingFactor;

use self::freetype::face::Face;
use self::freetype::glyph::Glyph;
use self::freetype::bitmap_glyph::BitmapGlyph;
use self::freetype::bitmap::PixelMode;
use self::freetype::bitmap::Bitmap;
use self::freetype::render_mode::RenderMode;
use self::freetype::face::KerningMode::KerningDefault;

#[uniforms]
struct Uniforms<'a> {
    matrix: [[f32; 4]; 4],
    texture: &'a glium::texture::Texture2d,
    color: [f32; 4],
}

#[derive(Eq, PartialEq, Hash, Copy)]
struct CacheKey {
    face_index: usize,
    size: isize,
    ch: char,
}

struct CacheValue {
    texture: glium::texture::Texture2d,
    glyph: Glyph,
    bitmap_glyph: BitmapGlyph,
    glyph_index: u32,
}

#[vertex_format]
#[derive(Copy)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

pub struct Gui {
    text_renderer: Rc<TextRenderer>,
    width: Cell<i32>,
    height: Cell<i32>,
    projection: Cell<Matrix4>,
    widget_set: Rc<WidgetSet>,
    set: RefCell<HashMap<u64, Weak<Widget>>>,
    next_id: Cell<u64>,
}

impl Gui {
    pub fn new(display: glium::Display) -> Self {
        let mut gui = Gui {
            width: 0,
            height: 0,
            projection: Matrix4::identity(),
            text_renderer: Rc::new(TextRenderer::new(display)),
            widget_set: Rc::new(WidgetSet::new()),
            next_id: 0,
        };
        gui.resize();
        gui
    }

    pub fn resize(&self) {
        let (w, h) = self.text_renderer.display.get_framebuffer_dimensions();
        self.width.set(w);
        self.height.set(h);
        self.projection.set(Matrix4::ortho(0.0, w as f32, h as f32, 0.0));
    }

    pub fn load_face(&self, path: &Path) -> Result<usize, freetype::error::Error> {
        self.text_renderer.load_face(path)
    }

    pub fn create_label(&self, face_index: usize) -> Rc<Widget<Label>> {
        let texture = glium::texture::Texture2d::new_empty(&self.text_renderer.display,
            UncompressedFloatFormat::U8U8U8U8, 16, 16);
        let vertex_buffer = glium::VertexBuffer::new(&self.text_renderer.display, vec![
            Vertex { position: [ 0.0,     0.0,     0.0], tex_coords: [0.0, 0.0] },
            Vertex { position: [ 0.0,     16.0,    0.0], tex_coords: [0.0, 1.0] },
            Vertex { position: [ 16.0,    0.0,     0.0], tex_coords: [1.0, 0.0] },
            Vertex { position: [ 16.0,    16.0,    0.0], tex_coords: [1.0, 1.0] }
        ]);
        let label = Label {
            renderer: self.text_renderer.clone(),
            face_index: face_index,
            text: String::from_str("label"),
            size: 16,
            texture: texture,
            color: [1.0, 1.0, 1.0, 1.0],
            vertex_buffer: vertex_buffer,
        };
        let id = self.make_id();
        let widget = Rc::new(Widget::new(label, id, self.widget_set.clone()));
        let widget_set = self.widget_set.set.borrow_mut();
        widget_set.insert(id, widget.clone().downgrade());
        widget
    }
    pub fn draw_frame(&self) {
        let set = self.widget_set.set.borrow();
        let projection = self.projection.get();
        let mut target = self.display.draw();
        target.clear_color(0.3, 0.3, 0.3, 1.0);
        for (id, widget_opt) in set.iter() {
            let widget = widget_opt.upgrade();
            widget.draw(&mut target, &projection);
        }
        target.finish();
    }
    fn make_id(&self) -> u64 {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        id
    }
}

trait Drawable {
    fn anchor_pos(&self) -> (f32, f32);
    fn draw(&self, frame: &mut glium::Frame, matrix: &Matrix4);
}

pub struct Widget<T> {
    item: RefCell<T>,
    pos: (i32, i32),
    scale: (f32, f32),
    rotation: f32,
    visible: bool,
}

impl<T> Widget<T> {
    fn new(item: T, id: usize, widget_set: Rc<WidgetSet>) -> Widget<T> {
        Widget {
            id: id,
            widget_set: widget_set,
            item: RefCell::new(item),
            pos: (0, 0),
            scale: (1.0, 1.0),
            rotation: 0.0,
            visible: true,
        }
    }
    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.pos = (x, y);
    }
    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.scale = (x, y);
    }
    pub fn pos(&self) {
        self.pos
    }
    pub fn scale(&self) {
        self.scale
    }
    /// don't save this reference
    pub fn item(&self) -> &mut T {
        self.item.borrow_mut()
    }
    fn draw(&self, frame: &mut glium::Frame, projection: &Matrix4) {
        if !self.visible {return}

        let item = self.item.borrow();
        let (pos_x, pos_y) = self.pos;
        let (scale_x, scale_y) = self.scale;
        let (anchor_x, anchor_y) = item.anchor_pos();
        let model = Matrix4::identity()
            .translate(pos_x as f32, pos_y as f32, 0.0)
            .scale(scale_x as f32, scale_y as f32, 0.0)
            .rotate(self.rotation, &Vector3::new(0.0, 0.0, 1.0))
            .translate(anchor_x, anchor_y);
        let mvp = projection.mult(model);
        item.draw(frame, &mvp);
    }
}

pub struct TextRenderer {
    library: freetype::Library,
    display: glium::Display,
    cache: RefCell<HashMap<CacheKey, Rc<Box<CacheValue>>>>,
    index_buffer: glium::IndexBuffer,
    program_gray: glium::Program,
    program_color: glium::Program,
    draw_params: glium::DrawParameters,
    face_list: RefCell<Vec<freetype::Face>>,
}

impl TextRenderer {
    fn new(display: glium::Display) -> TextRenderer {
        let index_buffer = glium::IndexBuffer::new(&display, TriangleStrip(vec![0 as u16, 1, 2, 3]));

        let program_gray = glium::Program::from_source(&display, r"
            #version 110
            uniform mat4 matrix;
            attribute vec3 position;
            attribute vec2 tex_coords;
            varying vec2 v_tex_coords;
            void main() {
                gl_Position = vec4(position, 1.0) * matrix;
                v_tex_coords = tex_coords;
            }
        ", r"
            #version 110
            uniform sampler2D texture;
            uniform vec4 color;
            varying vec2 v_tex_coords;
            void main() {
                gl_FragColor = vec4(color.rgb, color.a * texture2D(texture, v_tex_coords));
            }
        ", None).unwrap();
        let program_color = glium::Program::from_source(&display, r"
            #version 110
            uniform mat4 matrix;
            attribute vec3 position;
            attribute vec2 tex_coords;
            varying vec2 v_tex_coords;
            void main() {
                gl_Position = vec4(position, 1.0) * matrix;
                v_tex_coords = tex_coords;
            }
        ", r"
            #version 110
            uniform sampler2D texture;
            uniform vec4 color;
            varying vec2 v_tex_coords;
            void main() {
                gl_FragColor = vec4(color.rgb, texture2D(texture, v_tex_coords).a * color.a);
            }
        ", None).unwrap();

        let library = freetype::Library::init().unwrap();
        TextRenderer {
            library: library,
            display: display,
            program_color: program_color,
            program_gray: program_gray,
            cache: RefCell::new(HashMap::new()),
            face_list: RefCell::new(Vec::new()),
            index_buffer: index_buffer,
            draw_params: glium::DrawParameters {
                blending_function: Option::Some(glium::BlendingFunction::Addition{
                    source: LinearBlendingFactor::SourceAlpha,
                    destination: LinearBlendingFactor::OneMinusSourceAlpha,
                }),
                .. Default::default()
            },
        }
    }

    fn load_face(&self, path: &Path) -> Result<usize, freetype::error::Error> {
        match self.library.new_face(path, 0) {
            Ok(face) => {
                let mut face_list_mut = self.face_list.borrow_mut();
                let index = face_list_mut.len();
                face_list_mut.push(face);
                Ok(index)
            },
            Err(err) => Err(err),
        }
    }

    fn get_cache_entry(&self, key: CacheKey) -> Rc<Box<CacheValue>> {
        let mut cache = self.cache.borrow_mut();
        match cache.entry(key) {
            Entry::Occupied(entry) => entry.into_mut().clone(),
            Entry::Vacant(entry) => {
                let face_list = self.face_list.borrow();
                let face = &face_list[key.face_index];
                face.set_char_size(0, key.size * 64, 0, 0).unwrap();
                let glyph_index = face.get_char_index(key.ch as usize);
                face.load_glyph(glyph_index, freetype::face::RENDER).unwrap();
                let glyph_slot = face.glyph();
                let glyph = glyph_slot.get_glyph().unwrap();
                let bitmap_glyph = glyph.to_bitmap(RenderMode::Normal, Option::None).unwrap();
                let texturable_bitmap = TexturableBitmap::new(bitmap_glyph.bitmap());
                let texture = glium::texture::Texture2d::new(&self.display, texturable_bitmap);
                let cache_value = Rc::new(Box::new(CacheValue {
                    texture: texture,
                    glyph: glyph,
                    bitmap_glyph: bitmap_glyph,
                    glyph_index: glyph_index,
                }));
                entry.insert(cache_value).clone()
            },
        }
    }
}

pub struct Label {
    renderer: Rc<TextRenderer>,
    text: String,
    face_index: usize,
    size: isize,
    texture: glium::texture::Texture2d,
    color: [f32; 4],
    vertex_buffer: glium::VertexBuffer<Vertex>,
}

impl Drawable for Label {
    fn draw(&self, frame: &mut glium::Frame, matrix: &Matrix4) {
        let uniforms = Uniforms {
            matrix: *matrix.as_array(),
            texture: &self.texture,
            color: self.color,
        };
        frame.draw(&self.vertex_buffer, &self.renderer.index_buffer,
                   &self.renderer.program_color, uniforms,
                   &self.renderer.draw_params).ok().unwrap();
    }
}

impl Label {
    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn set_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.color = [red, green, blue, alpha];
    }

    pub fn update(&mut self) {
        // one pass to determine width and height
        // pen_x and pen_y are on the baseline. the char can go lower than it
        let mut pen_x: f32 = 0.0;
        let mut pen_y: f32 = 0.0;
        let mut previous_glyph_index = 0;
        let mut first = true;
        let mut above_size: f32 = 0.0; // pixel count above the baseline
        let mut below_size: f32 = 0.0; // pixel count below the baseline
        let mut bounding_width = 0.0;
        for ch in self.text.chars() {
            let key = CacheKey {
                face_index: self.face_index,
                size: self.size,
                ch: ch,
            };
            let cache_entry = self.renderer.get_cache_entry(key);

            if first {
                first = false;
                let face_list = self.renderer.face_list.borrow();
                let face = &face_list[self.face_index];
                let kerning = face.get_kerning(previous_glyph_index,
                                                       cache_entry.glyph_index,
                                                       KerningDefault).unwrap();
                pen_x += (kerning.x as f32) / 64.0;
            }

            let bmp_start_left = cache_entry.bitmap_glyph.left() as f32;
            let bmp_start_top = cache_entry.bitmap_glyph.top() as f32;
            let bitmap = cache_entry.bitmap_glyph.bitmap();
            let bmp_width = bitmap.width() as f32;
            let bmp_height = bitmap.rows() as f32;
            let right = (pen_x + bmp_start_left + bmp_width).ceil();
            let this_above_size = pen_y + bmp_start_top;
            let this_below_size = bmp_height - this_above_size;
            above_size = if this_above_size > above_size {this_above_size} else {above_size};
            below_size = if this_below_size > below_size {this_below_size} else {below_size};
            bounding_width = right;

            previous_glyph_index = cache_entry.glyph_index;
            pen_x += (cache_entry.glyph.advance_x() as f32) / 65536.0;
            pen_y += (cache_entry.glyph.advance_y() as f32) / 65536.0;
        }
        let bounding_height = (above_size + below_size).ceil();

        self.texture = glium::texture::Texture2d::new_empty(&self.renderer.display,
            UncompressedFloatFormat::U8U8U8U8, bounding_width as u32, bounding_height as u32);
        self.texture.as_surface().clear_color(0.0, 0.0, 0.0, 0.0);

        self.vertex_buffer = glium::VertexBuffer::new(&self.renderer.display, vec![
            Vertex { position: [ 0.0,            0.0,             0.0], tex_coords: [0.0, 0.0] },
            Vertex { position: [ 0.0,            bounding_height, 0.0], tex_coords: [0.0, 1.0] },
            Vertex { position: [ bounding_width, 0.0,             0.0], tex_coords: [1.0, 0.0] },
            Vertex { position: [ bounding_width, bounding_height, 0.0], tex_coords: [1.0, 1.0] }
        ]);

        // second pass to render to texture
        pen_x = 0.0;
        pen_y = 0.0;
        previous_glyph_index = 0;
        first = true;
        let projection = Matrix4::ortho(0.0, bounding_width, 0.0, bounding_height);
        for ch in self.text.chars() {
            let key = CacheKey {
                face_index: self.face_index,
                size: self.size,
                ch: ch,
            };
            let cache_entry = self.renderer.get_cache_entry(key);

            if first {
                first = false;
                let face_list = self.renderer.face_list.borrow();
                let face = &face_list[self.face_index];
                let kerning = face.get_kerning(previous_glyph_index,
                                                       cache_entry.glyph_index,
                                                       KerningDefault).unwrap();
                pen_x += (kerning.x as f32) / 64.0;
            }

            let bmp_start_left = cache_entry.bitmap_glyph.left() as f32;
            let bmp_start_top = cache_entry.bitmap_glyph.top() as f32;
            let bitmap = cache_entry.bitmap_glyph.bitmap();
            let bmp_width = bitmap.width() as f32;
            let bmp_height = bitmap.rows() as f32;
            let left = pen_x + bmp_start_left;
            let top = above_size - bmp_start_top;
            let model = Matrix4::identity().translate(left, top, 0.0);
            let mvp = projection.mult(&model);
            let texture = &cache_entry.texture;
            let uniforms = Uniforms {
                matrix: *mvp.as_array(),
                texture: texture,
                color: [0.0, 0.0, 0.0, 1.0],
            };
            let vertex_buffer = glium::VertexBuffer::new(&self.renderer.display, vec![
                Vertex { position: [ 0.0,        0.0,           0.0], tex_coords: [0.0, 0.0] },
                Vertex { position: [ 0.0,        bmp_height,    0.0], tex_coords: [0.0, 1.0] },
                Vertex { position: [ bmp_width,  0.0,           0.0], tex_coords: [1.0, 0.0] },
                Vertex { position: [ bmp_width,  bmp_height,    0.0], tex_coords: [1.0, 1.0] }
            ]);

            self.texture.as_surface().draw(&vertex_buffer, &self.renderer.index_buffer,
                                           &self.renderer.program_gray, uniforms,
                                           &Default::default()).ok().unwrap();

            previous_glyph_index = cache_entry.glyph_index;
            pen_x += (cache_entry.glyph.advance_x() as f32) / 65536.0;
            pen_y += (cache_entry.glyph.advance_y() as f32) / 65536.0;
        }
    }
}

struct TexturableBitmap {
    bitmap: Bitmap,
}

impl TexturableBitmap {
    fn new(bitmap: Bitmap) -> Self {
        TexturableBitmap {
            bitmap: bitmap,
        }
    }
}

impl glium::texture::Texture2dData for TexturableBitmap {
    type Data = u8;

    fn get_format() -> ClientFormat {
        ClientFormat::U8
    }

    fn get_dimensions(&self) -> (u32, u32) {
        (self.bitmap.width() as u32, self.bitmap.rows() as u32)
    }

    fn into_vec(self) -> Vec<u8> {
        let signed_pitch = self.bitmap.pitch();
        enum Flow {
            Down,
            Up,
        }
        let flow = if signed_pitch >= 0 {Flow::Down} else {Flow::Up};
        let pitch = signed_pitch.abs();
        match self.bitmap.pixel_mode() {
            PixelMode::Gray => {
                match flow {
                    Flow::Down => {
                        if pitch == self.bitmap.width() {
                            return self.bitmap.buffer().to_vec();
                        } else {
                            panic!("unsupported pitch != width");
                        }
                    },
                    Flow::Up => panic!("flow up unsupported"),
                }
            },
            PixelMode::Bgra => panic!("Bgra pixel mode"),
            _ => panic!("unexpected pixel mode: {:?}", self.bitmap.pixel_mode()),
        }
    }

    fn from_vec(buffer: Vec<u8>, width: u32) -> Self {
        let x = buffer[0] as u32 + width; // to get rid of unused warning
        panic!("why do we need from_vec? {}", x);
    }
}
