
pub trait Succ {
    fn succ(&self) -> Option<Self>
    where
        Self: Sized;
    fn first() -> Self;

    fn succ_iter() -> SuccIter<Self>
    where
        Self: Clone,
    {
        SuccIter::new()
    }
}

#[derive(Debug, Clone)]
pub struct SuccIter<T: Succ + Clone>(Option<T>);

impl<T: Succ + Clone> SuccIter<T> {
    pub fn new() -> Self {
        Self(Some(T::first()))
    }
}

impl<T: Succ + Clone + Sized> Iterator for SuccIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.0.clone();
        self.0 = self.0.as_ref().and_then(|x| x.succ());
        current
    }
}
