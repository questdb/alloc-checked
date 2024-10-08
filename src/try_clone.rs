/// A variant of the `Clone` trait which can fail.
pub trait TryClone: Sized {
    type Error;

    fn try_clone(&self) -> Result<Self, Self::Error>;
    fn try_clone_from(&mut self, source: &Self) -> Result<(), Self::Error> {
        *self = source.try_clone()?;
        Ok(())
    }
}
