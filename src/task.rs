#![allow(clippy::type_complexity)]
#![allow(dead_code)]

use kompact::prelude::*;

use crate::control::*;
use crate::data::*;
use crate::pipeline::*;
use crate::port::*;

use std::sync::Arc;
use std::time::Duration;

#[derive(ComponentDefinition)]
pub(crate) struct Task<S: StateReqs, I: EventReqs, O: EventReqs> {
    pub(crate) ctx: ComponentContext<Self>,
    pub(crate) name: &'static str,
    /// Port for receiving input events (from proxies)
    pub(crate) iport: ProvidedPort<OneWayPort<I>>,
    /// Actors for broadcasting output events
    pub(crate) oport: RequiredPort<OneWayPort<O>>,
    pub(crate) buffer: Vec<O>,
    pub(crate) state: S,
    pub(crate) logic: fn(&mut Self, I),
    pub(crate) timer: Option<TaskTimer<S, I, O>>,
    pub(crate) scheduled: Option<ScheduledTimer>,
}

#[derive(Clone)]
pub(crate) struct TaskTimer<S: StateReqs, I: EventReqs, O: EventReqs> {
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
            buffer: Vec::new(),
            state,
            logic,
            timer: None,
            scheduled: None,
        }
    }

    pub(crate) fn new_with_timer(
        name: &'static str,
        state: S,
        logic: fn(&mut Self, I),
        duration: Duration,
        trigger: fn(&mut Self),
    ) -> Self {
        Self {
            ctx: ComponentContext::uninitialised(),
            name,
            iport: ProvidedPort::uninitialised(),
            oport: RequiredPort::uninitialised(),
            buffer: Vec::new(),
            state,
            logic,
            timer: Some(TaskTimer { duration, trigger }),
            scheduled: None,
        }
    }

    pub(crate) fn emit(&mut self, event: O) {
        self.oport.trigger(event)
    }

    pub(crate) fn trigger(&mut self, timeout_id: ScheduledTimer) -> Handled {
        match self.scheduled {
            Some(ref timeout) if *timeout == timeout_id => {
                let timer = self.timer.as_ref().unwrap();
                let duration = timer.duration;
                (timer.trigger)(self);
                self.scheduled = Some(self.schedule_once(duration, Self::trigger));
                Handled::Ok
            }
            Some(_) => Handled::Ok,
            None => {
                warn!(self.log(), "Got unexpected timeout: {:?}", timeout_id);
                Handled::Ok
            }
        }
    }
}

pub(crate) fn lazy_connect<S: StateReqs, I: EventReqs, O: EventReqs>(
    task: &Arc<Component<Task<S, I, O>>>,
) -> Arc<dyn Fn(ProvidedRef<OneWayPort<O>>)> {
    let task = task.clone();
    Arc::new(move |iport: ProvidedRef<OneWayPort<O>>| {
        task.connect_to_provided(iport);
    })
}

impl<S: StateReqs, I: EventReqs, O: EventReqs> Provide<OneWayPort<I>> for Task<S, I, O> {
    fn handle(&mut self, event: I) -> Handled {
        (self.logic)(self, event);
        Handled::Ok
    }
}

impl<S: StateReqs, I: EventReqs, O: EventReqs> Require<OneWayPort<O>> for Task<S, I, O> {
    fn handle(&mut self, _: Never) -> Handled {
        Handled::Ok
    }
}

impl<S: StateReqs, I: EventReqs, O: EventReqs> ComponentLifecycle for Task<S, I, O> {
    fn on_start(&mut self) -> Handled {
        if let Some(timer) = &mut self.timer {
            let duration = timer.duration;
            self.scheduled = Some(self.schedule_once(duration, Self::trigger));
        }
        Handled::Ok
    }

    fn on_stop(&mut self) -> Handled {
        if let Some(scheduled) = self.scheduled.take() {
            self.cancel_timer(scheduled);
        }
        Handled::Ok
    }

    fn on_kill(&mut self) -> Handled {
        Handled::Ok
    }
}
