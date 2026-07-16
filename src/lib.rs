#![allow(clippy::blocks_in_conditions)]

extern crate base64;
extern crate chrono;
extern crate clap;
extern crate gtmpl;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_json;
extern crate tokio;
extern crate tracing_batteries;

#[macro_use]
mod macros;
#[macro_use]
pub mod tasks;
#[macro_use]
pub mod commands;
pub mod completion;
pub mod console;
pub mod engine;
pub mod errors;
pub mod fs;
pub mod git;
pub mod online;
pub mod search;
pub mod telemetry;
pub mod update;

#[cfg(test)]
mod test;
