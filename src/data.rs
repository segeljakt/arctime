use std::fmt::Debug;

pub(crate) trait EventReqs: 'static + Sync + Send + Debug + Clone {}
impl<T> EventReqs for T where T: 'static + Sync + Send + Debug + Clone {}

pub(crate) trait StateReqs: 'static + Sync + Send + Debug + Clone {}
impl<T> StateReqs for T where T: 'static + Sync + Send + Debug + Clone {}

#[derive(Debug, Clone)]
pub(crate) enum Union<L, R> {
    L(L),
    R(R),
}
