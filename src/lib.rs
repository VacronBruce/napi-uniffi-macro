#![deny(clippy::all)]

use bridge::export;
#[cfg(feature = "ffi")]
uniffi::setup_scaffolding!();

#[cfg(feature = "node")]
#[macro_use]
extern crate napi_derive;

#[export]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}
