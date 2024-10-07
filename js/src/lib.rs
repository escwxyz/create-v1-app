#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn run() -> napi::Result<()> {
  create_v1_app::run().map_err(|e| napi::Error::from_reason(e.to_string()))
}
