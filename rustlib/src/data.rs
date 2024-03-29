use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
  pub mainsite: String,
  pub appname: String,
  pub emaildomain: String,
  pub db: PathBuf,
  pub admin_email: String,
  pub regen_login_tokens: bool,
  pub login_token_expiration_ms: Option<i64>,
  pub email_token_expiration_ms: i64,
  pub reset_token_expiration_ms: i64,
  pub invite_token_expiration_ms: i64,
  pub open_registration: bool,
  pub send_emails: bool,
  pub non_admin_invite: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginData {
  pub userid: i64,
  pub name: String,
  pub email: String,
  pub admin: bool,
  pub active: bool,
  pub data: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdminSettings {
  pub open_registration: bool,
  pub send_emails: bool,
  pub non_admin_invite: bool,
}

pub fn admin_settings(config: &Config) -> AdminSettings {
  AdminSettings {
    open_registration: config.open_registration,
    send_emails: config.send_emails,
    non_admin_invite: config.non_admin_invite,
  }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct User {
  pub id: i64,
  pub name: String,
  pub hashwd: String,
  pub salt: String,
  pub email: String,
  pub registration_key: Option<String>,
  pub admin: bool,
  pub active: bool,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct UserInvite {
  pub email: Option<String>,
  pub token: String,
  pub url: String,
  pub data: Option<String>,
  pub creator: i64,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct GetInvite {
  pub email: Option<String>,
  pub data: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct RegistrationData {
  pub uid: String,
  pub pwd: String,
  pub email: String,
}

#[derive(Deserialize, Debug)]
pub struct RSVP {
  pub uid: String,
  pub pwd: String,
  pub email: String,
  pub invite: String,
}

#[derive(Deserialize, Debug)]
pub struct Login {
  pub uid: String,
  pub pwd: String,
}

#[derive(Deserialize, Debug)]
pub struct ResetPassword {
  pub uid: String,
}

#[derive(Serialize, Debug)]
pub struct PwdReset {
  pub userid: i64,
  pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct SetPassword {
  pub uid: String,
  pub newpwd: String,
  pub reset_key: Uuid,
}

#[derive(Deserialize, Debug)]
pub struct ChangePassword {
  pub oldpwd: String,
  pub newpwd: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChangeEmail {
  pub pwd: String,
  pub email: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WhatMessage {
  pub what: String,
  pub data: Option<serde_json::Value>,
}
