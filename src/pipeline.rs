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
    pub(crate) tasks: Arc<RefCell<Vec<Arc<dyn AbstractComponent<Message = TaskMessage>>>>>,
}

impl<S: SystemHandle> Pipeline<S> {
    pub(crate) fn finalize(self) {
        for task in self.tasks.borrow_mut().drain(..).rev() {
            self.system.start(&task);
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
            tasks,
        }
    }
}

impl<S: StateReqs, I: EventReqs, O: EventReqs> Task<S, I, O> {
    pub(crate) fn pipeline(&self) -> Pipeline<impl SystemHandle> {
        let system = self.ctx.system();
        let client = system.create(Client::new);
        let tasks = Arc::new(RefCell::new(Vec::new()));
        Pipeline {
            system,
            client,
            tasks,
        }
    }
}
