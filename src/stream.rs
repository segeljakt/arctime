use kompact::prelude::*;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::Arc;

use kompact::component::AbstractComponent;

use crate::client::*;
use crate::control::*;
use crate::data::*;
use crate::pipeline::*;
use crate::port::*;
use crate::task::*;

#[derive(Clone)]
pub(crate) struct Stream<T: EventReqs> {
    client: Arc<Component<Client>>,
    connect: Arc<dyn Fn(Arc<dyn AbstractComponent<Message = TaskMessage>>)>,
    tasks: Arc<RefCell<Vec<Arc<dyn AbstractComponent<Message = TaskMessage>>>>>,
    marker: PhantomData<T>,
}

impl<I: EventReqs> Stream<I> {
    pub(crate) fn new(
        client: Arc<Component<Client>>,
        connect: Arc<dyn Fn(Arc<dyn AbstractComponent<Message = TaskMessage>>)>,
        tasks: Arc<RefCell<Vec<Arc<dyn AbstractComponent<Message = TaskMessage>>>>>,
    ) -> Self {
        Self {
            client,
            connect,
            tasks,
            marker: PhantomData,
        }
    }

    /// Apply a transformation to a stream to produce a new stream.
    pub(crate) fn apply<S: StateReqs, O: EventReqs>(self, task: Task<S, I, O>) -> Stream<O> {
        let task = self.client.system().create(|| task);
        (self.connect)(task.clone());
        let connect = lazy_connect(&task);
        self.tasks.borrow_mut().push(task);
        Stream::new(self.client, connect, self.tasks)
    }

    /// Write a stream to a sink.
    /// Also returns a new pipeline (which can for example be finalized)
    pub(crate) fn sink<S: StateReqs>(self, task: Task<S, I, Never>) -> Pipeline<impl SystemHandle> {
        let stream = self.apply(task);
        Pipeline {
            system: stream.client.on_definition(|c| c.ctx().system()),
            client: stream.client,
            tasks: stream.tasks,
        }
    }

    /// Merge two streams into one.
    pub(crate) fn merge<S: StateReqs, X: EventReqs, O: EventReqs>(
        self,
        other: Stream<X>,
        task: Task<S, Union<I, X>, O>,
    ) -> Stream<O> {
        let mergel = self
            .client
            .system()
            .create(|| Task::new("Merge Left", (), |task, event| task.emit(Union::L(event))));
        let merger = self
            .client
            .system()
            .create(|| Task::new("Merge Right", (), |task, event| task.emit(Union::R(event))));
        let task = self.client.system().create(|| task);
        biconnect_components(&task, &mergel);
        biconnect_components(&task, &merger);
        (self.connect)(mergel.clone());
        (other.connect)(merger.clone());
        let connect = lazy_connect(&task);
        self.tasks.borrow_mut().push(mergel);
        self.tasks.borrow_mut().push(merger);
        self.tasks.borrow_mut().push(task);
        Stream::new(self.client, connect, self.tasks)
    }

    /// Split one stream into two.
    pub(crate) fn split<S: StateReqs, X: EventReqs, O: EventReqs>(
        self,
        task: Task<S, I, Union<O, X>>,
    ) -> (Stream<O>, Stream<X>) {
        let stream = self.apply(task);
        let streaml = stream
            .clone()
            .apply(Task::new("Split Left", (), |task, event| {
                if let Union::L(event) = event {
                    task.emit(event)
                }
            }));
        let streamr = stream.apply(Task::new("Split Right", (), |task, event| {
            if let Union::R(event) = event {
                task.emit(event)
            }
        }));
        (streaml, streamr)
    }
}
