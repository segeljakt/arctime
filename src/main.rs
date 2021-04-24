mod data;
mod pipeline;
mod port;
mod task;
// mod task_old;
// mod task_old2;

use data::*;
use task::*;

use kompact::prelude::*;

type Map<I, O> = Task<(), I, Never, O, Never>;
type Filter<T> = Task<(), T, Never, T, Never>;
type Reduce<I, O> = Task<O, I, Never, O, Never>;
type Print<T> = Task<(), T, Never, Never, Never>;

fn main() {
    let system = KompactConfig::default().build().expect("system");

    let map = system.create_task((), |task: &mut Map<f32, i32>, event| match event {
        Either::A(event) => task.emit(Either::A(event as i32 - 1)),
        Either::B(_) => unreachable!(),
    });

    let filter = system.create_task((), |task: &mut Filter<i32>, event| match event {
        Either::A(event) => {
            if event > 0 {
                task.emit(Either::A(event));
            }
        }
        Either::B(_) => unreachable!(),
    });

    let reduce = system.create_task(0, |task: &mut Reduce<i32, i32>, event| match event {
        Either::A(event) => {
            task.state += event;
            task.emit(Either::A(event));
        }
        Either::B(_) => unreachable!(),
    });

    let print = system.create_task((), |task: &mut Print<i32>, event| match event {
        Either::A(event) => {
            info!(task.ctx.log(), "{}", event)
        }
        Either::B(_) => unreachable!(),
    });

    map.on_definition(|map| {
        filter.on_definition(|filter| {
            map.orefs0.push(filter.proxy0.actor_ref());
        })
    });
    filter.on_definition(|filter| {
        reduce.on_definition(|reduce| {
            filter.orefs0.push(reduce.proxy0.actor_ref());
        })
    });

    reduce.on_definition(|reduce| {
        print.on_definition(|print| {
            reduce.orefs0.push(print.proxy0.actor_ref());
        })
    });

    system.start(&map);
    system.start(&filter);
    system.start(&reduce);
    system.start(&print);

    let source = map.on_definition(|map| map.proxy0.actor_ref());

    for i in 0..100 {
        source.tell(i as f32);
    }

    system.await_termination();
}
