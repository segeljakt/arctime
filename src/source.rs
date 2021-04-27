use crate::control::*;
use kompact::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::client::*;
use crate::data::*;
use crate::pipeline::*;
use crate::port::*;
use crate::stream::*;
use crate::task::*;

type Source<T, O> = Task<T, Never, O, Never>;

fn noop<T: Iterator<Item = O> + StateReqs, O: EventReqs>(task: &mut Source<T, O>, event: Never) {}

impl<S: SystemHandle> Pipeline<S> {
    pub(crate) fn source<T, O: EventReqs>(&self, iter: T, duration: Duration) -> Stream<O>
    where
        T: IntoIterator<Item = O>,
        <T as IntoIterator>::IntoIter: StateReqs,
    {
        let iter = iter.into_iter();
        let task = self.system.create(move || {
            Task::new_with_timer("Source", iter, noop, duration, |task| {
                if let Some(item) = task.state.next() {
                    task.emit(item);
                } else {
                    task.terminate();
                }
            })
            .set_role(Role::Producer)
        });
        let connect = lazy_connect(task.clone());
        let client = self.client.clone();
        self.starters
            .borrow_mut()
            .push(Box::new(move || client.system().start(&task)));
        Stream::new(self.client.clone(), connect, self.starters.clone())
    }
}
