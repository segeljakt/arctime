#![allow(clippy::type_complexity)]
#![allow(dead_code)]

use kompact::prelude::*;

use crate::data::*;
use crate::port::*;

#[derive(ComponentDefinition, Actor)]
pub(crate) struct Task<S: StateReqs, I0: EventReqs, I1: EventReqs, O0: EventReqs, O1: EventReqs> {
    pub(crate) ctx: ComponentContext<Self>,
    /// Port for receiving input events (from proxies)
    pub(crate) iport: ProvidedPort<OneWayPort<Either<I0, I1>>>,
    pub(crate) proxy0: ActorRef<I0>,
    pub(crate) proxy1: ActorRef<I1>,
    /// Actors for broadcasting output events
    pub(crate) orefs0: Vec<ActorRef<O0>>,
    pub(crate) orefs1: Vec<ActorRef<O1>>,
    pub(crate) state: S,
    pub(crate) logic: fn(&mut Self, Either<I0, I1>),
}

pub(crate) trait CreateTask<
    S: StateReqs,
    I0: EventReqs,
    I1: EventReqs,
    O0: EventReqs,
    O1: EventReqs,
>
{
    fn create_task(
        &self,
        state: S,
        logic: fn(&mut Task<S, I0, I1, O0, O1>, Either<I0, I1>),
    ) -> std::sync::Arc<Component<Task<S, I0, I1, O0, O1>>>;
}

impl<S: StateReqs, I0: EventReqs, I1: EventReqs, O0: EventReqs, O1: EventReqs>
    CreateTask<S, I0, I1, O0, O1> for KompactSystem
{
    fn create_task(
        &self,
        state: S,
        logic: fn(&mut Task<S, I0, I1, O0, O1>, Either<I0, I1>),
    ) -> std::sync::Arc<Component<Task<S, I0, I1, O0, O1>>> {
        let proxy0 = self.create(Proxy0::<I0, I1>::new);
        let proxy1 = self.create(Proxy1::<I0, I1>::new);
        let proxy0ref = proxy0.actor_ref();
        let proxy1ref = proxy1.actor_ref();
        let task = self.create(move || Task::new(proxy0ref, proxy1ref, state, logic));
        biconnect_components(&task, &proxy0).expect("biconnect");
        biconnect_components(&task, &proxy1).expect("biconnect");
        self.start(&proxy0);
        self.start(&proxy1);
        task
    }
}

impl<S: StateReqs, I0: EventReqs, I1: EventReqs, O0: EventReqs, O1: EventReqs>
    Task<S, I0, I1, O0, O1>
{
    pub(crate) fn new(
        proxy0: ActorRef<I0>,
        proxy1: ActorRef<I1>,
        state: S,
        logic: fn(&mut Self, Either<I0, I1>),
    ) -> Self {
        Self {
            ctx: ComponentContext::uninitialised(),
            iport: ProvidedPort::uninitialised(),
            proxy0,
            proxy1,
            orefs0: Vec::new(),
            orefs1: Vec::new(),
            state,
            logic,
        }
    }
    pub(crate) fn emit(&mut self, event: Either<O0, O1>) {
        match event {
            Either::A(event) => self.orefs0.iter().for_each(|r| r.tell(event.clone())),
            Either::B(event) => self.orefs1.iter().for_each(|r| r.tell(event.clone())),
        }
    }
}

impl<S: StateReqs, I0: EventReqs, I1: EventReqs, O0: EventReqs, O1: EventReqs>
    Provide<OneWayPort<Either<I0, I1>>> for Task<S, I0, I1, O0, O1>
{
    fn handle(&mut self, event: Either<I0, I1>) -> Handled {
        (self.logic)(self, event);
        Handled::Ok
    }
}

#[derive(ComponentDefinition)]
pub(crate) struct Proxy0<I0: EventReqs, I1: EventReqs> {
    pub(crate) ctx: ComponentContext<Self>,
    pub(crate) oport: RequiredPort<OneWayPort<Either<I0, I1>>>,
}

#[derive(ComponentDefinition)]
pub(crate) struct Proxy1<I0: EventReqs, I1: EventReqs> {
    pub(crate) ctx: ComponentContext<Self>,
    pub(crate) oport: RequiredPort<OneWayPort<Either<I0, I1>>>,
}

impl<I0: EventReqs, I1: EventReqs> Proxy0<I0, I1> {
    pub(crate) fn new() -> Self {
        Self {
            ctx: ComponentContext::uninitialised(),
            oport: RequiredPort::uninitialised(),
        }
    }
}

impl<I0: EventReqs, I1: EventReqs> Proxy1<I0, I1> {
    pub(crate) fn new() -> Self {
        Self {
            ctx: ComponentContext::uninitialised(),
            oport: RequiredPort::uninitialised(),
        }
    }
}

impl<I0: EventReqs, I1: EventReqs> Actor for Proxy0<I0, I1> {
    type Message = I0;

    fn receive_local(&mut self, msg: Self::Message) -> Handled {
        self.oport.trigger(Either::A(msg));
        Handled::Ok
    }

    fn receive_network(&mut self, _: NetMessage) -> Handled {
        todo!()
    }
}

impl<I0: EventReqs, I1: EventReqs> Actor for Proxy1<I0, I1> {
    type Message = I1;

    fn receive_local(&mut self, msg: Self::Message) -> Handled {
        self.oport.trigger(Either::B(msg));
        Handled::Ok
    }

    fn receive_network(&mut self, _: NetMessage) -> Handled {
        todo!()
    }
}

impl<I0: EventReqs, I1: EventReqs> Require<OneWayPort<Either<I0, I1>>> for Proxy0<I0, I1> {
    fn handle(&mut self, _: Never) -> Handled {
        Handled::Ok
    }
}

impl<I0: EventReqs, I1: EventReqs> Require<OneWayPort<Either<I0, I1>>> for Proxy1<I0, I1> {
    fn handle(&mut self, _: Never) -> Handled {
        Handled::Ok
    }
}

impl<S: StateReqs, I0: EventReqs, I1: EventReqs, O0: EventReqs, O1: EventReqs> ComponentLifecycle
    for Task<S, I0, I1, O0, O1>
{
}

impl<I0: EventReqs, I1: EventReqs> ComponentLifecycle for Proxy0<I0, I1> {}
impl<I0: EventReqs, I1: EventReqs> ComponentLifecycle for Proxy1<I0, I1> {}
