use std::sync::Arc;
use clap::{App, ArgMatches};
use std::vec::Vec;
use super::core;
use super::tasks;
use super::errors;
use super::online;
use async_trait::async_trait;

mod ignore;
mod info;
mod open;
mod scratch;

#[async_trait]
pub trait Command {
    fn name(&self) -> String;
    fn app<'a, 'b>(&self) -> App<'a, 'b>;
    async fn run<'a>(&self, core: Arc<core::Core>, matches: &ArgMatches<'a>) -> Result<i32, errors::Error>;
}

pub fn commands() -> Vec<Arc<dyn Command>> {
    vec![
        Arc::new(info::InfoCommand{}),
        Arc::new(ignore::IgnoreCommand{}),
        Arc::new(open::OpenCommand{}),
        Arc::new(scratch::ScratchCommand{}),
    ]
}