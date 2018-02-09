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
