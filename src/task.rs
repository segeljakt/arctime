#![allow(clippy::type_complexity)]
#![allow(dead_code)]

use kompact::component::AbstractComponent;
use kompact::prelude::*;
use std::collections::VecDeque;

use crate::control::*;
use crate::data::*;
use crate::pipeline::*;
use crate::port::*;
use crate::stream::*;

use std::sync::Arc;
use std::time::Duration;

pub enum Role {
    Producer,
    ProducerConsumer,
    Consumer,
}

#[derive(ComponentDefinition)]
pub(crate) struct Task<S: StateReqs, I: EventReqs, O: EventReqs, R: EventReqs = Never> {
    pub(crate) ctx: ComponentContext<Self>,
    pub(crate) name: &'static str,
    pub(crate) iport: ProvidedPort<OneWayPort<I>>,
    pub(crate) oport: RequiredPort<OneWayPort<O>>,
    pub(crate) state: S,
    pub(crate) logic: fn(&mut Self, I),
    pub(crate) oneshot: Option<OneShot<S, I, O, R>>,
    pub(crate) scheduled_oneshot: Option<ScheduledTimer>,
    pub(crate) role: Role,
}

#[derive(Clone)]
pub(crate) struct OneShot<S: StateReqs, I: EventReqs, O: EventReqs, R: EventReqs> {
    duration: Duration,
    trigger: fn(&mut Task<S, I, O, R>),
}

impl<R: EventReqs, S: StateReqs, I: EventReqs, O: EventReqs> Actor for Task<S, I, O, R> {
    type Message = Ask<(), R>;

    fn receive_local(&mut self, msg: Self::Message) -> Handled {
        todo!()
    }

    fn receive_network(&mut self, msg: NetMessage) -> Handled {
        todo!()
    }
}

impl<S: StateReqs, I: EventReqs, O: EventReqs, R: EventReqs> Task<S, I, O, R> {
    pub(crate) fn new(name: &'static str, state: S, logic: fn(&mut Self, I)) -> Self {
        Self {
            ctx: ComponentContext::uninitialised(),
            name,
            iport: ProvidedPort::uninitialised(),
            oport: RequiredPort::uninitialised(),
            state,
            logic,
            oneshot: None,
            scheduled_oneshot: None,
            role: Role::ProducerConsumer,
        }
    }

    pub(crate) fn set_role(mut self, role: Role) -> Self {
        Self { role, ..self }
    }

    pub(crate) fn new_with_timer(
        name: &'static str,
        state: S,
        logic: fn(&mut Self, I),
        duration: Duration,
        trigger: fn(&mut Self),
    ) -> Self {
        Self {
            oneshot: Some(OneShot { duration, trigger }),
            ..Self::new(name, state, logic)
        }
    }

    pub(crate) fn emit(&mut self, event: O) {
        self.oport.trigger(Some(event));
    }

    pub(crate) fn terminate(&mut self) {
        self.oport.trigger(None);
        self.ctx.suicide();
    }

    pub(crate) fn oneshot_trigger(&mut self, timeout: ScheduledTimer) -> Handled {
        match self.scheduled_oneshot {
            Some(ref scheduled_oneshot) if *scheduled_oneshot == timeout => {
                let oneshot = self.oneshot.as_ref().unwrap();
                let duration = oneshot.duration;
                (oneshot.trigger)(self);
                self.scheduled_oneshot = Some(self.schedule_once(duration, Self::oneshot_trigger));
                Handled::Ok
            }
            Some(_) => Handled::Ok,
            None => {
                warn!(self.log(), "Got unexpected timeout: {:?}", timeout);
                Handled::Ok
            }
        }
    }
}

pub(crate) fn lazy_connect<S: StateReqs, I: EventReqs, O: EventReqs, R: EventReqs>(
    producer: Arc<Component<Task<S, I, O, R>>>,
) -> Arc<dyn Fn(&mut ProvidedPort<OneWayPort<O>>) + 'static> {
    Arc::new(move |mut iport| {
        producer.on_definition(|producer| {
            iport.connect(producer.oport.share());
            producer.oport.connect(iport.share());
        })
    })
}

impl<S: StateReqs, I: EventReqs, O: EventReqs, R: EventReqs> Provide<OneWayPort<I>>
    for Task<S, I, O, R>
{
    fn handle(&mut self, event: Option<I>) -> Handled {
        if let Some(event) = event {
            (self.logic)(self, event);
            Handled::Ok
        } else {
            self.oport.trigger(None);
            Handled::DieNow
        }
    }
}

impl<S: StateReqs, I: EventReqs, O: EventReqs, R: EventReqs> Require<OneWayPort<O>>
    for Task<S, I, O, R>
{
    fn handle(&mut self, event: FlowControl) -> Handled {
        Handled::Ok
    }
}

impl<S: StateReqs, I: EventReqs, O: EventReqs, R: EventReqs> ComponentLifecycle
    for Task<S, I, O, R>
{
    fn on_start(&mut self) -> Handled {
        if let Some(oneshot) = self.oneshot.as_mut() {
            let duration = oneshot.duration;
            self.scheduled_oneshot = Some(self.schedule_once(duration, Self::oneshot_trigger));
        }
        Handled::Ok
    }

    fn on_stop(&mut self) -> Handled {
        if let Some(scheduled_oneshot) = self.scheduled_oneshot.take() {
            self.cancel_timer(scheduled_oneshot);
        }
        Handled::Ok
    }

    fn on_kill(&mut self) -> Handled {
        Handled::Ok
    }
}
