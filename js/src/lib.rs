#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn run(args: Vec<String>) -> napi::Result<()> {
  create_v1_app::run(args).map_err(|e| napi::Error::from_reason(e.to_string()))
}
