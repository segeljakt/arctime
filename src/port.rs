use kompact::prelude::*;
use std::marker::PhantomData;

use crate::data::*;

#[derive(Debug)]
pub(crate) struct OneWayPort<T>(PhantomData<T>);

impl<T: EventReqs> Port for OneWayPort<T> {
    type Indication = Never;
    type Request = T;
}
