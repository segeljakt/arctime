use kompact::prelude::*;
use std::cell::RefCell;
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
    connect: Arc<dyn Fn(ProvidedRef<OneWayPort<T>>)>,
    tasks: Arc<RefCell<Vec<Arc<dyn AbstractComponent<Message = TaskMessage>>>>>,
}

impl<I: EventReqs> Stream<I> {
    pub(crate) fn new(
        client: Arc<Component<Client>>,
        connect: Arc<dyn Fn(ProvidedRef<OneWayPort<I>>)>,
        tasks: Arc<RefCell<Vec<Arc<dyn AbstractComponent<Message = TaskMessage>>>>>,
    ) -> Self {
        Self {
            client,
            connect,
            tasks,
        }
    }

    pub(crate) fn apply<S: StateReqs, O: EventReqs>(
        self,
        cons: impl Fn() -> Task<S, I, O>,
    ) -> Stream<O> {
        let task = self.client.system().create(cons);
        (self.connect)(task.provided_ref());
        let connect = lazy_connect(&task);
        self.tasks.borrow_mut().push(task);
        Stream::new(self.client, connect, self.tasks)
    }

    pub(crate) fn merge<S: StateReqs, X: EventReqs, O: EventReqs>(
        self,
        other: Stream<X>,
        cons: impl Fn() -> Task<S, Union<I, X>, O>,
    ) -> Stream<O> {
        let mergel = self.client.system().create(|| {
            Task::new("mergel", (), |task: &mut Task<_, _, _>, event| {
                task.emit(Union::L(event))
            })
        });
        let merger = self.client.system().create(|| {
            Task::new("merger", (), |task: &mut Task<_, _, _>, event| {
                task.emit(Union::R(event))
            })
        });
        let task = self.client.system().create(cons);
        (self.connect)(mergel.provided_ref());
        (other.connect)(merger.provided_ref());
        biconnect_components(&task, &mergel);
        biconnect_components(&task, &merger);
        let connect = lazy_connect(&task);
        self.tasks.borrow_mut().push(mergel);
        self.tasks.borrow_mut().push(merger);
        self.tasks.borrow_mut().push(task);
        Stream::new(self.client, connect, self.tasks)
    }

    pub(crate) fn split<S: StateReqs, X: EventReqs, O: EventReqs>(
        self,
        cons: impl Fn() -> Task<S, I, Union<O, X>>,
    ) -> (Stream<O>, Stream<X>) {
        let splitl = self.client.system().create(|| {
            Task::new("splitl", (), |task: &mut Task<_, _, _>, event| {
                if let Union::L(event) = event {
                    task.emit(event)
                }
            })
        });
        let splitr = self.client.system().create(|| {
            Task::new("splitr", (), |task: &mut Task<_, _, _>, event| {
                if let Union::R(event) = event {
                    task.emit(event)
                }
            })
        });
        let task = self.client.system().create(cons);
        (self.connect)(task.provided_ref());
        biconnect_components(&splitl, &task);
        biconnect_components(&splitr, &task);
        let connectl = lazy_connect(&splitl);
        let connectr = lazy_connect(&splitr);
        let streaml = Stream::new(self.client.clone(), connectl, self.tasks.clone());
        let streamr = Stream::new(self.client.clone(), connectr, self.tasks.clone());
        self.tasks.borrow_mut().push(splitl);
        self.tasks.borrow_mut().push(splitr);
        self.tasks.borrow_mut().push(task);
        (streaml, streamr)
    }
}
