#![allow(clippy::type_complexity)]
use crate::control::*;
use kompact::component::AbstractComponent;
use kompact::config::ConfigEntry;
use kompact::prelude::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use crate::client::*;
use crate::data::*;
use crate::executor::*;
use crate::task::*;

pub(crate) struct Pipeline<S: SystemHandle> {
    pub(crate) system: S,
    pub(crate) client: Arc<Component<Client>>,
    pub(crate) starters: Arc<RefCell<Vec<Box<dyn FnOnce() + 'static>>>>,
}

impl<S: SystemHandle> Pipeline<S> {
    pub(crate) fn finalize(self) {
        for starter in self.starters.borrow_mut().drain(..).rev() {
            starter();
        }
    }
}

impl Executor {
    pub(crate) fn pipeline(&self) -> Pipeline<impl SystemHandle> {
        let tasks = Arc::new(RefCell::new(Vec::new()));
        let client = self.system.create(Client::new);
        let system = client.on_definition(|c| c.ctx().system());
        Pipeline {
            system,
            client,
            starters: tasks,
        }
    }
}

impl<S: StateReqs, I: EventReqs, O: EventReqs, R: EventReqs> Task<S, I, O, R> {
    pub(crate) fn pipeline(&self) -> Pipeline<impl SystemHandle> {
        let system = self.ctx.system();
        let client = system.create(Client::new);
        let tasks = Arc::new(RefCell::new(Vec::new()));
        Pipeline {
            system,
            client,
            starters: tasks,
        }
    }
}
