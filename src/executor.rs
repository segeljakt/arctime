use kompact::component::AbstractComponent;
use kompact::config::ConfigEntry;
use kompact::prelude::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use crate::client::*;
use crate::control::*;
use crate::data::*;
use crate::task::*;

pub(crate) struct Executor {
    pub(crate) system: KompactSystem,
}

impl Executor {
    pub(crate) fn new() -> Self {
        Executor {
            system: KompactConfig::default().build().expect("system"),
        }
    }

    pub(crate) fn execute(self) {
        self.system.await_termination()
    }
}
