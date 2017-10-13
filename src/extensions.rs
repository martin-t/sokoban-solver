use data::MapCell;

pub trait Vec2d {
    fn print(&self);
}

// TODO all printing to streams/formatters
impl Vec2d for Vec<Vec<bool>> {
    fn print(&self) {
        for row in self.iter() {
            for &cell in row.iter() {
                print!("{}", if cell { 1 } else { 0 });
            }
            println!();
        }
        println!();
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
        println!();
    }
}
