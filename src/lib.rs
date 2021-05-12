#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(arbitrary_self_types)]
#![allow(unused)]

pub mod client;
pub mod control;
pub mod data;
pub mod epochs;
pub mod executor;
pub mod loops;
pub mod pipeline;
pub mod port;
pub mod sink;
pub mod source;
pub mod stream;
pub mod task;
pub mod timer;
pub mod transform;

pub mod prelude {
    pub use crate::client::*;
    pub use crate::control::*;
    pub use crate::data::*;
    pub use crate::epochs::*;
    pub use crate::executor::*;
    pub use crate::loops::*;
    pub use crate::pipeline::*;
    pub use crate::port::*;
    pub use crate::sink::*;
    pub use crate::source::*;
    pub use crate::stream::*;
    pub use crate::transform::*;
    // pub use crate::task;
    pub use kompact::prelude::*;
    pub use std::any::Any;
    pub use std::any::TypeId;
    pub use std::marker::PhantomData;
    pub use std::sync::Arc;
}

pub mod macros {
    pub use arctime_macros::rewrite;
}
