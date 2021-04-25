#![allow(unused)]

mod client;
mod control;
mod data;
mod pipeline;
mod port;
mod source;
mod stream;
mod task;

use client::*;
use data::*;
use pipeline::*;
use task::*;

use kompact::prelude::*;
use std::time::Duration;

type Map<I, O> = Task<(), I, O>;
type Filter<T> = Task<(), T, T>;
type Reduce<I, O> = Task<O, I, O>;
type Print<T> = Task<(), T, Never>;

fn main() {
    let pipeline = Pipeline::new();

    pipeline
        .source(0..100, Duration::new(0, 50_000_000))
        // task Map(): i32 -> i32 {
        //     on event => emit event - 1
        // }
        .apply(|| {
            Task::new("Map", (), |task: &mut Map<i32, i32>, event: i32| {
                task.emit(event + 1)
            })
        })
        // task Filter(): i32 -> i32 {
        //     on event => if event > 0 {
        //         emit event
        //     }
        // }
        .apply(|| {
            Task::new("Filter", (), |task: &mut Filter<i32>, event: i32| {
                if event % 2 == 0 {
                    task.emit(event);
                }
            })
        })
        // task Reduce(state: i32): i32 -> i32 {
        //     var state = state;
        //     on event => {
        //         state += event;
        //         emit event
        //     }
        // }
        .apply(|| {
            Task::new("Reduce", 0, |task: &mut Reduce<i32, i32>, event: i32| {
                task.state += event;
                task.emit(event);
            })
        })
        // task Print(): i32 -> () {
        //     on event => print(event)
        // }
        .apply(|| {
            Task::new("Print", (), |task: &mut Print<i32>, event: i32| {
                info!(task.ctx.log(), "{}", event)
            })
        });

    pipeline.execute();
}
