use crate::error;
use barrel::backend::Sqlite;
use barrel::{types, Migration};
use rusqlite::{params, Connection};
use std::path::Path;

pub fn udpate1(dbfile: &Path) -> Result<(), error::Error> {
  // db connection without foreign key checking.
  let conn = Connection::open(dbfile)?;
  let mut m = Migration::new();

  // user table
  m.create_table("orgauth_user", |t| {
    t.add_column(
      "id",
      types::integer()
        .primary(true)
        .increments(true)
        .nullable(false),
    );
    t.add_column("name", types::text().nullable(false).unique(true));
    t.add_column("hashwd", types::text().nullable(false));
    t.add_column("salt", types::text().nullable(false));
    t.add_column("email", types::text().nullable(false));
    t.add_column("registration_key", types::text().nullable(true));
    t.add_column("createdate", types::integer().nullable(false));
  });

  // add token table.  multiple tokens per user to support multiple browsers and/or devices.
  m.create_table("orgauth_token", |t| {
    t.add_column(
      "user",
      types::foreign(
        "orgauth_user",
        "id",
        types::ReferentialAction::Restrict,
        types::ReferentialAction::Restrict,
      )
      .nullable(false),
    );
    t.add_column("token", types::text().nullable(false));
    t.add_column("tokendate", types::integer().nullable(false));
    t.add_index(
      "orgauth_tokenunq",
      types::index(vec!["user", "token"]).unique(true),
    );
  });

  // add newemail table.  each request for a new email creates an entry.
  m.create_table("orgauth_newemail", |t| {
    t.add_column(
      "user",
      types::foreign(
        "orgauth_user",
        "id",
        types::ReferentialAction::Restrict,
        types::ReferentialAction::Restrict,
      )
      .nullable(false),
    );
    t.add_column("email", types::text().nullable(false));
    t.add_column("token", types::text().nullable(false));
    t.add_column("tokendate", types::integer().nullable(false));
    t.add_index(
      "orgauth_newemailunq",
      types::index(vec!["user", "token"]).unique(true),
    );
  });

  // add newpassword table.  each request for a new password creates an entry.
  m.create_table("orgauth_newpassword", |t| {
    t.add_column(
      "user",
      types::foreign(
        "orgauth_user",
        "id",
        types::ReferentialAction::Restrict,
        types::ReferentialAction::Restrict,
      )
      .nullable(false),
    );
    t.add_column("token", types::text().nullable(false));
    t.add_column("tokendate", types::integer().nullable(false));
    t.add_index(
      "orgauth_resetpasswordunq",
      types::index(vec!["user", "token"]).unique(true),
    );
  });

  conn.execute_batch(m.make::<Sqlite>().as_str())?;

  Ok(())
}

pub fn udpate2(dbfile: &Path) -> Result<(), error::Error> {
  // db connection without foreign key checking.
  let conn = Connection::open(dbfile)?;
  conn.execute("PRAGMA foreign_keys = false;", params![])?;
  let mut m1 = Migration::new();

  // temp table for user data.
  m1.create_table("orgauth_user_temp", |t| {
    t.add_column(
      "id",
      types::integer()
        .primary(true)
        .increments(true)
        .nullable(false),
    );
    t.add_column("name", types::text().nullable(false).unique(true));
    t.add_column("hashwd", types::text().nullable(false));
    t.add_column("salt", types::text().nullable(false));
    t.add_column("email", types::text().nullable(false));
    t.add_column("registration_key", types::text().nullable(true));
    t.add_column("createdate", types::integer().nullable(false));
  });

  conn.execute_batch(m1.make::<Sqlite>().as_str())?;

  // copy everything from temp.
  conn.execute(
    "insert into orgauth_user_temp (id, name, hashwd, salt, email, registration_key, createdate)
        select id, name, hashwd, salt, email, registration_key, createdate from orgauth_user",
    params![],
  )?;

  let mut m2 = Migration::new();
  m2.drop_table("orgauth_user");

  m2.create_table("orgauth_user", |t| {
    t.add_column(
      "id",
      types::integer()
        .primary(true)
        .increments(true)
        .nullable(false),
    );
    t.add_column("name", types::text().nullable(false).unique(true));
    t.add_column("hashwd", types::text().nullable(false));
    t.add_column("salt", types::text().nullable(false));
    t.add_column("email", types::text().nullable(false));
    t.add_column("registration_key", types::text().nullable(true));
    t.add_column("admin", types::boolean().nullable(false));
    t.add_column("active", types::boolean().nullable(false));
    t.add_column("createdate", types::integer().nullable(false));
  });

  conn.execute_batch(m2.make::<Sqlite>().as_str())?;

  conn.execute(
    "insert into orgauth_user (id, name, hashwd, salt, email, registration_key, admin, active, createdate)
        select id, name, hashwd, salt, email, registration_key, 0, 1, createdate from orgauth_user_temp",
    params![],
  )?;

  conn.execute("drop table orgauth_user_temp", params![])?;

  Ok(())
}

pub fn udpate3(dbfile: &Path) -> Result<(), error::Error> {
  // db connection without foreign key checking.
  let conn = Connection::open(dbfile)?;
  conn.execute("PRAGMA foreign_keys = false;", params![])?;
  let mut m = Migration::new();

  m.create_table("orgauth_user_invite", |t| {
    t.add_column("email", types::text().nullable(true));
    t.add_column("token", types::text().nullable(false));
    t.add_column("tokendate", types::integer().nullable(false));
    t.add_index(
      "orgauth_user_invite_unq",
      types::index(vec!["user", "token"]).unique(true),
    );
  });

  conn.execute_batch(m.make::<Sqlite>().as_str())?;

  Ok(())
}

pub fn udpate4(dbfile: &Path) -> Result<(), error::Error> {
  // db connection without foreign key checking.
  let conn = Connection::open(dbfile)?;
  conn.execute("PRAGMA foreign_keys = false;", params![])?;
  let mut m = Migration::new();

  // eaiser to drop the table.  data is lost though.
  m.drop_table("orgauth_user_invite");
  // add newemail table.  each request for a new email creates an entry.
  m.create_table("orgauth_user_invite", |t| {
    t.add_column("email", types::text().nullable(true));
    t.add_column("token", types::text().nullable(false));
    t.add_column("tokendate", types::integer().nullable(false));
    t.add_column(
      "creator",
      types::foreign(
        "orgauth_user",
        "id",
        types::ReferentialAction::Restrict,
        types::ReferentialAction::Restrict,
      ),
    );
    t.add_column("data", types::text().nullable(true));
    t.add_index(
      "orgauth_user_invite_unq",
      types::index(vec!["user", "token"]).unique(true),
    );
  });

  conn.execute_batch(m.make::<Sqlite>().as_str())?;

  Ok(())
}

pub fn udpate5(dbfile: &Path) -> Result<(), error::Error> {
  // db connection without foreign key checking.
  let conn = Connection::open(dbfile)?;
  conn.execute("PRAGMA foreign_keys = false;", params![])?;

  conn.execute("update orgauth_user set name = lower(name)", params![])?;

  Ok(())
}

pub fn udpate6(dbfile: &Path) -> Result<(), error::Error> {
  // db connection without foreign key checking.
  let conn = Connection::open(dbfile)?;
  conn.execute("PRAGMA foreign_keys = false;", params![])?;

  let mut m = Migration::new();

  // add token table.  multiple tokens per user to support multiple browsers and/or devices.
  m.change_table("orgauth_token", |t| {
    t.add_column("regendate", types::integer().nullable(true));
  });

  conn.execute_batch(m.make::<Sqlite>().as_str())?;

  Ok(())
}

pub fn udpate7(dbfile: &Path) -> Result<(), error::Error> {
  // db connection without foreign key checking.
  let conn = Connection::open(dbfile)?;
  conn.execute("PRAGMA foreign_keys = false;", params![])?;

  let mut m = Migration::new();

  // add token table.  multiple tokens per user to support multiple browsers and/or devices.
  m.change_table("orgauth_token", |t| {
    t.add_column("prevtoken", types::text().nullable(true));
  });

  conn.execute_batch(m.make::<Sqlite>().as_str())?;

  Ok(())
}

pub fn udpate8(dbfile: &Path) -> Result<(), error::Error> {
  // db connection without foreign key checking.
  let conn = Connection::open(dbfile)?;
  conn.execute("PRAGMA foreign_keys = false;", params![])?;
  let mut m1 = Migration::new();

  // temp table for user data.
  m1.create_table("orgauth_user_temp", |t| {
    t.add_column(
      "id",
      types::integer()
        .primary(true)
        .increments(true)
        .nullable(false),
    );
    t.add_column("name", types::text().nullable(false).unique(true));
    t.add_column("uuid", types::text().nullable(true).unique(true));
    t.add_column("hashwd", types::text().nullable(false));
    t.add_column("salt", types::text().nullable(false));
    t.add_column("email", types::text().nullable(false));
    t.add_column("registration_key", types::text().nullable(true));
    t.add_column("admin", types::boolean().nullable(false));
    t.add_column("active", types::boolean().nullable(false));
    t.add_column("createdate", types::integer().nullable(false));
  });

  conn.execute_batch(m1.make::<Sqlite>().as_str())?;

  // copy everything from temp.
  conn.execute(
    "insert into orgauth_user_temp (id, name, hashwd, salt, email, registration_key, admin, active, createdate)
        select id, name, hashwd, salt, email, registration_key, admin, active, createdate from orgauth_user",
    params![],
  )?;

  // now make uuids for all users.
  let mut pstmt = conn.prepare("select id from orgauth_user_temp")?;
  let ids: Vec<i64> = pstmt
    .query_map(params![], |row| Ok(row.get(0)?))?
    .filter_map(|x| x.ok())
    .collect();

  // this is horrifically slow
  for id in ids {
    let uuid = uuid::Uuid::new_v4();

    conn.execute(
      "update orgauth_user_temp set uuid = ?1 where id = ?2",
      params![uuid.to_string(), id],
    )?;

    println!("updated user {} {}", id, uuid);
  }

  let mut m2 = Migration::new();
  m2.drop_table("orgauth_user");

  m2.create_table("orgauth_user", |t| {
    t.add_column(
      "id",
      types::integer()
        .primary(true)
        .increments(true)
        .nullable(false),
    );
    t.add_column("name", types::text().nullable(false).unique(true));
    t.add_column("uuid", types::text().nullable(false).unique(true));
    t.add_column("hashwd", types::text().nullable(false));
    t.add_column("salt", types::text().nullable(false));
    t.add_column("email", types::text().nullable(false));
    t.add_column("registration_key", types::text().nullable(true));
    t.add_column("admin", types::boolean().nullable(false));
    t.add_column("active", types::boolean().nullable(false));
    t.add_column("remote_url", types::text().nullable(true));
    t.add_column("cookie", types::text().nullable(true));
    t.add_column("createdate", types::integer().nullable(false));
  });

  conn.execute_batch(m2.make::<Sqlite>().as_str())?;

  conn.execute(
    "insert into orgauth_user (id, name, uuid, hashwd, salt, email, registration_key, admin, active, createdate)
        select id, name, uuid, hashwd, salt, email, registration_key, admin, active, createdate from orgauth_user_temp",
    params![],
  )?;

  conn.execute("drop table orgauth_user_temp", params![])?;

  Ok(())
}
