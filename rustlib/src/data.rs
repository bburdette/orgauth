use elm_rs::{Elm, ElmDecode, ElmEncode};
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

// #[derive(Elm, ElmDecode, ElmEncode, Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
// pub enum UserId {
//   Uid(i64),
// }

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
  pub remote_registration: bool,
}

#[derive(Elm, ElmDecode, ElmEncode, Serialize, Deserialize, Debug, Clone)]
pub struct LoginData {
  pub userid: i64,
  pub uuid: Uuid,
  pub name: String,
  pub email: String,
  pub admin: bool,
  pub active: bool,
  pub data: Option<String>,
}

#[derive(Elm, ElmDecode, ElmEncode, Serialize, Deserialize, Debug, Clone)]
pub struct AdminSettings {
  pub open_registration: bool,
  pub send_emails: bool,
  pub non_admin_invite: bool,
  pub remote_registration: bool,
}

pub fn admin_settings(config: &Config) -> AdminSettings {
  AdminSettings {
    open_registration: config.open_registration,
    send_emails: config.send_emails,
    non_admin_invite: config.non_admin_invite,
    remote_registration: config.remote_registration,
  }
}

#[derive(Elm, ElmDecode, ElmEncode, Clone, Deserialize, Serialize, Debug)]
pub struct User {
  pub id: i64,
  pub uuid: Uuid,
  pub name: String,
  pub hashwd: String,
  pub salt: String,
  pub email: String,
  pub registration_key: Option<String>,
  pub admin: bool,
  pub active: bool,
  pub remote_url: Option<String>,
  pub cookie: Option<String>,
}

// Represents a remote user that is not registered on this server.
#[derive(Elm, ElmDecode, ElmEncode, Clone, Deserialize, Serialize, Debug)]
pub struct PhantomUser {
  pub id: i64,
  pub uuid: Uuid,
  pub name: String,
  pub active: bool,
  pub extra_login_data: String,
}

#[derive(Elm, ElmDecode, ElmEncode, Clone, Deserialize, Serialize, Debug)]
pub struct UserInvite {
  pub email: Option<String>,
  pub token: String,
  pub url: String,
  pub data: Option<String>,
  pub creator: i64,
}

#[derive(Elm, ElmDecode, ElmEncode, Clone, Deserialize, Serialize, Debug)]
pub struct GetInvite {
  pub email: Option<String>,
  pub data: Option<String>,
}

#[derive(Elm, ElmDecode, ElmEncode, Serialize, Deserialize, Debug)]
pub struct RegistrationData {
  pub uid: String,
  pub pwd: String,
  pub email: String,
  pub remote_url: String,
}

#[derive(Elm, ElmDecode, ElmEncode, Serialize, Deserialize, Debug)]
pub struct RSVP {
  pub uid: String,
  pub pwd: String,
  pub email: String,
  pub invite: String,
}

#[derive(Elm, ElmDecode, ElmEncode, Serialize, Deserialize, Debug)]
pub struct Login {
  pub uid: String,
  pub pwd: String,
}

#[derive(Elm, ElmDecode, ElmEncode, Serialize, Deserialize, Debug)]
pub struct ResetPassword {
  pub uid: String,
}

#[derive(Elm, ElmDecode, ElmEncode, Serialize, Deserialize, Debug)]
pub struct PwdReset {
  pub userid: i64,
  pub url: String,
}

#[derive(Elm, ElmDecode, ElmEncode, Serialize, Deserialize, Debug)]
pub struct SetPassword {
  pub uid: String,
  pub newpwd: String,
  pub reset_key: Uuid,
}

#[derive(Elm, ElmDecode, ElmEncode, Serialize, Deserialize, Debug)]
pub struct ChangePassword {
  pub oldpwd: String,
  pub newpwd: String,
}

#[derive(Elm, ElmDecode, ElmEncode, Serialize, Deserialize, Debug, Clone)]
pub struct ChangeEmail {
  pub pwd: String,
  pub email: String,
}

#[derive(Elm, ElmDecode, ElmEncode, Serialize, Deserialize, Debug)]
pub enum UserRequest {
  UrqRegister(RegistrationData),
  UrqLogin(Login),
  UrqReadInvite(String),
  UrqRSVP(RSVP),
  UrqResetPassword(ResetPassword),
  UrqSetPassword(SetPassword),
  UrqLogout,
  UrqAuthedRequest(AuthedRequest),
}

#[derive(Elm, ElmDecode, ElmEncode, Serialize, Deserialize, Debug)]
pub enum AuthedRequest {
  AthGetInvite(GetInvite),
  AthChangePassword(ChangePassword),
  AthChangeEmail(ChangeEmail),
  AthReadRemoteUser(i64),
}

#[derive(Elm, ElmDecode, ElmEncode, Deserialize, Serialize, Debug)]
pub enum UserResponse {
  UrpRegistrationSent,
  UrpUserExists,
  UrpUnregisteredUser,
  UrpInvalidUserOrPwd,
  UrpInvalidUserId,
  UrpBlankUserName,
  UrpBlankPassword,
  UrpNotLoggedIn,
  UrpAccountDeactivated,
  UrpLoggedIn(LoginData),
  UrpLoggedOut,
  UrpChangedPassword,
  UrpChangedEmail,
  UrpResetPasswordAck,
  UrpSetPasswordAck,
  UrpInvite(UserInvite),
  UrpRemoteRegistrationFailed,
  UrpRemoteUser(PhantomUser),
  UrpNoData,
  UrpServerError(String),
}

#[derive(Elm, ElmDecode, ElmEncode, Deserialize, Serialize, Debug)]
pub enum AdminRequest {
  ArqGetUsers,
  ArqDeleteUser(i64),
  ArqUpdateUser(LoginData),
  ArqGetInvite(GetInvite),
  ArqGetPwdReset(i64),
}

// #[derive(Deserialize, Serialize, Debug)]
// pub struct AdminRequestMessage {
//   pub what: AdminRequest,
//   pub data: Option<serde_json::Value>,
// }

#[derive(Elm, ElmDecode, ElmEncode, Deserialize, Serialize, Debug)]
pub enum AdminResponse {
  ArpUsers(Vec<LoginData>),
  ArpUserDeleted(i64),
  ArpUserNotDeleted(i64),
  ArpNoUserId,
  ArpNoData,
  ArpUserUpdated(LoginData),
  ArpServerError,
  ArpUserInvite(UserInvite),
  ArpPwdReset(PwdReset),
  ArpNotLoggedIn,
  ArpInvalidUserOrPassword,
  ArpAccessDenied,
}

// #[derive(Deserialize, Serialize, Debug)]
// pub struct AdminResponseMessage {
//   pub what: AdminResponse,
//   pub data: Option<serde_json::Value>,
// }
