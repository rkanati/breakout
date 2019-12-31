
pub trait Dilate<By> {
    type Output;
    fn expand  (&self, by: By) -> Self::Output;
    fn contract(&self, by: By) -> Self::Output;
}

