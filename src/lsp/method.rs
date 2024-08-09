pub mod hover;
pub mod didchange;

pub enum TryResult<T, R> {
    Receive(T),
    Yet(R),
}
