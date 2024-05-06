#![deny(clippy::all)]

use bridge::{export, export_error};
use reqwest::*;
use serde_json::*;
use std::result::Result;

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
  SUN,
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

#[export_error]
#[derive(Debug, thiserror::Error)]
pub enum CustomError {
  #[error("reqwest error !!")]
  Http(#[from] reqwest::Error),

  #[error("json error !!")]
  Json(#[from] serde_json::Error),

  #[error("Other error")]
  Other,
}

#[export]
pub async fn get_be_token(user: String, admin: bool) -> Result<String, CustomError> {
  // Create request body
  let body_data = json!({
      "identity": user,
      "name": user,
      "is_admin": admin,
  });

  let body = serde_json::to_vec(&body_data).map_err(|e| CustomError::Json(e))?;

  // Create HTTP client
  let client = Client::new();

  // Create request
  let be_url = "https://sg-be.jointell.net"; // Replace with your backend URL
  let req = client
    .post(&format!("{}/generate-token", be_url))
    .header("Authorization", "Basic dXNlcjpzaGVueHVuMQ==")
    .header("Content-Type", "application/json")
    .body(body)
    .build()
    .map_err(|e| CustomError::Http(e))?;

  let resp = client
    .execute(req)
    .await
    .map_err(|e| CustomError::Http(e))?;

  let resp_body = resp.text().await.map_err(|e| CustomError::Http(e))?;

  Ok(resp_body)
}

#[export(object)]
pub struct RoomInfo {
  pub name: String,
  pub url: String,
}

#[export(object)]
pub struct RoomInfoReply {
  pub info: RoomInfo,
  pub timestamp: i32,
}