
#[derive(Copy, Clone)]
pub enum MaybeInf<T> {
    Inf,
    NonInf(T),
}

