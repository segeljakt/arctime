#![allow(clippy::type_complexity)]
#![allow(dead_code)]

use kompact::component::AbstractComponent;
use kompact::prelude::*;
use std::collections::VecDeque;

use crate::control::*;
use crate::data::*;
use crate::pipeline::*;
use crate::port::*;

use std::sync::Arc;
use std::time::Duration;

pub const FRAME_SIZE: usize = 8;
pub const N_FRAMES: usize = 4;

pub enum Role {
    Producer,
    ProducerConsumer,
    Consumer,
}

#[derive(ComponentDefinition)]
pub(crate) struct Task<S: StateReqs, I: EventReqs, O: EventReqs> {
    pub(crate) ctx: ComponentContext<Self>,
    pub(crate) name: &'static str,
    /// Port for receiving input events (from proxies)
    pub(crate) iport: ProvidedPort<OneWayPort<I>>,
    /// Actors for broadcasting output events
    pub(crate) oport: RequiredPort<OneWayPort<O>>,
    pub(crate) state: S,
    pub(crate) logic: fn(&mut Self, I),
    pub(crate) oneshot: Option<OneShot<S, I, O>>,
    pub(crate) scheduled_oneshot: Option<ScheduledTimer>,
    pub(crate) scheduled_periodic: Option<ScheduledTimer>,
    pub(crate) role: Role,
}

use crate::stream::Stream;

#[derive(Clone)]
pub(crate) struct OneShot<S: StateReqs, I: EventReqs, O: EventReqs> {
    duration: Duration,
    trigger: fn(&mut Task<S, I, O>),
}

impl<S: StateReqs, I: EventReqs, O: EventReqs> Actor for Task<S, I, O> {
    type Message = TaskMessage;

    fn receive_local(&mut self, msg: Self::Message) -> Handled {
        todo!()
    }

    fn receive_network(&mut self, msg: NetMessage) -> Handled {
        todo!()
    }
}

impl<S: StateReqs, I: EventReqs, O: EventReqs> Task<S, I, O> {
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
            scheduled_periodic: None,
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

pub(crate) fn lazy_connect<S: StateReqs, I: EventReqs, O: EventReqs>(
    producer: &Arc<Component<Task<S, I, O>>>,
) -> Arc<dyn Fn(Arc<dyn AbstractComponent<Message = TaskMessage>>)> {
    let producer = producer.clone() as Arc<dyn AbstractComponent<Message = TaskMessage>>;
    Arc::new(move |consumer| {
        producer.on_dyn_definition(|producer| {
            consumer.on_dyn_definition(|consumer| {
                let oport = producer.get_required_port::<OneWayPort<O>>().unwrap();
                let iport = consumer.get_provided_port::<OneWayPort<O>>().unwrap();
                biconnect_ports(iport, oport);
            })
        })
    })
}

impl<S: StateReqs, I: EventReqs, O: EventReqs> Provide<OneWayPort<I>> for Task<S, I, O> {
    fn handle(&mut self, event: Option<I>) -> Handled {
        if let Some(event) = event {
            (self.logic)(self, event);
            Handled::Ok
        } else {
            self.oport.trigger(None);
            Handled::DieNow
        }
        // coz::begin!("event-begin");
        //         Handled::block_on(self, move |mut async_self| async move {
        //         })
        // coz::end!("event-end");
    }
}

impl<S: StateReqs, I: EventReqs, O: EventReqs> Require<OneWayPort<O>> for Task<S, I, O> {
    fn handle(&mut self, event: FlowControl) -> Handled {
        Handled::Ok
    }
}

impl<S: StateReqs, I: EventReqs, O: EventReqs> ComponentLifecycle for Task<S, I, O> {
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
        if let Some(scheduled_periodic) = self.scheduled_periodic.take() {
            self.cancel_timer(scheduled_periodic);
        }
        Handled::Ok
    }

    fn on_kill(&mut self) -> Handled {
        Handled::Ok
    }
}
