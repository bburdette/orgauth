use crate::data::{ChangeEmail, ChangePassword, LoginData, User, UserInvite};
use crate::data::{Config, RegistrationData};
use crate::util::{is_token_expired, now, salt_string};
use actix_session::Session;
use crypto_hash::{hex_digest, Algorithm};
use log::{error, info};
use rusqlite::{params, Connection};
use simple_error::bail;
use std::error::Error;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

pub fn connection_open(dbfile: &Path) -> Result<Connection, Box<dyn Error>> {
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
  creator: Option<i64>,
  on_new_user: &mut Box<
    dyn FnMut(
      &Connection,
      &RegistrationData,
      Option<String>,
      Option<i64>,
      i64,
    ) -> Result<(), Box<dyn Error>>,
  >,
) -> Result<i64, Box<dyn Error>> {
  let now = now()?;
  let salt = salt_string();
  let hashwd = hex_digest(
    Algorithm::SHA256,
    (rd.pwd.clone() + salt.as_str()).into_bytes().as_slice(),
  );

  // make a user record.
  conn.execute(
    "insert into orgauth_user (name, hashwd, salt, email, admin, active, registration_key, createdate)
      values (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)",
    params![rd.uid.to_lowercase(), hashwd, salt, rd.email, admin, registration_key, now],
  )?;

  let uid = conn.last_insert_rowid();

  (on_new_user)(&conn, &rd, data, creator, uid)?;

  Ok(uid)
}

pub fn user_id(conn: &Connection, name: &str) -> Result<i64, Box<dyn Error>> {
  let id: i64 = conn.query_row(
    "select id from orgauth_user
      where orgauth_user.name = ?1",
    params![name.to_lowercase()],
    |row| Ok(row.get(0)?),
  )?;
  Ok(id)
}

pub fn login_data(conn: &Connection, uid: i64) -> Result<LoginData, Box<dyn Error>> {
  let user = read_user_by_id(&conn, uid)?;
  Ok(LoginData {
    userid: uid,
    name: user.name,
    email: user.email,
    admin: user.admin,
    active: user.active,
    data: None,
  })
}

pub fn login_data_cb(
  conn: &Connection,
  uid: i64,
  extra_login_data: &mut Box<
    dyn FnMut(&Connection, i64) -> Result<Option<serde_json::Value>, Box<dyn Error>>,
  >,
) -> Result<LoginData, Box<dyn Error>> {
  let user = read_user_by_id(&conn, uid)?;
  Ok(LoginData {
    userid: uid,
    name: user.name,
    email: user.email,
    admin: user.admin,
    active: user.active,
    data: extra_login_data(&conn, uid)?,
  })
}

pub fn update_login_data(conn: &Connection, ld: &LoginData) -> Result<(), Box<dyn Error>> {
  let mut user = read_user_by_id(&conn, ld.userid)?;
  user.name = ld.name.to_lowercase();
  user.email = ld.email.clone();
  user.admin = ld.admin;
  user.active = ld.active;
  update_user(&conn, &user)
}

pub fn read_users(
  conn: &Connection,
  extra_login_data: &mut Box<
    dyn FnMut(&Connection, i64) -> Result<Option<serde_json::Value>, Box<dyn Error>>,
  >,
) -> Result<Vec<LoginData>, Box<dyn Error>> {
  let mut pstmt = conn.prepare(
    // return zklinks that link to or from notes that link to 'public'.
    "select id from orgauth_user",
  )?;

  let r = Ok(
    pstmt
      .query_map(params![], |row| {
        let id = row.get(0)?;
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

pub fn read_user_by_name(conn: &Connection, name: &str) -> Result<User, Box<dyn Error>> {
  let user = conn.query_row(
    "select id, hashwd, salt, email, registration_key, admin, active
      from orgauth_user where name = ?1",
    params![name.to_lowercase()],
    |row| {
      Ok(User {
        id: row.get(0)?,
        name: name.to_lowercase(),
        hashwd: row.get(1)?,
        salt: row.get(2)?,
        email: row.get(3)?,
        registration_key: row.get(4)?,
        admin: row.get(5)?,
        active: row.get(6)?,
      })
    },
  )?;

  Ok(user)
}

pub fn read_user_by_id(conn: &Connection, id: i64) -> Result<User, Box<dyn Error>> {
  let user = conn.query_row(
    "select id, name, hashwd, salt, email, registration_key, admin, active
      from orgauth_user where id = ?1",
    params![id],
    |row| {
      Ok(User {
        id: row.get(0)?,
        name: row.get(1)?,
        hashwd: row.get(2)?,
        salt: row.get(3)?,
        email: row.get(4)?,
        registration_key: row.get(5)?,
        admin: row.get(6)?,
        active: row.get(7)?,
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

fn read_user_by_token(conn: &Connection, token: Uuid) -> Result<(User, TokenInfo), Box<dyn Error>> {
  let (user, tokendate, regendate, prevtoken) : (User, i64, Option<i64>, Option<String>) = conn.query_row(
    "select id, name, hashwd, salt, email, registration_key, admin, active, 
        orgauth_token.tokendate, orgauth_token.regendate, orgauth_token.prevtoken
      from orgauth_user, orgauth_token where orgauth_user.id = orgauth_token.user and orgauth_token.token = ?1",
    params![token.to_string()],
    |row| {
      Ok((
        User {
          id: row.get(0)?,
          name: row.get(1)?,
          hashwd: row.get(2)?,
          salt: row.get(3)?,
          email: row.get(4)?,
          registration_key: row.get(5)?,
          admin: row.get(6)?,
          active: row.get(7)?,
        },
        row.get(8)?,
        row.get(9)?,
        row.get(10)?,
      ))
    },
  )?;

  Ok((
    user,
    TokenInfo {
      tokendate: tokendate,
      regendate: regendate,
      prevtoken: prevtoken,
    },
  ))
}

fn check_user(
  user: &User,
  token: Uuid,
  tokendate: i64,
  token_expiration_ms: Option<i64>,
) -> Result<(), Box<dyn Error>> {
  if !user.active {
    bail!("account is inactive")
  } else {
    if let Some(texp) = token_expiration_ms {
      if is_token_expired(texp, tokendate) {
        bail!("login expired")
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
) -> Result<User, Box<dyn Error>> {
  let (user, tokeninfo) = read_user_by_token(&conn, token)?;

  check_user(&user, token, tokeninfo.tokendate, token_expiration_ms)?;

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
        let wat = conn.execute(
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
) -> Result<(), Box<dyn Error>> {
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
) -> Result<User, Box<dyn Error>> {
  let tx = conn.transaction()?;

  let (user, tokeninfo) = read_user_by_token(&tx, token)?;

  check_user(&user, token, tokeninfo.tokendate, token_expiration_ms)?;

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
      add_token(&tx, user.id, new_token, Some(token))?;
      session.set("token", new_token)?;
    }
  }

  tx.commit()?;

  Ok(user)
}

pub fn add_token(
  conn: &Connection,
  user: i64,
  token: Uuid,
  prevtoken: Option<Uuid>,
) -> Result<(), Box<dyn Error>> {
  let now = now()?;
  conn.execute(
    "insert into orgauth_token (user, token, tokendate, prevtoken)
     values (?1, ?2, ?3, ?4)",
    params![
      user,
      token.to_string(),
      now,
      prevtoken.map(|s| s.to_string())
    ],
  )?;

  Ok(())
}

pub fn mark_prevtoken(
  conn: &Connection,
  // token: Uuid,
  prevtoken: Uuid,
) -> Result<bool, Box<dyn Error>> {
  // set regendate to now.
  let now = now()?;
  let wat = conn.execute(
    "update orgauth_token set regendate = ?1 where token = ?2",
    params![now, prevtoken.to_string()],
  )?;

  match wat {
    1 => Ok(true),
    0 => Ok(false), // could mean token doesn't exist, or regendate expired.
    x => bail!("too many records updated: {}", x),
  }
}

pub fn purge_login_tokens(
  conn: &Connection,
  token_expiration_ms: i64,
) -> Result<(), Box<dyn Error>> {
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
      Ok(PurgeToken(user, token, tokendate, prevtoken)) => {
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

pub fn purge_email_tokens(
  conn: &Connection,
  token_expiration_ms: i64,
) -> Result<(), Box<dyn Error>> {
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

pub fn purge_reset_tokens(
  conn: &Connection,
  token_expiration_ms: i64,
) -> Result<(), Box<dyn Error>> {
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

pub fn purge_user_invites(
  conn: &Connection,
  token_expiration_ms: i64,
) -> Result<(), Box<dyn Error>> {
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

pub fn purge_tokens(config: &Config) -> Result<(), Box<dyn Error>> {
  let conn = connection_open(config.db.as_path())?;

  if let Some(expms) = config.login_token_expiration_ms {
    purge_login_tokens(&conn, expms)?;
  }

  purge_email_tokens(&conn, config.email_token_expiration_ms)?;

  purge_reset_tokens(&conn, config.reset_token_expiration_ms)?;

  purge_user_invites(&conn, config.invite_token_expiration_ms)?;
  Ok(())
}

pub fn update_user(conn: &Connection, user: &User) -> Result<(), Box<dyn Error>> {
  conn.execute(
    "update orgauth_user set name = ?1, hashwd = ?2, salt = ?3, email = ?4, registration_key = ?5, admin = ?6, active = ?7
           where id = ?8",
    params![
      user.name.to_lowercase(),
      user.hashwd,
      user.salt,
      user.email,
      user.registration_key,
      user.admin,
      user.active,
      user.id,
    ],
  )?;

  Ok(())
}

// email change request.
pub fn add_newemail(
  conn: &Connection,
  user: i64,
  token: Uuid,
  email: String,
) -> Result<(), Box<dyn Error>> {
  let now = now()?;
  conn.execute(
    "insert into orgauth_newemail (user, email, token, tokendate)
     values (?1, ?2, ?3, ?4)",
    params![user, email, token.to_string(), now],
  )?;

  Ok(())
}

// email change request.
pub fn read_newemail(
  conn: &Connection,
  user: i64,
  token: Uuid,
) -> Result<(String, i64), Box<dyn Error>> {
  let result = conn.query_row(
    "select email, tokendate from orgauth_newemail
     where user = ?1
      and token = ?2",
    params![user, token.to_string()],
    |row| Ok((row.get(0)?, row.get(1)?)),
  )?;
  Ok(result)
}

// email change request.
pub fn remove_newemail(conn: &Connection, user: i64, token: Uuid) -> Result<(), Box<dyn Error>> {
  conn.execute(
    "delete from orgauth_newemail
     where user = ?1 and token = ?2",
    params![user, token.to_string()],
  )?;

  Ok(())
}

// password reset request.
pub fn add_newpassword(conn: &Connection, user: i64, token: Uuid) -> Result<(), Box<dyn Error>> {
  let now = now()?;
  conn.execute(
    "insert into orgauth_newpassword (user, token, tokendate)
     values (?1, ?2, ?3)",
    params![user, token.to_string(), now],
  )?;

  Ok(())
}

// password reset request.
pub fn read_newpassword(conn: &Connection, user: i64, token: Uuid) -> Result<i64, Box<dyn Error>> {
  let result = conn.query_row(
    "select tokendate from orgauth_newpassword
     where user = ?1
      and token = ?2",
    params![user, token.to_string()],
    |row| Ok(row.get(0)?),
  )?;
  Ok(result)
}

// password reset request.
pub fn remove_newpassword(conn: &Connection, user: i64, token: Uuid) -> Result<(), Box<dyn Error>> {
  conn.execute(
    "delete from orgauth_newpassword
     where user = ?1 and token = ?2",
    params![user, token.to_string()],
  )?;

  Ok(())
}

// email change request.
pub fn add_userinvite(
  conn: &Connection,
  token: Uuid,
  email: Option<String>,
  creator: i64,
  data: Option<String>,
) -> Result<(), Box<dyn Error>> {
  let now = now()?;
  conn.execute(
    "insert into orgauth_user_invite (email, token, tokendate, creator, data)
     values (?1, ?2, ?3, ?4, ?5)",
    params![email, token.to_string(), now, creator, data],
  )?;

  Ok(())
}

// email change request.
pub fn remove_userinvite(conn: &Connection, token: &str) -> Result<(), Box<dyn Error>> {
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
) -> Result<Option<UserInvite>, Box<dyn Error>> {
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
        creator: row.get(3)?,
      })
    },
  ) {
    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
    Ok(v) => Ok(Some(v)),
    Err(e) => Err(Box::new(e)),
  }
}

// change password, checking old password first.
pub fn change_password(
  conn: &Connection,
  uid: i64,
  cp: ChangePassword,
) -> Result<(), Box<dyn Error>> {
  let mut userdata = read_user_by_id(&conn, uid)?;
  match userdata.registration_key {
    Some(_reg_key) => bail!("invalid user or password"),
    None => {
      if hex_digest(
        Algorithm::SHA256,
        (cp.oldpwd.clone() + userdata.salt.as_str())
          .into_bytes()
          .as_slice(),
      ) != userdata.hashwd
      {
        // old password is bad, can't change.
        bail!("invalid password!")
      } else {
        let newhash = hex_digest(
          Algorithm::SHA256,
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

// change password without requiring old password.
// for unregistered users.
pub fn override_password(
  conn: &Connection,
  uid: i64,
  newpwd: String,
) -> Result<(), Box<dyn Error>> {
  let mut userdata = read_user_by_id(&conn, uid)?;
  // just being cautious in limiting this to only unregistered.
  match userdata.registration_key {
    Some(ref _reg_key) => {
      let newhash = hex_digest(
        Algorithm::SHA256,
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
  uid: i64,
  cp: ChangeEmail,
) -> Result<(String, Uuid), Box<dyn Error>> {
  let userdata = read_user_by_id(&conn, uid)?;
  match userdata.registration_key {
    Some(_reg_key) => bail!("invalid user or password"),
    None => {
      if hex_digest(
        Algorithm::SHA256,
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

pub fn delete_user(conn: &Connection, uid: i64) -> Result<(), Box<dyn Error>> {
  info!("deleting user: {}", uid);
  conn.execute("delete from orgauth_token where user = ?1", params!(uid))?;
  conn.execute("delete from orgauth_newemail where user = ?1", params!(uid))?;
  conn.execute(
    "delete from orgauth_newpassword where user = ?1",
    params!(uid),
  )?;
  conn.execute("delete from orgauth_user where id = ?1", params!(uid))?;

  Ok(())
}
