use thiserror::Error;

pub trait IteratorExt: Iterator {
    fn singleton(mut self) -> Result<Self::Item, SingletonError<Self::Item>>
    where
        Self: Sized,
    {
        match self.next() {
            Some(a) => match self.next() {
                Some(b) => Err(SingletonError::MultipleItems([a, b])),
                None => Ok(a),
            },
            None => Err(SingletonError::ZeroItems),
        }
    }
}

impl<T> IteratorExt for T where T: Iterator {}

#[derive(Error, Debug)]
pub enum SingletonError<T> {
    #[error("expected one item, but found none")]
    ZeroItems,

    #[error("expected one item, but found at least two")]
    MultipleItems([T; 2]),
}
