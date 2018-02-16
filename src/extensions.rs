pub trait Scratch<TOut> {
    type Result;

    fn create_scratchpad(&self, default: TOut) -> Self::Result;
}
