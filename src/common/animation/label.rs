pub trait Label: 'static + Send + Sync + std::fmt::Debug + bevy::utils::label::DynHash {
    fn dyn_clone(&self) -> Box<dyn Label>;
}

impl PartialEq for dyn Label {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other.as_dyn_eq())
    }
}

impl Eq for dyn Label {}

impl std::hash::Hash for dyn Label {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

impl Clone for Box<dyn Label> {
    fn clone(&self) -> Self {
        (&**self).dyn_clone()
    }
}

impl<T: 'static + Send + Sync + Clone + Eq + std::fmt::Debug + std::hash::Hash> Label for T {
    fn dyn_clone(&self) -> std::boxed::Box<dyn Label> {
        std::boxed::Box::new(std::clone::Clone::clone(self))
    }
}

pub type BoxedLabel = Box<dyn Label>;

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Clone, Hash, Debug, PartialEq, Eq)]
    pub enum ExampleLabel { A, B }
    #[test] pub fn boxed_label(){
        let a: BoxedLabel = ExampleLabel::A.dyn_clone();
        let b: BoxedLabel = ExampleLabel::A.dyn_clone();
        let c: BoxedLabel = ExampleLabel::B.dyn_clone();
        let d: BoxedLabel = a.clone();
        assert!(&a == &b);
        assert!(&a != &c);
        assert!(&b == &d);
        assert!(&d != &c);
        let e = a.as_any().downcast_ref::<ExampleLabel>().unwrap();
        assert!(*e == ExampleLabel::A)
    }
}