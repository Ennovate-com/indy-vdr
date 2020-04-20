use std::collections::HashMap;
use std::iter::FromIterator;
use std::time::{Duration, SystemTime};

use super::networker;
use super::types::{self, Message, PoolSetup};

mod base;
pub use base::{PoolRequest, PoolRequestImpl, RequestHandle};

use crate::common::error::prelude::*;

#[derive(Debug)]
pub enum RequestEvent {
    Received(
        String,  // node alias
        String,  // message
        Message, // parsed
    ),
    Timeout(
        String, // node_alias
    ),
}

#[derive(Debug)]
pub enum RequestExtEvent {
    Init,
    Sent(
        String,     // node alias
        SystemTime, // send time
    ),
    Received(
        String,     // node alias
        String,     // message
        Message,    // parsed
        SystemTime, // received time
    ),
    Timeout(
        String, // node_alias
    ),
}

#[derive(Debug)]
pub enum RequestResult<T> {
    Reply(T),
    Failed(VdrError),
}

impl<T> RequestResult<T> {
    pub fn map_result<F, R>(self, f: F) -> VdrResult<RequestResult<R>>
    where
        F: FnOnce(T) -> VdrResult<R>,
    {
        match self {
            Self::Reply(reply) => Ok(RequestResult::Reply(f(reply)?)),
            Self::Failed(err) => Ok(RequestResult::Failed(err)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RequestState {
    NotStarted,
    Active,
    Terminated,
}

impl std::fmt::Display for RequestState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = match self {
            Self::NotStarted => "NotStarted",
            Self::Active => "Active",
            Self::Terminated => "Terminated",
        };
        f.write_str(state)
    }
}

pub type TimingResult = HashMap<String, f32>;

#[derive(Debug)]
pub enum RequestTarget {
    Default,
    Full(
        Option<Vec<String>>, // nodes to send
        Option<i64>,         // timeout
    ),
}

#[derive(Debug)]
pub struct RequestTiming {
    replies: HashMap<String, (SystemTime, f32)>,
}

impl RequestTiming {
    pub fn new() -> Self {
        Self {
            replies: HashMap::new(),
        }
    }

    pub fn sent(&mut self, node_alias: &str, send_time: SystemTime) {
        self.replies
            .insert(node_alias.to_owned(), (send_time, -1.0));
    }

    pub fn received(&mut self, node_alias: &str, recv_time: SystemTime) {
        self.replies.get_mut(node_alias).map(|node| {
            let duration = recv_time
                .duration_since(node.0)
                .unwrap_or(Duration::new(0, 0))
                .as_secs_f32();
            node.1 = duration;
        });
    }

    pub fn result(&self) -> Option<TimingResult> {
        Some(HashMap::from_iter(
            self.replies.iter().map(|(k, (_, v))| (k.clone(), *v)),
        ))
    }
}
