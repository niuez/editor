pub mod hover;

pub enum TryResult<T, R> {
    Receive(T),
    Yet(R),
}
