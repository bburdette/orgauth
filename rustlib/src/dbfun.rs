use crate::data::{
  ChangeEmail, ChangePassword, ChangeRemoteUrl, Login, LoginData, User, UserId, UserInvite,
  UserRequest, UserResponse,
};
use crate::data::{Config, RegistrationData};
use crate::error;
use crate::util::{is_token_expired, now, salt_string};
use actix_session::Session;
use log::{error, info, warn};
use rusqlite::{params, Connection};
use sha256;
use simple_error::bail;
use std::path::Path;
use std::time::Duration;
use uuid::Uuid;

pub fn connection_open(dbfile: &Path) -> Result<Connection, error::Error> {
  let conn = Connection::open(dbfile)?;

  // conn.busy_timeout(Duration::from_millis(500))?;
  conn.busy_handler(Some(|count| {
    info!("busy_handler: {}", count);
    let d = Duration::from_millis(500);
    std::thread::sleep(d);
    true
  }))?;

  conn.execute("PRAGMA foreign_keys = true;", params![])?;

  Ok(conn)
}

const REGEN_MS: i64 = 10 * 1000;

pub fn new_user(
  conn: &Connection,
  rd: &RegistrationData,
  registration_key: Option<String>,
  data: Option<String>,
  admin: bool,
  uuid: Option<Uuid>,
  creator: Option<UserId>,
  remote_url: Option<String>,
  remote_data: Option<String>,
  cookie: Option<String>,
  on_new_user: &mut Box<
    dyn FnMut(
      &Connection,
      &RegistrationData,
      Option<String>,
      Option<String>, // <- remote_data
      Option<UserId>,
      UserId,
    ) -> Result<(), error::Error>,
  >,
) -> Result<UserId, error::Error> {
  let now = now()?;
  let salt = salt_string();
  let hashwd = sha256::digest((rd.pwd.clone() + salt.as_str()).into_bytes().as_slice());
  let uuid = match uuid {
    None => uuid::Uuid::new_v4(),
    Some(uuid) => uuid,
  };

  match (&cookie, &remote_url) {
    (Some(_), Some(_)) => (),
    (None, None) => (),
    (Some(_), None) => return Err("remote_url required with cookie".into()),
    (None, Some(_)) => return Err("cookie required with remote_url".into()),
  }

  // make a user record.
  conn.execute(
    "insert into orgauth_user (name, uuid, hashwd, salt, email, admin, active, registration_key, remote_url, cookie, createdate)
      values (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?9, ?10)",
    params![rd.uid.to_lowercase(), uuid.to_string(), hashwd, salt, rd.email, admin, registration_key, remote_url, cookie, now],
  )?;

  let uid = UserId::Uid(conn.last_insert_rowid());

  (on_new_user)(&conn, &rd, data, remote_data, creator, uid)?;

  Ok(uid)
}

pub fn phantom_user(
  conn: &Connection,
  name: &String,
  uuid: Uuid,
  extra_login_data: Option<String>,
  active: bool,
  on_new_user: &mut Box<
    dyn FnMut(
      &Connection,
      &RegistrationData,
      Option<String>,
      Option<String>,
      Option<UserId>,
      UserId,
    ) -> Result<(), error::Error>,
  >,
) -> Result<UserId, error::Error> {
  let now = now()?;
  let rd = RegistrationData {
    uid: name.to_lowercase(),
    pwd: "".to_string(),
    email: "".to_string(),
    remote_url: "".to_string(),
  };

  // make a user record.
  conn.execute(
    "insert into orgauth_user (name, uuid, hashwd, salt, email, admin, active, registration_key, createdate)
      values (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, ?8)",
    params![name.to_lowercase(), uuid.to_string(), "phantom", "phantom", "phantom", active,"phantom", now],
  )?;

  let uid = UserId::Uid(conn.last_insert_rowid());

  // TODO: need to use remote note uuid??
  (on_new_user)(&conn, &rd, None, extra_login_data, None, uid)?;

  Ok(uid)
}

// pub fn user_id(conn: &Connection, name: &str) -> Result<i64, error::Error> {
pub fn user_id(conn: &Connection, name: &str) -> Result<UserId, error::Error> {
  let id: i64 = conn.query_row(
    "select id from orgauth_user
      where orgauth_user.name = ?1",
    params![name.to_lowercase()],
    |row| Ok(row.get(0)?),
  )?;
  Ok(UserId::Uid(id))
}

pub fn login_data(conn: &Connection, uid: UserId) -> Result<LoginData, error::Error> {
  let user = read_user_by_id(&conn, uid)?;
  Ok(LoginData {
    userid: uid,
    uuid: user.uuid,
    name: user.name,
    email: user.email,
    admin: user.admin,
    active: user.active,
    remote_url: user.remote_url,
    data: None,
  })
}

pub fn login_data_cb(
  conn: &Connection,
  uid: UserId,
  extra_login_data: &mut Box<
    dyn FnMut(&Connection, UserId) -> Result<Option<serde_json::Value>, error::Error>,
  >,
) -> Result<LoginData, error::Error> {
  let user = read_user_by_id(&conn, uid)?;
  Ok(LoginData {
    userid: uid,
    uuid: user.uuid,
    name: user.name,
    email: user.email,
    admin: user.admin,
    active: user.active,
    remote_url: user.remote_url,
    data: extra_login_data(&conn, uid)?.map(|x| x.to_string()),
  })
}

pub fn update_login_data(conn: &Connection, ld: &LoginData) -> Result<(), error::Error> {
  let mut user = read_user_by_id(&conn, ld.userid)?;
  user.name = ld.name.to_lowercase();
  user.email = ld.email.clone();
  user.admin = ld.admin;
  user.active = ld.active;
  user.remote_url = ld.remote_url.clone();
  update_user(&conn, &user)
}

pub fn read_users(
  conn: &Connection,
  extra_login_data: &mut Box<
    dyn FnMut(&Connection, UserId) -> Result<Option<serde_json::Value>, error::Error>,
  >,
) -> Result<Vec<LoginData>, error::Error> {
  let mut pstmt = conn.prepare(
    // return zklinks that link to or from notes that link to 'public'.
    "select id from orgauth_user",
  )?;

  let r = Ok(
    pstmt
      .query_map(params![], |row| {
        let id = UserId::Uid(row.get(0)?);
        Ok(id)
      })?
      .filter_map(|rid| match rid {
        Ok(id) => login_data_cb(&conn, id, extra_login_data).ok(),
        Err(_) => None,
      })
      .collect(),
  );
  r
}

pub fn read_user_by_name(conn: &Connection, name: &str) -> Result<User, error::Error> {
  let user = conn.query_row_and_then(
    "select id, uuid, hashwd, salt, email, registration_key, admin, active, remote_url, cookie
      from orgauth_user where name = ?1",
    params![name.to_lowercase()],
    |row| {
      Ok::<_, error::Error>(User {
        id: UserId::Uid(row.get(0)?),
        uuid: Uuid::parse_str(row.get::<usize, String>(1)?.as_str())?,
        name: name.to_lowercase(),
        hashwd: row.get(2)?,
        salt: row.get(3)?,
        email: row.get(4)?,
        registration_key: row.get(5)?,
        admin: row.get(6)?,
        active: row.get(7)?,
        remote_url: row.get(8)?,
        cookie: row.get(9)?,
      })
    },
  )?;

  Ok(user)
}

pub fn read_user_by_id(conn: &Connection, id: UserId) -> Result<User, error::Error> {
  let user = conn.query_row_and_then(
    "select id, uuid, name, hashwd, salt, email, registration_key, admin, active, remote_url, cookie
      from orgauth_user where id = ?1",
    params![Into::<i64>::into(id)],
    |row| {
      Ok::<_, error::Error>(User {
        id: UserId::Uid(row.get(0)?),
        uuid: Uuid::parse_str(row.get::<usize, String>(1)?.as_str())?,
        name: row.get(2)?,
        hashwd: row.get(3)?,
        salt: row.get(4)?,
        email: row.get(5)?,
        registration_key: row.get(6)?,
        admin: row.get(7)?,
        active: row.get(8)?,
        remote_url: row.get(9)?,
        cookie: row.get(10)?,
      })
    },
  )?;

  Ok(user)
}

pub fn read_user_by_uuid(conn: &Connection, uuid: &Uuid) -> Result<User, error::Error> {
  let user = conn.query_row_and_then(
    "select id, uuid, name, hashwd, salt, email, registration_key, admin, active, remote_url, cookie
      from orgauth_user where uuid = ?1",
    params![uuid.to_string().as_str()],
    |row| {
      Ok::<_, error::Error>(User {
        id: UserId::Uid(row.get(0)?),
        uuid: Uuid::parse_str(row.get::<usize, String>(1)?.as_str())?,
        name: row.get(2)?,
        hashwd: row.get(3)?,
        salt: row.get(4)?,
        email: row.get(5)?,
        registration_key: row.get(6)?,
        admin: row.get(7)?,
        active: row.get(8)?,
        remote_url: row.get(9)?,
        cookie: row.get(10)?,
      })
    },
  )?;

  Ok(user)
}

struct TokenInfo {
  tokendate: i64,
  regendate: Option<i64>,
  prevtoken: Option<String>,
}

fn read_user_by_token(conn: &Connection, token: Uuid) -> Result<(User, TokenInfo), error::Error> {
  let (user, tokendate, regendate, prevtoken) : (User, i64, Option<i64>, Option<String>) = conn.query_row_and_then(
    "select id, uuid, name, hashwd, salt, email, registration_key, admin, active, remote_url, cookie,
        orgauth_token.tokendate, orgauth_token.regendate, orgauth_token.prevtoken
      from orgauth_user, orgauth_token where orgauth_user.id = orgauth_token.user and orgauth_token.token = ?1",
    params![token.to_string()],
    |row| {
      Ok::<_, error::Error>((
        User {
        id: UserId::Uid(row.get(0)?),
          uuid: Uuid::parse_str(row.get::<usize, String>(1)?.as_str())?,
          name: row.get(2)?,
          hashwd: row.get(3)?,
          salt: row.get(4)?,
          email: row.get(5)?,
          registration_key: row.get(6)?,
          admin: row.get(7)?,
          active: row.get(8)?,
          remote_url: row.get(9)?,
          cookie: row.get(10)?,
        },
        row.get(11)?,
        row.get(12)?,
        row.get(13)?,
      ))
    },
  )?;

  Ok((
    user,
    TokenInfo {
      tokendate,
      regendate,
      prevtoken,
    },
  ))
}

fn check_user(
  user: &User,
  tokendate: i64,
  token_expiration_ms: Option<i64>,
) -> Result<(), error::Error> {
  if !user.active {
    Err("account is inactive".into())
  } else {
    if let Some(texp) = token_expiration_ms {
      if is_token_expired(texp, tokendate) {
        // Err(error::Error::String("login expired".to_string()))
        return Err("login expired".into());
      }
    };

    Ok(())
  }
}

// Use this variant for api calls; doesn't refresh the token
// in regen mode, but does remove prev tokens.
pub fn read_user_by_token_api(
  conn: &Connection,
  token: Uuid,
  token_expiration_ms: Option<i64>,
  regen_login_tokens: bool,
) -> Result<User, error::Error> {
  let (user, tokeninfo) = read_user_by_token(&conn, token)?;

  check_user(&user, tokeninfo.tokendate, token_expiration_ms)?;

  if regen_login_tokens {
    if let Some(pt) = tokeninfo.prevtoken {
      let rdt = now()? - REGEN_MS;

      // delete IF regen is past.

      // prevtoken regen time expired?
      let dc: i64 = conn.query_row(
        "select count(*) from orgauth_token where token = ?1 and regendate < ?2",
        params![pt, rdt],
        |row| Ok(row.get(0)?),
      )?;

      if dc == 1 {
        remove_token_chain(&conn, &pt, &token.to_string())?;

        // clear out prevtoken field
        conn.execute(
          "update orgauth_token set prevtoken = null  where token = ?1",
          params![token.to_string()],
        )?;
      }
    }
  }

  Ok(user)
}

fn remove_token_chain(
  conn: &Connection,
  token: &String,
  keeptoken: &String,
) -> Result<(), error::Error> {
  let pt: Option<String> = conn.query_row(
    "select prevtoken from orgauth_token where token = ?1",
    params![token],
    |row| Ok(row.get(0)?),
  )?;

  if let Some(ref pt) = pt {
    remove_token_chain(&conn, &pt, &keeptoken)?;
  }

  // remove this token AND any tokens that descend from it.
  // EXCEPT for the keeptoken.
  conn.execute(
    "delete from orgauth_token where token = ?1 or (prevtoken = ?1 and token != ?2)",
    params![token, keeptoken],
  )?;

  Ok(())
}

// Use this one when loading a page, when the token will be saved to the browser.
// Not for api calls, where a new token would not be set.
pub fn read_user_with_token_pageload(
  conn: &mut Connection,
  session: &Session,
  token: Uuid,
  regen_login_tokens: bool,
  token_expiration_ms: Option<i64>,
) -> Result<User, error::Error> {
  for _i in 1..10 {
    // since this ftn involves a commit, there can be
    // a conflict with many simultaneous page loads.
    // loop up to 10x until we finally get access.
    match read_user_with_token_pageload_internal(
      conn,
      &session,
      token,
      regen_login_tokens,
      token_expiration_ms,
    ) {
      Ok(user) => return Ok(user),
      Err(error::Error::Rusqlite(rusqlite::Error::SqliteFailure(fe, mbstring))) => {
        warn!("SqliteFailure: {:?}, {:?}", fe, mbstring);
        match fe.code {
          rusqlite::ErrorCode::DatabaseBusy => {
            warn!("database busy sleeping 10");
            std::thread::sleep(Duration::from_millis(10));
            ()
          }
          rusqlite::ErrorCode::DatabaseLocked => {
            warn!("database locked sleeping 10");
            std::thread::sleep(Duration::from_millis(10));
            ()
          }
          _ => return Err(rusqlite::Error::SqliteFailure(fe, mbstring).into()),
        }
      }
      Err(e) => {
        error!("login_data_error {:?}", e);
        return Err(e.into());
      }
    }
  }

  Err("database busy 10x".into())
}

// Use this one when loading a page, when the token will be saved to the browser.
// Not for api calls, where a new token would not be set.
fn read_user_with_token_pageload_internal(
  conn: &mut Connection,
  session: &Session,
  token: Uuid,
  regen_login_tokens: bool,
  token_expiration_ms: Option<i64>,
) -> Result<User, error::Error> {
  let tx = conn.transaction()?;

  let (user, tokeninfo) = read_user_by_token(&tx, token)?;

  check_user(&user, tokeninfo.tokendate, token_expiration_ms)?;

  if regen_login_tokens {
    let nt = match tokeninfo.regendate {
      Some(dt) => {
        let now = now()?;
        if dt + REGEN_MS < now {
          true // expired
        } else {
          false
        }
      }
      None => true,
    };

    if nt {
      // add new login token, and flag old for removal.
      mark_prevtoken(&tx, token)?;
      let new_token = Uuid::new_v4();
      add_token(&tx, user.id, new_token, Some(token), None)?;
      session.insert("token", new_token)?;
    }
  }

  tx.commit()?;

  Ok(user)
}

pub fn add_token(
  conn: &Connection,
  user: UserId,
  token: Uuid,
  prevtoken: Option<Uuid>,
  tokentype: Option<&str>,
) -> Result<(), error::Error> {
  let now = now()?;
  conn.execute(
    "insert into orgauth_token (user, token, tokendate, prevtoken, type)
     values (?1, ?2, ?3, ?4, ?5)",
    params![
      user.to_i64(),
      token.to_string(),
      now,
      prevtoken.map(|s| s.to_string()),
      tokentype
    ],
  )?;

  Ok(())
}

pub fn mark_prevtoken(
  conn: &Connection,
  // token: Uuid,
  prevtoken: Uuid,
) -> Result<bool, error::Error> {
  // set regendate to now.
  let now = now()?;
  let wat = conn.execute(
    "update orgauth_token set regendate = ?1 where token = ?2",
    params![now, prevtoken.to_string()],
  )?;

  match wat {
    1 => Ok(true),
    0 => Ok(false), // could mean token doesn't exist, or regendate expired.
    x => Err(format!("too many records updated: {}", x).into()),
  }
}

pub fn purge_login_tokens(conn: &Connection, token_expiration_ms: i64) -> Result<(), error::Error> {
  let now = now()?;
  let expdt = now - token_expiration_ms;

  struct PurgeToken(i64, String, i64, Option<String>);

  let mut stmt = conn.prepare(
    "select user, token, tokendate, prevtoken from
      orgauth_token where tokendate < ?1",
  )?;

  let c_iter = stmt.query_map(params![expdt], |row| {
    Ok(PurgeToken(
      row.get(0)?,
      row.get(1)?,
      row.get(2)?,
      row.get(3)?,
    ))
  })?;

  for item in c_iter {
    match item {
      Ok(PurgeToken(user, token, _tokendate, _prevtoken)) => {
        info!("purging login token for user {}", user);
        conn.execute(
          "delete from orgauth_token where
          user = ?1 and token = ?2",
          params![user, token],
        )?;
      }
      Err(e) => error!("error purging token: {:?}", e),
    }
  }

  Ok(())
}

pub fn purge_login_tokens_type(
  conn: &Connection,
  token_expiration_ms: i64,
  token_type: &str,
) -> Result<(), error::Error> {
  let now = now()?;
  let expdt = now - token_expiration_ms;

  struct PurgeToken(i64, String, i64, Option<String>);

  let mut stmt = conn.prepare(
    "select user, token, tokendate, prevtoken from
      orgauth_token where tokendate < ?1 and type = ?2",
  )?;

  let c_iter = stmt.query_map(params![expdt, token_type], |row| {
    Ok(PurgeToken(
      row.get(0)?,
      row.get(1)?,
      row.get(2)?,
      row.get(3)?,
    ))
  })?;

  for item in c_iter {
    match item {
      Ok(PurgeToken(user, token, _tokendate, _prevtoken)) => {
        info!("purging login token for user {}, type {}", user, token_type);
        conn.execute(
          "delete from orgauth_token where
          user = ?1 and token = ?2",
          params![user, token],
        )?;
      }
      Err(e) => error!("error purging token: {:?}", e),
    }
  }

  Ok(())
}

pub fn purge_email_tokens(conn: &Connection, token_expiration_ms: i64) -> Result<(), error::Error> {
  let now = now()?;
  let expdt = now - token_expiration_ms;

  let count: i64 = conn.query_row(
    "select count(*) from
      orgauth_newemail where tokendate < ?1",
    params![expdt],
    |row| Ok(row.get(0)?),
  )?;

  if count > 0 {
    info!("removing {} expired orgauth_newemail records", count);

    conn.execute(
      "delete from orgauth_newemail
        where tokendate < ?1",
      params![expdt],
    )?;
  }

  Ok(())
}

pub fn purge_reset_tokens(conn: &Connection, token_expiration_ms: i64) -> Result<(), error::Error> {
  let now = now()?;
  let expdt = now - token_expiration_ms;

  let count: i64 = conn.query_row(
    "select count(*) from
      orgauth_newpassword where tokendate < ?1",
    params![expdt],
    |row| Ok(row.get(0)?),
  )?;

  if count > 0 {
    info!("removing {} expired orgauth_newpassword records", count);

    conn.execute(
      "delete from orgauth_newpassword
        where tokendate < ?1",
      params![expdt],
    )?;
  }

  Ok(())
}

pub fn purge_user_invites(conn: &Connection, token_expiration_ms: i64) -> Result<(), error::Error> {
  let now = now()?;
  let expdt = now - token_expiration_ms;

  let count: i64 = conn.query_row(
    "select count(*) from
      orgauth_user_invite where tokendate < ?1",
    params![expdt],
    |row| Ok(row.get(0)?),
  )?;

  if count > 0 {
    info!("removing {} expired orgauth_user_invite records", count);

    conn.execute(
      "delete from orgauth_user_invite
        where tokendate < ?1",
      params![expdt],
    )?;
  }

  Ok(())
}

pub fn purge_tokens(config: &Config) -> Result<(), error::Error> {
  let conn = connection_open(config.db.as_path())?;

  if let Some(expms) = config.login_token_expiration_ms {
    purge_login_tokens(&conn, expms)?;
  }

  purge_email_tokens(&conn, config.email_token_expiration_ms)?;

  purge_reset_tokens(&conn, config.reset_token_expiration_ms)?;

  purge_user_invites(&conn, config.invite_token_expiration_ms)?;
  Ok(())
}

pub fn update_user(conn: &Connection, user: &User) -> Result<(), error::Error> {
  conn.execute(
    "update orgauth_user set
       name = ?1,
       hashwd = ?2,
       salt = ?3,
       email = ?4,
       registration_key = ?5,
       admin = ?6,
       active = ?7,
       remote_url = ?8,
       cookie = ?9
     where id = ?10",
    params![
      user.name.to_lowercase(),
      user.hashwd,
      user.salt,
      user.email,
      user.registration_key,
      user.admin,
      user.active,
      user.remote_url,
      user.cookie,
      user.id.to_i64(),
    ],
  )?;

  Ok(())
}

// email change request.
pub fn add_newemail(
  conn: &Connection,
  user: UserId,
  token: Uuid,
  email: String,
) -> Result<(), error::Error> {
  let now = now()?;
  conn.execute(
    "insert into orgauth_newemail (user, email, token, tokendate)
     values (?1, ?2, ?3, ?4)",
    params![user.to_i64(), email, token.to_string(), now],
  )?;

  Ok(())
}

// email change request.
pub fn read_newemail(
  conn: &Connection,
  user: UserId,
  token: Uuid,
) -> Result<(String, i64), error::Error> {
  let result = conn.query_row(
    "select email, tokendate from orgauth_newemail
     where user = ?1
      and token = ?2",
    params![user.to_i64(), token.to_string()],
    |row| Ok((row.get(0)?, row.get(1)?)),
  )?;
  Ok(result)
}

// email change request.
pub fn remove_newemail(conn: &Connection, user: UserId, token: Uuid) -> Result<(), error::Error> {
  conn.execute(
    "delete from orgauth_newemail
     where user = ?1 and token = ?2",
    params![user.to_i64(), token.to_string()],
  )?;

  Ok(())
}

// password reset request.
pub fn add_newpassword(conn: &Connection, user: UserId, token: Uuid) -> Result<(), error::Error> {
  let now = now()?;
  conn.execute(
    "insert into orgauth_newpassword (user, token, tokendate)
     values (?1, ?2, ?3)",
    params![user.to_i64(), token.to_string(), now],
  )?;

  Ok(())
}

// password reset request.
pub fn read_newpassword(conn: &Connection, user: UserId, token: Uuid) -> Result<i64, error::Error> {
  let result = conn.query_row(
    "select tokendate from orgauth_newpassword
     where user = ?1
      and token = ?2",
    params![user.to_i64(), token.to_string()],
    |row| Ok(row.get(0)?),
  )?;
  Ok(result)
}

// password reset request.
pub fn remove_newpassword(
  conn: &Connection,
  user: UserId,
  token: Uuid,
) -> Result<(), error::Error> {
  conn.execute(
    "delete from orgauth_newpassword
     where user = ?1 and token = ?2",
    params![user.to_i64(), token.to_string()],
  )?;

  Ok(())
}

// email change request.
pub fn add_userinvite(
  conn: &Connection,
  token: Uuid,
  email: Option<String>,
  creator: UserId,
  data: Option<String>,
) -> Result<(), error::Error> {
  let now = now()?;
  conn.execute(
    "insert into orgauth_user_invite (email, token, tokendate, creator, data)
     values (?1, ?2, ?3, ?4, ?5)",
    params![email, token.to_string(), now, creator.to_i64(), data],
  )?;

  Ok(())
}

// email change request.
pub fn remove_userinvite(conn: &Connection, token: &str) -> Result<(), error::Error> {
  conn.execute(
    "delete from orgauth_user_invite
     where token = ?1",
    params![token],
  )?;

  Ok(())
}

// email change request.
pub fn read_userinvite(
  conn: &Connection,
  mainsite: &str,
  token: &str,
) -> Result<Option<UserInvite>, error::Error> {
  match conn.query_row(
    "select email, tokendate, data, creator from orgauth_user_invite
     where token = ?1",
    params![token],
    |row| {
      Ok(UserInvite {
        email: row.get(0)?,
        token: token.to_string(),
        // tokendate: row.get(1)?,
        url: format!("{}/invite/{}", mainsite, token),
        data: row.get(2)?,
        creator: UserId::Uid(row.get(3)?),
      })
    },
  ) {
    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
    Ok(v) => Ok(Some(v)),
    Err(e) => Err(e.into()),
  }
}

// change password, checking old password first.
pub fn change_password(
  conn: &Connection,
  uid: UserId,
  cp: &ChangePassword,
) -> Result<(), error::Error> {
  let mut userdata = read_user_by_id(&conn, uid)?;
  match userdata.registration_key {
    Some(_reg_key) => bail!("invalid user or password"),
    None => {
      if sha256::digest(
        (cp.oldpwd.clone() + userdata.salt.as_str())
          .into_bytes()
          .as_slice(),
      ) != userdata.hashwd
      {
        // old password is bad, can't change.
        bail!("invalid password!")
      } else {
        let newhash = sha256::digest(
          (cp.newpwd.clone() + userdata.salt.as_str())
            .into_bytes()
            .as_slice(),
        );
        userdata.hashwd = newhash;
        update_user(&conn, &userdata)?;
        info!("changed password for {}", userdata.name.to_lowercase());

        Ok(())
      }
    }
  }
}

// change password, checking old password first.
pub async fn change_remote_url(
  conn: &Connection,
  uid: UserId,
  user_uri_path: String,
  cru: &ChangeRemoteUrl,
) -> Result<UserResponse, error::Error> {
  let mut userdata = read_user_by_id(&conn, uid)?;
  {
    // check the pwd.
    if sha256::digest(
      (cru.pwd.clone() + userdata.salt.as_str())
        .into_bytes()
        .as_slice(),
    ) != userdata.hashwd
    {
      // old password is bad, can't change.
      bail!("invalid password!")
    } else {
      // try to log in to an existing account on the remote!
      let client = reqwest::Client::new();
      let l = UserRequest::UrqLogin(Login {
        uid: userdata.name.clone(),
        pwd: cru.pwd.clone(),
      });

      // TODO: this uri is dependent on the remote app!
      // which should be the same as this app, but still.
      let user_uri = format!("{}/{}", cru.remote_url, user_uri_path);

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
        if userdata.uuid != ld.uuid {
          return Ok(UserResponse::UrpInvalidUserUuid);
        }

        userdata.remote_url = Some(cru.remote_url.clone());
        userdata.cookie = cookie;

        update_user(conn, &userdata)?;
        info!("changed remote_url for {}", userdata.name.to_lowercase());
        Ok(UserResponse::UrpChangedRemoteUrl(cru.remote_url.clone()))
      } else {
        Ok(UserResponse::UrpRemoteRegistrationFailed)
      }
    }
  }
}

// change password without requiring old password.
// for unregistered users.
pub fn override_password(
  conn: &Connection,
  uid: UserId,
  newpwd: String,
) -> Result<(), error::Error> {
  let mut userdata = read_user_by_id(&conn, uid)?;
  // just being cautious in limiting this to only unregistered.
  match userdata.registration_key {
    Some(ref _reg_key) => {
      let newhash = sha256::digest(
        (newpwd.clone() + userdata.salt.as_str())
          .into_bytes()
          .as_slice(),
      );
      userdata.hashwd = newhash;
      update_user(&conn, &userdata)?;
      info!("changed password for {}", userdata.name.to_lowercase());

      Ok(())
    }
    None => {
      bail!("registered user; can't override password.")
    }
  }
}

pub fn change_email(
  conn: &Connection,
  uid: UserId,
  cp: ChangeEmail,
) -> Result<(String, Uuid), error::Error> {
  let userdata = read_user_by_id(&conn, uid)?;
  match userdata.registration_key {
    Some(_reg_key) => bail!("invalid user or password"),
    None => {
      if sha256::digest(
        (cp.pwd.clone() + userdata.salt.as_str())
          .into_bytes()
          .as_slice(),
      ) != userdata.hashwd
      {
        // bad password, can't change.
        bail!("invalid password!")
      } else {
        // create a 'newemail' record.
        let token = Uuid::new_v4();
        add_newemail(&conn, uid, token, cp.email)?;

        Ok((userdata.name.to_lowercase(), token))
      }
    }
  }
}

pub fn delete_user(conn: &Connection, uid: UserId) -> Result<(), error::Error> {
  info!("deleting user: {}", uid);
  conn.execute(
    "delete from orgauth_token where user = ?1",
    params!(uid.to_i64()),
  )?;
  conn.execute(
    "delete from orgauth_newemail where user = ?1",
    params!(uid.to_i64()),
  )?;
  conn.execute(
    "delete from orgauth_newpassword where user = ?1",
    params!(uid.to_i64()),
  )?;
  conn.execute(
    "delete from orgauth_user where id = ?1",
    params!(uid.to_i64()),
  )?;

  Ok(())
}
