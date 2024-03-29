mod quad;

use glam::{Vec2, Vec4};
use serde::Deserialize;

pub use quad::*;

#[derive(Deserialize, Debug, Clone)]
pub struct Scene {
    pub layers: Vec<Layer>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            layers: vec![Default::default()],
        }
    }

    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    pub fn with_layer(mut self, layer: Layer) -> Self {
        self.add_layer(layer);
        self
    }

    pub fn layer(&self) -> &Layer {
        self.layers.last().unwrap()
    }

    pub fn layer_mut(&mut self) -> &mut Layer {
        self.layers.last_mut().unwrap()
    }

    pub fn with_clip(mut self, clip: Vec4) -> Self {
        self.layer_mut().clip = Some(clip);
        self
    }

    pub fn with_blur(mut self, radius: f32) -> Self {
        self.layer_mut().background_blur_radius = radius;
        self
    }

    pub fn with_background(mut self, color: Vec4) -> Self {
        self.layer_mut().background_color = Some(color);
        self
    }

    pub fn with_font(mut self, font_name: String) -> Self {
        self.layer_mut().font_name = font_name;
        self
    }

    pub fn font(&self) -> &str {
        self.layer().font_name.as_str()
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.layer_mut().font_size = size;
        self
    }

    pub fn font_size(&self) -> f32 {
        self.layer().font_size
    }

    pub fn add_quad(&mut self, quad: Quad) {
        self.layer_mut().add_quad(quad);
    }

    pub fn with_quad(mut self, quad: Quad) -> Self {
        self.add_quad(quad);
        self
    }

    pub fn add_text(&mut self, text: Text) {
        self.layer_mut().add_text(text);
    }

    pub fn with_text(mut self, text: Text) -> Self {
        self.add_text(text);
        self
    }

    pub fn add_path(&mut self, path: Path) {
        self.layer_mut().add_path(path);
    }

    pub fn with_path(mut self, path: Path) -> Self {
        self.add_path(path);
        self
    }

    pub fn add_sprite(&mut self, sprite: Sprite) {
        self.layer_mut().add_sprite(sprite);
    }

    pub fn with_sprite(mut self, sprite: Sprite) -> Self {
        self.add_sprite(sprite);
        self
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Layer {
    #[serde(default)]
    pub clip: Option<Vec4>,
    #[serde(default)]
    pub background_blur_radius: f32,
    #[serde(default)]
    pub background_color: Option<Vec4>,
    #[serde(default = "default_font")]
    pub font_name: String,
    #[serde(default = "default_size")]
    pub font_size: f32,
    #[serde(default)]
    pub quads: Vec<Quad>,
    #[serde(default)]
    pub texts: Vec<Text>,
    #[serde(default)]
    pub paths: Vec<Path>,
    #[serde(default)]
    pub sprites: Vec<Sprite>,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            clip: None,
            background_blur_radius: 0.0,
            background_color: Some(Vec4::new(1.0, 1.0, 1.0, 1.0)),
            font_name: "Courier New".to_string(),
            font_size: 16.0,
            quads: Vec::new(),
            texts: Vec::new(),
            paths: Vec::new(),
            sprites: Vec::new(),
        }
    }
}

fn default_font() -> String {
    "Courier New".to_string()
}

fn default_size() -> f32 {
    16.0
}

impl Layer {
    pub fn with_clip(mut self, clip: Vec4) -> Self {
        self.clip = Some(clip);
        self
    }

    pub fn set_clip(&mut self, clip: Vec4) {
        self.clip = Some(clip);
    }

    pub fn with_blur(mut self, radius: f32) -> Self {
        self.background_blur_radius = radius;
        self
    }

    pub fn set_blur(&mut self, radius: f32) {
        self.background_blur_radius = radius;
    }

    pub fn with_background(mut self, color: Vec4) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn set_background(&mut self, color: Vec4) {
        self.background_color = Some(color);
    }

    pub fn with_font(mut self, font_name: String) -> Self {
        self.font_name = font_name;
        self
    }

    pub fn set_font(&mut self, font_name: String) {
        self.font_name = font_name;
    }

    pub fn add_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }

    pub fn with_quad(mut self, quad: Quad) -> Self {
        self.add_quad(quad);
        self
    }

    pub fn add_text(&mut self, text: Text) {
        self.texts.push(text);
    }

    pub fn with_text(mut self, text: Text) -> Self {
        self.add_text(text);
        self
    }

    pub fn add_path(&mut self, path: Path) {
        self.paths.push(path);
    }

    pub fn with_path(mut self, path: Path) -> Self {
        self.add_path(path);
        self
    }

    pub fn add_sprite(&mut self, sprite: Sprite) {
        self.sprites.push(sprite);
    }

    pub fn with_sprite(mut self, sprite: Sprite) -> Self {
        self.add_sprite(sprite);
        self
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Text {
    pub text: String,
    pub bottom_left: Vec2,
    pub size: f32,
    pub color: Vec4,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default = "default_subpixel")]
    pub subpixel: bool,
}

fn default_subpixel() -> bool {
    true
}

impl Text {
    pub fn new(text: String, bottom_left: Vec2, size: f32, color: Vec4) -> Self {
        Self {
            text,
            bottom_left,
            size,
            color,
            bold: false,
            italic: false,
            subpixel: true,
        }
    }

    pub fn with_bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn with_italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn without_subpixel(mut self) -> Self {
        self.subpixel = false;
        self
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PathCommand {
    CubicBezierTo {
        control1: Vec2,
        control2: Vec2,
        to: Vec2,
    },
    QuadraticBezierTo {
        control: Vec2,
        to: Vec2,
    },
    LineTo {
        to: Vec2,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct Path {
    #[serde(default)]
    pub fill: Option<Vec4>,
    #[serde(default)]
    pub stroke: Option<(f32, Vec4)>,
    pub start: Vec2,
    pub commands: Vec<PathCommand>,
}

impl Path {
    pub fn new_fill(fill: Vec4, start: Vec2) -> Self {
        Self {
            fill: Some(fill),
            stroke: None,
            start,
            commands: Vec::new(),
        }
    }

    pub fn new_stroke(stroke: (f32, Vec4), start: Vec2) -> Self {
        Self {
            fill: None,
            stroke: Some(stroke),
            start,
            commands: Vec::new(),
        }
    }

    pub fn new(start: Vec2) -> Self {
        Self {
            fill: None,
            stroke: None,
            start,
            commands: Vec::new(),
        }
    }

    pub fn with_fill(mut self, fill: Vec4) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn with_stroke(mut self, stroke: (f32, Vec4)) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn cubic_bezier_to(mut self, control1: Vec2, control2: Vec2, to: Vec2) -> Self {
        self.commands.push(PathCommand::CubicBezierTo {
            control1,
            control2,
            to,
        });
        self
    }

    pub fn quadratic_bezier_to(mut self, control: Vec2, to: Vec2) -> Self {
        self.commands
            .push(PathCommand::QuadraticBezierTo { control, to });
        self
    }

    pub fn line_to(mut self, to: Vec2) -> Self {
        self.commands.push(PathCommand::LineTo { to });
        self
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Sprite {
    pub top_left: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub texture: String,
}
