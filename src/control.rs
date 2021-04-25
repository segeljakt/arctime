use kompact::prelude::*;

#[derive(Debug, Clone)]
pub(crate) enum ServerMessage {
    Die,
}

#[derive(Debug, Clone)]
pub(crate) enum TaskMessage {
    Die,
}

#[derive(Debug, Clone)]
pub(crate) enum ClientMessage {
    Run,
}
