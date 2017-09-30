use std::fmt::Display;

use data::MapCell;

pub trait Vec2d {
    fn print(&self);
}

impl Vec2d for Vec<Vec<bool>> {
    fn print(&self) {
        for row in self.iter() {
            for &cell in row.iter() {
                print!("{}", if cell { 1 } else { 0 });
            }
            println!();
        }
    }
}

impl Vec2d for Vec<Vec<MapCell>> {
    fn print(&self) {
        for row in self.iter() {
            for &cell in row.iter() {
                print!("{}", cell.to_string());
            }
            println!();
        }
    }
}
