pub mod adb;
pub mod winapi;

use squire_error::Result;

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub trait InputBackend: Send + Sync {
    fn click(&self, point: Point) -> Result<()>;
    fn double_click(&self, point: Point) -> Result<()>;
    fn drag(&self, from: Point, to: Point) -> Result<()>;
    fn key_press(&self, key: u16) -> Result<()>;
}
