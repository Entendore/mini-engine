//! Rendering concern: framebuffer, primitives, images, atlases, sprites,
//! font. The window backend simply presents the finished buffer.

mod atlas;
mod color;
mod font;
mod framebuffer;
mod image;
mod sprite;

pub use atlas::{Animator, Atlas};
pub use color::Color;
pub use framebuffer::Renderer;
pub use image::{Blit, Image};
pub use sprite::{animate_sprites, draw_sprites, AnimatedSprite, SpriteImage};
