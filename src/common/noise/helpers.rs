pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

pub fn cubic_lerp(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    let t2 = t * t;
    let p = (d - c) - (a - b);
    t2 * t * p + t2 * ((a - b) - p) + t * (c - a) + b
}

pub enum Interpolation {
    Linear,
    Hermite,
    Quintic
}

impl Interpolation {
    pub fn curve(&self, value: f32) -> f32 {
        match self {
            Interpolation::Linear => value,
            Interpolation::Hermite => value * value * (3. - 2. * value),
            Interpolation::Quintic => value * value * value * (value * (value * 6. - 15.) + 10.)
        }
    }
}

pub struct WeightTable<T, S> {
    entries: Vec<(T, S)>,
    total: S,
}
impl<T, S> Default for WeightTable<T, S> where S: Default {
    fn default() -> Self { Self { entries: Vec::new(), total: Default::default() } }
}
impl<T, S> WeightTable<T, S>
where S: std::ops::Add<S, Output = S> + std::ops::Sub<S, Output = S> + Clone + Default + PartialOrd
{
    pub fn total(&self) -> S { self.total.clone() }
    pub fn add(&mut self, item: T, weight: S){
        self.total = self.total.clone() + weight.clone();
        self.entries.push((item, weight));
    }
    pub fn get<'a>(&'a self, random_weight: S) -> Option<&'a T> {
        self.get_index(random_weight).map(move |index| &self.entries[index].0)
    }
    pub fn take(&mut self, random_weight: S) -> Option<T> {
        self.get_index(random_weight).map(move |index| {
            let (item, weight) = self.entries.swap_remove(index);
            self.total = self.total.clone() - weight;
            item
        })
    }
    fn get_index(&self, random_weight: S) -> Option<usize> {
        let mut partial_weight: S = Default::default();
        for (i, (_, weight)) in self.entries.iter().enumerate() {
            partial_weight = partial_weight + weight.clone();
            if random_weight < partial_weight {
                return Some(i);
            }
        }
        None
    }
}