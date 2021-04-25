use kompact::prelude::*;
use std::sync::Arc;

use crate::control::*;
use crate::data::*;
use crate::pipeline::*;
use crate::port::*;
use crate::task::*;

#[derive(ComponentDefinition)]
pub(crate) struct Client {
    ctx: ComponentContext<Self>,
}

impl Client {
    pub(crate) fn new() -> Self {
        Self {
            ctx: ComponentContext::uninitialised(),
        }
    }
}

impl ComponentLifecycle for Client {}

impl Actor for Client {
    type Message = ClientMessage;

    fn receive_local(&mut self, msg: Self::Message) -> Handled {
        todo!()
    }

    fn receive_network(&mut self, msg: NetMessage) -> Handled {
        unreachable!()
    }
}
