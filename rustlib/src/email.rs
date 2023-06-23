use crate::error;
use crate::util;
use lettre::{transport::smtp::response::Response, Message, SmtpTransport, Transport};
use log::info;

pub fn send_newemail_confirmation(
  appname: &str,
  domain: &str,
  mainsite: &str,
  email: &str,
  uid: &str,
  newemail_token: &str,
) -> Result<Response, error::Error> {
  info!("Sending email change confirmation for user: {}", uid);
  let email = Message::builder()
    .from(format!("no-reply@{}", domain).parse()?)
    .to(email.parse()?)
    .subject(format!("change {} email", appname).to_string())
    .body(format!(
      "Click the link to change to your new email, {} user '{}'!\n\
       {}/newemail/{}/{}",
      appname, uid, mainsite, uid, newemail_token
    ))?;

  // to help with registration for desktop use, or if the server is barred from sending email.
  util::write_string(
    "last-email-change.txt",
    (format!(
      "Click the link to change to your new email, {} user '{}'!\n\
       {}/newemail/{}/{}",
      appname, uid, mainsite, uid, newemail_token
    ))
    .to_string()
    .as_str(),
  )?;

  let mailer = SmtpTransport::unencrypted_localhost();
  // Send the email
  mailer.send(&email).map_err(|e| e.into())
}

pub fn send_registration(
  appname: &str,
  domain: &str,
  mainsite: &str,
  email: &str,
  uid: &str,
  reg_id: &str,
) -> Result<Response, error::Error> {
  info!(
    "Sending registration email for user: {}, email: {}",
    uid, email
  );
  let email = Message::builder()
    .from(format!("no-reply@{}", domain).parse()?)
    .to(email.parse()?)
    .subject(format!("{} registration", appname).to_string())
    .body(format!(
      "Click the link to complete registration, {} user '{}'!\n\
       {}/register/{}/{}",
      appname, uid, mainsite, uid, reg_id
    ))?;

  // to help with registration for desktop use, or if the server is barred from sending email.
  util::write_string(
    "last-email.txt",
    (format!(
      "Click the link to complete registration, {} user '{}'!\n\
       {}/register/{}/{}",
      appname, uid, mainsite, uid, reg_id
    ))
    .to_string()
    .as_str(),
  )?;

  let mailer = SmtpTransport::unencrypted_localhost();
  // Send the email
  mailer.send(&email).map_err(|e| e.into())
}

pub fn send_reset(
  appname: &str,
  domain: &str,
  mainsite: &str,
  email: &str,
  username: &str,
  reset_id: &str,
) -> Result<Response, error::Error> {
  info!("Sending reset email for user: {}", username);

  let email = Message::builder()
    .from(format!("no-reply@{}", domain).parse()?)
    .to(email.parse()?)
    .subject(format!("{} password reset", appname).to_string())
    .body(format!(
      "Click the link to complete password reset, {} user '{}'!\n\
       {}/reset/{}/{}",
      appname, username, mainsite, username, reset_id
    ))?;

  // to help with reset for desktop use, or if the server is barred from sending email.
  util::write_string(
    "last-email.txt",
    (format!(
      "Click the link to complete reset, {} user '{}'!\n\
       {}/reset/{}/{}",
      appname, username, mainsite, username, reset_id
    ))
    .to_string()
    .as_str(),
  )?;

  let mailer = SmtpTransport::unencrypted_localhost();
  // Send the email
  mailer.send(&email).map_err(|e| e.into())
}

pub fn send_registration_notification(
  appname: &str,
  domain: &str,
  adminemail: &str,
  email: &str,
  uid: &str,
  _reg_id: &str,
) -> Result<Response, error::Error> {
  info!("sending registration notification to admin!");
  let email = Message::builder()
    .from(format!("no-reply@{}", domain).parse()?)
    .to(adminemail.parse()?)
    .subject(format!("{} new registration, uid: {}", appname, uid).to_string())
    .body(format!(
      "Someones trying to register for {}! {}, {}",
      appname, uid, email
    ))?;

  let mailer = SmtpTransport::unencrypted_localhost();
  // Send the email
  mailer.send(&email).map_err(|e| e.into())
}

pub fn send_rsvp_notification(
  appname: &str,
  domain: &str,
  adminemail: &str,
  email: &str,
  uid: &str,
) -> Result<Response, error::Error> {
  info!("sending rsvp notification to admin!");
  let email = Message::builder()
    .from(format!("no-reply@{}", domain).parse()?)
    .to(adminemail.parse()?)
    .subject(format!("{} new rsvp, uid: {}", appname, uid).to_string())
    .body(format!(
      "Someones has rsvped to a new-user invite for {}! {}, {}",
      appname, uid, email
    ))?;

  let mailer = SmtpTransport::unencrypted_localhost();
  // Send the email
  mailer.send(&email).map_err(|e| e.into())
}
