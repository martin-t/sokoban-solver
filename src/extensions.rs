use level::MapCell;


pub trait Scratch<Inner> {
    type Result;

    fn create_scratchpad(&self, default: Inner) -> Self::Result;
}

impl<Inner: Copy> Scratch<Inner> for Vec<Vec<Inner>> {
    type Result = Vec<Vec<Inner>>;

    fn create_scratchpad(&self, default: Inner) -> Self::Result {
        let mut scratch = Vec::new();

        for row in self.iter() {
            scratch.push(vec![default; row.len()]);
        }

        scratch
    }
}

// TODO not used - unify with MyVec2d
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
