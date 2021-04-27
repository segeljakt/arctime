#![allow(clippy::type_complexity)]

use kompact::prelude::*;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

use kompact::component::AbstractComponent;

use crate::client::*;
use crate::control::*;
use crate::data::*;
use crate::pipeline::*;
use crate::port::*;
use crate::task::*;

type BlackBox = Box<dyn FnOnce()>;
type Connect<T> = dyn Fn(&mut ProvidedPort<OneWayPort<T>>) + 'static;

#[derive(Clone)]
pub(crate) struct Stream<T: EventReqs> {
    pub(crate) client: Arc<Component<Client>>,
    pub(crate) connect: Arc<dyn Fn(&mut ProvidedPort<OneWayPort<T>>) + 'static>,
    pub(crate) starters: Rc<RefCell<Vec<BlackBox>>>,
    marker: PhantomData<T>,
}

impl<I: EventReqs> Stream<I> {
    /// Apply a transformation to a stream to produce a new stream.
    pub(crate) fn apply_terminating<S: StateReqs, O: EventReqs, R: EventReqs>(
        self,
        task: Task<S, I, O, R>,
    ) -> Stream<O> {
        let task = self.client.system().create(|| task);
        task.on_definition(|c| (self.connect)(&mut c.iport));
        let connect = lazy_connect(task.clone());
        let client = self.client.clone();
        self.starters
            .borrow_mut()
            .push(Box::new(move || client.system().start(&task)));
        Stream::new(self.client, connect, self.starters)
    }
}

impl<I: EventReqs> Stream<I> {
    pub(crate) fn new(
        client: Arc<Component<Client>>,
        connect: Arc<Connect<I>>,
        starters: Rc<RefCell<Vec<BlackBox>>>,
    ) -> Self {
        Self {
            client,
            connect,
            starters,
            marker: PhantomData,
        }
    }

    /// Apply a transformation to a stream to produce a new stream.
    pub(crate) fn apply<S: StateReqs, O: EventReqs>(self, task: Task<S, I, O, Never>) -> Stream<O> {
        let task = self.client.system().create(|| task);
        task.on_definition(|c| (self.connect)(&mut c.iport));
        let connect = lazy_connect(task.clone());
        let client = self.client.clone();
        self.starters
            .borrow_mut()
            .push(Box::new(move || client.system().start(&task)));
        Stream::new(self.client, connect, self.starters)
    }

    /// Merge two streams into one.
    pub(crate) fn merge<S: StateReqs, X: EventReqs, O: EventReqs>(
        self,
        other: Stream<X>,
        task: Task<S, Union<I, X>, O, Never>,
    ) -> Stream<O> {
        let mergel: Arc<Component<Task<_, _, _, _>>> = self.client.system().create(|| {
            Task::new(
                "Merge Left",
                (),
                |task: &mut Task<(), I, Union<I, X>, Never>, event| task.emit(Union::L(event)),
            )
        });
        let merger: Arc<Component<Task<_, _, _, _>>> = self.client.system().create(|| {
            Task::new(
                "Merge Right",
                (),
                |task: &mut Task<(), X, Union<I, X>, Never>, event| task.emit(Union::R(event)),
            )
        });
        let task = self.client.system().create(|| task);
        biconnect_components(&task, &mergel);
        biconnect_components(&task, &merger);
        mergel.on_definition(|c| (self.connect)(&mut c.iport));
        merger.on_definition(|c| (other.connect)(&mut c.iport));
        let connect = lazy_connect(task.clone());
        let client = self.client.clone();
        self.starters
            .borrow_mut()
            .push(Box::new(move || client.system().start(&mergel)));
        let client = self.client.clone();
        self.starters
            .borrow_mut()
            .push(Box::new(move || client.system().start(&merger)));
        let client = self.client.clone();
        self.starters
            .borrow_mut()
            .push(Box::new(move || client.system().start(&task)));
        Stream::new(self.client, connect, self.starters)
    }

    /// Split one stream into two.
    pub(crate) fn split<S: StateReqs, X: EventReqs, O: EventReqs>(
        self,
        task: Task<S, I, Union<O, X>, Never>,
    ) -> (Stream<O>, Stream<X>) {
        let stream = self.apply(task);
        let streaml = stream.clone().apply(Task::new(
            "Split Left",
            (),
            |task: &mut Task<(), Union<O, X>, O, Never>, event| {
                if let Union::L(event) = event {
                    task.emit(event)
                }
            },
        ));
        let streamr = stream.apply(Task::new(
            "Split Right",
            (),
            |task: &mut Task<(), Union<O, X>, X, Never>, event| {
                if let Union::R(event) = event {
                    task.emit(event)
                }
            },
        ));
        (streaml, streamr)
    }
}
