use crate::data::{
  AdminRequest, AdminResponse, AuthedRequest, Config, Login, PhantomUser, PwdReset,
  RegistrationData, User, UserId, UserInvite, UserRequest, UserResponse,
};
use crate::dbfun;
use crate::email;
use crate::error;
use crate::util;
use crate::util::is_token_expired;
use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse};
use log::{error, info, warn};
use reqwest;
use rusqlite::{params, Connection};
use serde_json;
use sha256;
use std::str::FromStr;
use util::now;
use uuid::Uuid;

pub struct Callbacks {
  pub on_new_user: Box<
    dyn FnMut(
      &Connection,       // <- conn
      &RegistrationData, // <- rd
      Option<String>,    // <- extraLoginData
      Option<String>,    // <- remote_data
      Option<UserId>,    // <- creator
      UserId,            // <- uid
    ) -> Result<(), error::Error>,
  >,
  pub extra_login_data:
    Box<dyn FnMut(&Connection, UserId) -> Result<Option<serde_json::Value>, error::Error>>,
  pub on_delete_user: Box<dyn FnMut(&Connection, UserId) -> Result<bool, error::Error>>,
}

pub trait Tokener {
  fn set(&mut self, uuid: Uuid) -> Result<(), error::Error>;
  fn remove(&mut self);
  fn get(&self) -> Option<Uuid>;
}

pub struct ActixTokener<'a> {
  pub session: &'a Session,
}

impl Tokener for ActixTokener<'_> {
  fn set(&mut self, uuid: Uuid) -> Result<(), error::Error> {
    self.session.insert("token", uuid)?;
    Ok(())
  }
  fn remove(&mut self) {
    self.session.remove("token");
  }
  fn get(&self) -> Option<Uuid> {
    self.session.get("token").unwrap_or(None)
  }
}

pub struct UuidTokener {
  pub uuid: Option<Uuid>,
}

impl Tokener for UuidTokener {
  fn set(&mut self, uuid: Uuid) -> Result<(), error::Error> {
    self.uuid = Some(uuid);
    Ok(())
  }
  fn remove(&mut self) {
    self.uuid = None;
  }
  fn get(&self) -> Option<Uuid> {
    self.uuid
  }
}

pub fn log_user_in(
  tokener: &mut dyn Tokener,
  callbacks: &mut Callbacks,
  conn: &Connection,
  uid: UserId,
) -> Result<UserResponse, error::Error> {
  let mut ld = dbfun::login_data(&conn, uid)?;
  let data = (callbacks.extra_login_data)(&conn, ld.userid)?;
  ld.data = data.map(|x| x.to_string());
  // new token here, and token date.
  let token = Uuid::new_v4();
  // new token has no "prev"
  dbfun::add_token(&conn, uid, token, None)?;
  tokener.set(token)?;

  Ok(UserResponse::UrpLoggedIn(ld))
}

pub async fn user_interface(
  conn: &Connection,
  tokener: &mut dyn Tokener,
  config: &Config,
  callbacks: &mut Callbacks,
  user_uri_path: Option<String>,
  msg: UserRequest,
) -> Result<UserResponse, error::Error> {
  match msg {
    UserRequest::UrqRegister(rd) => {
      if !config.open_registration {
        return Err("new user registration is disabled".into());
      }
      // do the registration thing.
      // user already exists?
      match dbfun::read_user_by_name(&conn, rd.uid.as_str()) {
        Ok(mut user) => {
          match user.registration_key {
            Some(ref reg_key) => {
              // user exists but has not yet registered.  allow update of user data.

              if rd.pwd.trim() == "" {
                return Ok(UserResponse::UrpBlankPassword);
              }

              user.email = rd.email;

              dbfun::update_user(&conn, &user)?;
              if sha256::digest(
                (rd.pwd.clone() + user.salt.as_str())
                  .into_bytes()
                  .as_slice(),
              ) != user.hashwd
              {
                // change password.
                dbfun::override_password(&conn, user.id, rd.pwd)?;
              }

              if config.send_emails {
                // send a registration email.
                email::send_registration(
                  config.appname.as_str(),
                  config.emaildomain.as_str(),
                  config.mainsite.as_str(),
                  user.email.as_str(),
                  rd.uid.as_str(),
                  reg_key.as_str(),
                )?;
                // notify the admin.
                email::send_registration_notification(
                  config.appname.as_str(),
                  config.emaildomain.as_str(),
                  config.admin_email.as_str(),
                  user.email.as_str(),
                  rd.uid.as_str(),
                  reg_key.as_str(),
                )?;
                Ok(UserResponse::UrpRegistrationSent)
              } else {
                log_user_in(tokener, callbacks, &conn, user.id)
              }
            }
            None => {
              // if user is already registered, can't register again.
              // err - user exists.
              Ok(UserResponse::UrpUserExists)
            }
          }
        }
        Err(error::Error::Rusqlite(rusqlite::Error::QueryReturnedNoRows)) => {
          // user does not exist, which is what we want for a new user.

          // check for non-blank uid and password.
          if rd.uid.trim() == "" {
            return Ok(UserResponse::UrpBlankUserName);
          }
          if rd.pwd.trim() == "" {
            return Ok(UserResponse::UrpBlankPassword);
          }

          // are we doing remote registration?
          if config.remote_registration && rd.remote_url != "" {
            let user_uri_path = match user_uri_path {
              Some(s) => s,
              None => return Ok(UserResponse::UrpRemoteRegistrationFailed),
            };
            // try to log in to an existing account on the remote!
            let client = reqwest::Client::new();
            let l = UserRequest::UrqLogin(Login {
              uid: rd.uid.clone(),
              pwd: rd.pwd.clone(),
            });

            // TODO: this uri is dependent on the remote app!
            // which should be the same as this app, but still.
            let user_uri = format!("{}/{}", rd.remote_url, user_uri_path);

            let res = client.post(user_uri).json(&l).send().await?;
            let cookie = match res.headers().get(reqwest::header::SET_COOKIE) {
              Some(ck) => Some(
                ck.to_str()
                  .map_err(|_| error::Error::String("invalid cookie".to_string()))?
                  .to_string(),
              ),
              None => None,
            };

            let wm = serde_json::from_value::<UserResponse>(res.json().await?)?;
            if let UserResponse::UrpLoggedIn(ld) = wm {
              // make a local user record.
              // write a user record.
              let uid = dbfun::new_user(
                &conn,
                &rd,
                Option::None,
                Option::None,
                // invite.data,
                false,
                Some(ld.uuid),
                Option::None,
                // Some(invite.creator),
                Some(rd.remote_url.clone()),
                ld.data.map(|x| x.to_string()),
                cookie,
                &mut callbacks.on_new_user,
              )?;

              log_user_in(tokener, callbacks, &conn, uid)
            } else {
              Ok(UserResponse::UrpRemoteRegistrationFailed)
            }
          } else {
            // get email from 'data'.
            let registration_key = Uuid::new_v4().to_string();
            let uid = dbfun::new_user(
              &conn,
              &rd,
              if config.send_emails {
                Some(registration_key.clone().to_string())
              } else {
                // Instant registration for send_emails = false.
                None
              },
              None,
              false, // NOT admin by default.
              None,
              None,
              None,
              None,
              None,
              &mut callbacks.on_new_user,
            )?;

            if config.send_emails {
              // send a registration email.
              email::send_registration(
                config.appname.as_str(),
                config.emaildomain.as_str(),
                config.mainsite.as_str(),
                rd.email.as_str(),
                rd.uid.as_str(),
                registration_key.as_str(),
              )?;

              // notify the admin.
              email::send_registration_notification(
                config.appname.as_str(),
                config.emaildomain.as_str(),
                config.admin_email.as_str(),
                rd.email.as_str(),
                rd.uid.as_str(),
                registration_key.as_str(),
              )?;
              Ok(UserResponse::UrpRegistrationSent)
            } else {
              log_user_in(tokener, callbacks, &conn, uid)
            }
          }
        }
        Err(e) => Err(e),
      }
    }
    UserRequest::UrqRSVP(rsvp) => {
      // invite exists?
      info!("rsvp: {:?}", rsvp.uid);
      let invite =
        match dbfun::read_userinvite(&conn, config.mainsite.as_str(), rsvp.invite.as_str()) {
          Ok(None) => return Err("user invite not found".into()),
          Err(e) => return Err(e),
          Ok(Some(i)) => i,
        };

      // uid already exists?
      match dbfun::read_user_by_name(&conn, rsvp.uid.as_str()) {
        Ok(mut userdata) => {
          // password matches?
          if sha256::digest(
            (rsvp.pwd.clone() + userdata.salt.as_str())
              .into_bytes()
              .as_slice(),
          ) != userdata.hashwd
          {
            // don't distinguish between bad user id and bad pwd
            // maybe would ok for one-time use invites.
            Ok(UserResponse::UrpInvalidUserOrPwd)
          } else if !userdata.active {
            Ok(UserResponse::UrpAccountDeactivated)
          } else {
            match userdata.registration_key {
              Some(_reg_key) => {
                // If an 'unregistered user' - someone who tried the registration through email - gets hold of an
                // invite link, then they can complete registration with that.
                // They do have to use the same password they used from their registration though.
                userdata.registration_key = None;
                dbfun::update_user(&conn, &userdata)?;
              }
              None => (),
            }
            // password matches, account active, already registered

            // delete the invite.
            dbfun::remove_userinvite(&conn, &rsvp.invite.as_str())?;
            // log in.
            log_user_in(tokener, callbacks, &conn, userdata.id)
          }
        }
        Err(_) => {
          // user does not exist, which is what we want for a new user.

          // check for non-blank uid and password.
          if rsvp.uid.trim() == "" {
            return Ok(UserResponse::UrpBlankUserName);
          }
          if rsvp.pwd.trim() == "" {
            return Ok(UserResponse::UrpBlankPassword);
          }

          let rd = RegistrationData {
            uid: rsvp.uid.clone(),
            pwd: rsvp.pwd.clone(),
            email: rsvp.email.clone(),
            remote_url: "".to_string(),
          };

          // write a user record.
          let uid = dbfun::new_user(
            &conn,
            &rd,
            Option::None,
            invite.data,
            false,
            None,
            Some(invite.creator),
            None,
            None,
            None,
            &mut callbacks.on_new_user,
          )?;

          // delete the invite.
          dbfun::remove_userinvite(&conn, &rsvp.invite.as_str())?;

          // notify the admin.
          if config.send_emails {
            match email::send_rsvp_notification(
              config.appname.as_str(),
              config.emaildomain.as_str(),
              config.admin_email.as_str(),
              rsvp.email.as_str(),
              rsvp.uid.as_str(),
            ) {
              Ok(_) => (),
              Err(e) => {
                // warn if error sending email; but keep on with new user login.
                warn!(
                  "error sending rsvp notification for user: {}, {}",
                  rd.uid, e
                )
              }
            }
          }

          // respond with login.
          log_user_in(tokener, callbacks, &conn, uid)
        }
      }
    }
    UserRequest::UrqReadInvite(token) => {
      match dbfun::read_userinvite(&conn, config.mainsite.as_str(), token.as_str()) {
        Ok(None) => Err("user invite not found".into()),
        Err(e) => Err(e),
        Ok(Some(invite)) => Ok(UserResponse::UrpInvite(invite)),
      }
    }
    UserRequest::UrqLogin(login) => {
      let userdata = dbfun::read_user_by_name(&conn, login.uid.as_str())?;
      match userdata.registration_key {
        Some(_reg_key) => Ok(UserResponse::UrpUnregisteredUser),
        None => {
          if userdata.active {
            if sha256::digest(
              (login.pwd.clone() + userdata.salt.as_str())
                .into_bytes()
                .as_slice(),
            ) != userdata.hashwd
            {
              // don't distinguish between bad user id and bad pwd!
              Ok(UserResponse::UrpInvalidUserOrPwd)
            } else {
              log_user_in(tokener, callbacks, &conn, userdata.id)
            }
          } else {
            Ok(UserResponse::UrpAccountDeactivated)
          }
        }
      }
    }
    UserRequest::UrqLogout => {
      tokener.remove();

      Ok(UserResponse::UrpLoggedOut)
    }
    UserRequest::UrqResetPassword(reset_password) => {
      let userdata = dbfun::read_user_by_name(&conn, reset_password.uid.as_str())?;
      match userdata.registration_key {
        Some(_reg_key) => Ok(UserResponse::UrpUnregisteredUser),
        None => {
          let reset_key = Uuid::new_v4();

          // make 'newpassword' record.
          dbfun::add_newpassword(&conn, userdata.id, reset_key.clone())?;

          if config.send_emails {
            // send reset email.
            email::send_reset(
              config.appname.as_str(),
              config.emaildomain.as_str(),
              config.mainsite.as_str(),
              userdata.email.as_str(),
              userdata.name.as_str(),
              reset_key.to_string().as_str(),
            )?;
          }

          Ok(UserResponse::UrpResetPasswordAck)
        }
      }
    }
    UserRequest::UrqSetPassword(set_password) => {
      let mut userdata = dbfun::read_user_by_name(&conn, set_password.uid.as_str())?;
      match userdata.registration_key {
        Some(_reg_key) => Ok(UserResponse::UrpUnregisteredUser),
        None => {
          let npwd = dbfun::read_newpassword(&conn, userdata.id, set_password.reset_key)?;

          if is_token_expired(config.reset_token_expiration_ms, npwd) {
            Ok(UserResponse::UrpServerError("password reset failed".into()))
          } else {
            userdata.hashwd = sha256::digest(
              (set_password.newpwd + userdata.salt.as_str())
                .into_bytes()
                .as_slice(),
            );
            dbfun::remove_newpassword(&conn, userdata.id, set_password.reset_key)?;
            dbfun::update_user(&conn, &userdata)?;
            Ok(UserResponse::UrpSetPasswordAck)
          }
        }
      }
    }
    UserRequest::UrqAuthedRequest(ar) => {
      // are we logged in?
      match tokener.get() {
        None => Ok(UserResponse::UrpNotLoggedIn),
        Some(token) => {
          let conn = dbfun::connection_open(config.db.as_path())?;
          match dbfun::read_user_by_token_api(
            &conn,
            token,
            config.login_token_expiration_ms,
            config.regen_login_tokens,
          ) {
            Err(_e) => Ok(UserResponse::UrpInvalidUserOrPwd),
            Ok(userdata) => {
              // finally!  processing messages as logged in user.
              user_interface_loggedin(&config, &conn, callbacks, userdata.id, &ar).await
            }
          }
        }
      }
    }
  }
}

pub async fn user_interface_loggedin(
  config: &Config,
  conn: &Connection,
  callbacks: &mut Callbacks,
  uid: UserId,
  msg: &AuthedRequest,
) -> Result<UserResponse, error::Error> {
  match msg {
    AuthedRequest::AthChangePassword(cp) => {
      dbfun::change_password(&conn, uid, cp)?;
      Ok(UserResponse::UrpChangedPassword)
    }
    AuthedRequest::AthChangeEmail(cp) => {
      let (name, token) = dbfun::change_email(&conn, uid, cp.clone())?;
      // send a confirmation email.
      if config.send_emails {
        email::send_newemail_confirmation(
          config.appname.as_str(),
          config.emaildomain.as_str(),
          config.mainsite.as_str(),
          cp.email.as_str(),
          name.as_str(),
          token.to_string().as_str(),
        )?;
      }

      Ok(UserResponse::UrpChangedEmail)
    }
    AuthedRequest::AthChangeRemoteUrl(cp) => {
      let uresp = dbfun::change_remote_url(&conn, uid, "user".to_string(), &cp).await?;

      Ok(uresp)
    }
    AuthedRequest::AthReadRemoteUser(id) => {
      match (
        dbfun::read_user_by_id(&conn, id.clone()),
        (callbacks.extra_login_data)(&conn, id.clone()),
      ) {
        (Ok(userdata), Ok(extralogindata)) => Ok(UserResponse::UrpRemoteUser(PhantomUser {
          id: userdata.id,
          uuid: userdata.uuid,
          name: userdata.name,
          active: userdata.active,
          extra_login_data: extralogindata
            .map(|x| x.to_string())
            .unwrap_or("".to_string()),
        })),
        (Err(_e), _) => Ok(UserResponse::UrpInvalidUserId),
        (_, Err(e)) => Ok(UserResponse::UrpServerError(e.to_string())),
      }
    }
    AuthedRequest::AthGetInvite(gi) => {
      if config.non_admin_invite {
        let invite_key = Uuid::new_v4();

        dbfun::add_userinvite(
          &conn,
          invite_key.clone(),
          gi.email.clone(),
          uid,
          gi.data.clone(),
        )?;
        Ok(UserResponse::UrpInvite(UserInvite {
          email: None,
          token: invite_key.to_string(),
          url: format!("{}/invite/{}", config.mainsite, invite_key.to_string()),
          creator: uid,
          data: gi.data.clone(),
        }))
      } else {
        Err("non-admin user invites are disabled!".into())
      }
    }
  }
}

pub fn admin_interface_check(
  tokener: &mut dyn Tokener,

  config: &Config,
  callbacks: &mut Callbacks,
  msg: AdminRequest,
) -> Result<AdminResponse, error::Error> {
  match tokener.get() {
    None => Ok(AdminResponse::ArpNotLoggedIn),
    Some(token) => {
      let conn = dbfun::connection_open(config.db.as_path())?;
      match dbfun::read_user_by_token_api(
        &conn,
        token,
        config.login_token_expiration_ms,
        config.regen_login_tokens,
      ) {
        Err(_e) => Ok(AdminResponse::ArpInvalidUserOrPassword),
        Ok(userdata) => {
          if userdata.admin {
            // finally!  processing messages as logged in user.
            admin_interface(&conn, &config, &userdata, callbacks, &msg)
          } else {
            Ok(AdminResponse::ArpAccessDenied)
          }
        }
      }
    }
  }
}

pub fn admin_interface(
  conn: &Connection,
  config: &Config,
  user: &User,
  callbacks: &mut Callbacks,
  msg: &AdminRequest,
) -> Result<AdminResponse, error::Error> {
  match msg {
    AdminRequest::ArqGetUsers => {
      let users = dbfun::read_users(&conn, &mut callbacks.extra_login_data)?;
      Ok(AdminResponse::ArpUsers(users))
    }
    AdminRequest::ArqDeleteUser(uid) => {
      let uid = uid.clone();
      conn.execute("begin transaction", params!())?;
      if (callbacks.on_delete_user)(&conn, uid)? {
        dbfun::delete_user(&conn, uid)?;
        conn.execute("commit", params!())?;
        Ok(AdminResponse::ArpUserDeleted(uid))
      } else {
        conn.execute("rollback", params!())?;
        Ok(AdminResponse::ArpUserNotDeleted(uid))
      }
    }
    AdminRequest::ArqUpdateUser(ld) => {
      dbfun::update_login_data(&conn, &ld)?;
      let uld = dbfun::login_data(&conn, ld.userid)?;
      Ok(AdminResponse::ArpUserUpdated(uld))
    }
    AdminRequest::ArqGetInvite(gi) => {
      let invite_key = Uuid::new_v4();

      dbfun::add_userinvite(
        &conn,
        invite_key.clone(),
        gi.email.clone(),
        user.id,
        gi.data.clone(),
      )?;
      Ok(AdminResponse::ArpUserInvite(UserInvite {
        email: None,
        token: invite_key.to_string(),
        url: format!("{}/invite/{}", config.mainsite, invite_key.to_string()),
        creator: user.id,
        data: gi.data.clone(),
      }))
    }
    AdminRequest::ArqGetPwdReset(uid) => {
      let uid = uid.clone();
      let user = dbfun::read_user_by_id(&conn, uid)?;
      let reset_key = Uuid::new_v4();
      // make 'newpassword' record.
      dbfun::add_newpassword(&conn, uid, reset_key.clone())?;

      // send reset email.
      if config.send_emails {
        email::send_reset(
          config.appname.as_str(),
          config.emaildomain.as_str(),
          config.mainsite.as_str(),
          user.email.as_str(),
          user.name.as_str(),
          reset_key.to_string().as_str(),
        )?;
      }

      Ok(AdminResponse::ArpPwdReset(PwdReset {
        userid: uid,
        url: format!(
          "{}/reset/{}/{}",
          config.mainsite,
          user.name,
          reset_key.to_string()
        ),
      }))
    }
  }
}

pub fn register(data: &Config, req: HttpRequest) -> HttpResponse {
  info!("registration: uid: {:?}", req.match_info().get("uid"));
  match dbfun::connection_open(data.db.as_path()) {
    Ok(conn) => match (req.match_info().get("uid"), req.match_info().get("key")) {
      (Some(uid), Some(key)) => {
        // read user record.  does the reg key match?
        match dbfun::read_user_by_name(&conn, uid) {
          Ok(user) => {
            if user.registration_key == Some(key.to_string()) {
              let mut mu = user;
              mu.registration_key = None;
              match dbfun::update_user(&conn, &mu) {
                Ok(_) => HttpResponse::Ok().body(
                  format!(
                    "<h1>You are registered!<h1> <a href=\"{}\">\
                       Proceed to the main site</a>",
                    data.mainsite
                  )
                  .to_string(),
                ),
                Err(_e) => HttpResponse::Ok().body("<h1>registration failed</h1>".to_string()),
              }
            } else {
              HttpResponse::Ok().body("<h1>registration failed</h1>".to_string())
            }
          }
          Err(_e) => HttpResponse::Ok().body("registration key or user doesn't match".to_string()),
        }
      }
      _ => HttpResponse::Ok().body("Uid, key not found!".to_string()),
    },

    Err(_e) => HttpResponse::Ok().body("<h1>registration failed</h1>".to_string()),
  }
}

pub fn new_email(data: &Config, req: HttpRequest) -> HttpResponse {
  info!("new email: uid: {:?}", req.match_info().get("uid"));
  match dbfun::connection_open(data.db.as_path()) {
    Ok(conn) => match (req.match_info().get("uid"), req.match_info().get("token")) {
      (Some(uid), Some(tokenstr)) => {
        match Uuid::from_str(tokenstr) {
          Err(_e) => HttpResponse::BadRequest().body("invalid token".to_string()),
          Ok(token) => {
            // read user record.  does the reg key match?
            match dbfun::read_user_by_name(&conn, uid) {
              Ok(user) => {
                match dbfun::read_newemail(&conn, user.id, token) {
                  Ok((email, tokendate)) => {
                    match now() {
                      Err(_e) => HttpResponse::InternalServerError()
                        .body("<h1>'now' failed!</h1>".to_string()),

                      Ok(now) => {
                        if (now - tokendate) > data.email_token_expiration_ms {
                          // TODO token expired?
                          HttpResponse::UnprocessableEntity()
                            .body("<h1>email change failed - token expired</h1>".to_string())
                        } else {
                          // put the email in the user record and update.
                          let mut mu = user.clone();
                          mu.email = email;
                          match dbfun::update_user(&conn, &mu) {
                            Ok(_) => {
                              // delete the change email token record.
                              match dbfun::remove_newemail(&conn, user.id, token) {
                                Ok(_) => (),
                                Err(e) => error!("error removing newemail record: {:?}", e),
                              }
                              HttpResponse::Ok().body(
                                format!(
                                  "<h1>Email address changed!<h1> <a href=\"{}\">\
                                   Proceed to the main site</a>",
                                  data.mainsite
                                )
                                .to_string(),
                              )
                            }
                            Err(_e) => HttpResponse::InternalServerError()
                              .body("<h1>email change failed</h1>".to_string()),
                          }
                        }
                      }
                    }
                  }
                  Err(_e) => HttpResponse::InternalServerError()
                    .body("<h1>email change failed</h1>".to_string()),
                }
              }
              Err(_e) => HttpResponse::BadRequest()
                .body("email change token or user doesn't match".to_string()),
            }
          }
        }
      }
      _ => HttpResponse::BadRequest().body("username or token not found!".to_string()),
    },

    Err(_e) => {
      HttpResponse::InternalServerError().body("<h1>database connection failed</h1>".to_string())
    }
  }
}
