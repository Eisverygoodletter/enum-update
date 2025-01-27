pub trait EnumUpdate<U> {
    fn apply(&mut self, update: U);
}