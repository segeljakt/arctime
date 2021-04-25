use crate::control::*;
use kompact::component::AbstractComponent;
use kompact::prelude::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use crate::client::*;

pub(crate) struct Pipeline {
    pub(crate) system: KompactSystem,
    pub(crate) client: Arc<Component<Client>>,
    pub(crate) tasks: Arc<RefCell<Vec<Arc<dyn AbstractComponent<Message = TaskMessage>>>>>,
}

impl Pipeline {
    pub(crate) fn new() -> Self {
        let system = KompactConfig::default().build().expect("system");
        let client = system.create(Client::new);
        let tasks = Arc::new(RefCell::new(Vec::new()));
        Self {
            system,
            client,
            tasks,
        }
    }
    pub(crate) fn execute(self) {
        for task in self.tasks.borrow().iter().rev() {
            self.system.start(task);
        }
        self.system.await_termination()
    }
}
