#![allow(unused)]

mod client;
mod control;
mod data;
mod executor;
mod pipeline;
mod port;
mod sink;
mod source;
mod stream;
mod task;

use client::*;
use data::*;
use executor::*;
use pipeline::*;
use task::*;

use kompact::prelude::*;
use std::time::Duration;

fn main() {
    let executor = Executor::new();

    executor
        .pipeline()
        .source(0..200, Duration::new(0, 50_000_000))
        .apply(Task::new("Map", (), |task, event| task.emit(event + 1)))
        .apply(Task::new("Filter", (), |task, event| {
            if event % 2 == 0 {
                task.emit(event);
            }
        }))
        .apply(Task::new("Reduce", 0, |task, event| {
            task.state += event;
            task.emit(event);
        }))
        .apply(Task::new("Nested", (), |task, event: i32| {
            task.pipeline()
                .source(event..100, Duration::new(0, 100_000_000))
                .sink(Task::new("Inner print", (), |task, event| {
                    info!(task.ctx.log(), "Inner: {}", event);
                }))
                .finalize();
            task.emit(event)
        }))
        .sink(Task::new("Print", (), |task, event| {
            info!(task.ctx.log(), "Outer: {}", event);
        }))
        .finalize();

    executor.execute();
}
