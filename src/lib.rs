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

#[export]
pub enum Week {
  MON,
  TUE,
  WED,
  THU,
  FRI,
  SAT,
  SUN
}

#[export]
pub fn get_week_value(w: Week) -> i32 {
  w as i32
}

#[export]
pub struct Service {}

#[export]
impl Service {
  fn hello(&self) -> String {
    "SERVICE:HELLO".to_owned()
  }
}

#[export]
pub fn Service_new() -> Service {
  Service {}
}

#[export]
pub fn Service_hello(s: &Service) -> String {
  s.hello()
} 