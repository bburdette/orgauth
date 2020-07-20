use rusqlite::{params, Connection};
use serde_json;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::error::Error;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Debug, Clone)]
pub struct PdfInfo {
  pub last_read: Option<i64>,
  pub filename: String,
  pub state: Option<serde_json::Value>,
}

#[derive(Serialize, Debug, Clone)]
pub struct FullBlogEntry {
  id: i64,
  title: String,
  content: String,
  user: i64,
  createdate: i64,
  changeddate: i64,
}

#[derive(Serialize, Debug, Clone)]
pub struct BlogListEntry {
  id: i64,
  title: String,
  user: i64,
  createdate: i64,
  changeddate: i64,
}

#[derive(Serialize, Debug, Clone)]
pub struct UpdateBlogEntry {
  id: i64,
  title: String,
  content: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct NewBlogEntry {
  title: String,
  content: String,
  user: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
  pub id: i64,
  pub name: String,
  pub hashwd: String,
  pub salt: String,
  pub email: String,
  pub registration_key: Option<String>,
  // current_tb: Option<i32>,
}

pub fn dbinit(dbfile: &Path) -> rusqlite::Result<()> {
  let conn = Connection::open(dbfile)?;

  println!("pre user");
  // create the pdfinfo table.
  conn.execute(
    "CREATE TABLE user (
                id          INTEGER NOT NULL PRIMARY KEY,
                name        TEXT NOT NULL UNIQUE,
                hashwd      TEXT NOT NULL,
                salt        TEXT NOT NULL,
                email       TEXT NOT NULL,
                registration_key  TEXT,
                createdate  INTEGER NOT NULL
                )",
    params![],
  )?;

  println!("pre tag");
  conn.execute(
    "CREATE TABLE tag (
                id          INTEGER NOT NULL PRIMARY KEY,
                name        TEXT NOT NULL UNIQUE,
                user				INTEGER NOT NULL,
                FOREIGN KEY(user) REFERENCES user(id)
                )",
    params![],
  )?;

  println!("pre be");
  conn.execute(
    "CREATE TABLE blogentry (
                id          	INTEGER NOT NULL PRIMARY KEY,
                title					TEXT NOT NULL,
                content 			TEXT NOT NULL,
                user 					INTEGER NOT NULL,
                createdate 		INTEGER NOT NULL,
                changeddate 	INTEGER NOT NULL,
                FOREIGN KEY(user) REFERENCES user(id)
                )",
    params![],
  )?;

  println!("pre bt");
  conn.execute(
    "CREATE TABLE blogtag (
                tagid     		   INTEGER NOT NULL,
                blogentryid      INTEGER NOT NULL,
                FOREIGN KEY(tagid) REFERENCES tag(id),
                FOREIGN KEY(blogentryid) REFERENCES blogentry(id),
                CONSTRAINT unq UNIQUE (tagid, blogentryid)
                )",
    params![],
  )?;

  Ok(())
}

pub fn naiow() -> Result<i64, Box<dyn Error>> {
  let nowsecs = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .map(|n| n.as_secs())?;
  let s: i64 = nowsecs.try_into()?;
  Ok(s * 1000)
}

pub fn add_user(dbfile: &Path, name: &str, hashwd: &str) -> Result<i64, Box<dyn Error>> {
  let conn = Connection::open(dbfile)?;

  let nowi64secs = naiow()?;

  println!("adding user: {}", name);
  let wat = conn.execute(
    "INSERT INTO user (name, hashwd, createdate)
                VALUES (?1, ?2, ?3)",
    params![name, hashwd, nowi64secs],
  )?;

  println!("wat: {}", wat);

  Ok(conn.last_insert_rowid())
}

pub fn read_user(dbfile: &Path, name: &str) -> Result<User, Box<dyn Error>> {
  let conn = Connection::open(dbfile)?;

  let user = conn.query_row(
    "SELECT id, name, hashwd, salt, email, registration_key, createdate
      from user WHERE name = ?1",
    params![name],
    |row| {
      Ok(User {
        id: row.get(0)?,
        name: name.to_string(),
        hashwd: row.get(1)?,
        salt: row.get(2)?,
        email: row.get(3)?,
        registration_key: row.get(4)?,
      })
    },
  )?;

  Ok(user)
}

pub fn new_user(
  dbfile: &Path,
  name: String,
  hashwd: String,
  salt: String,
  email: String,
  registration_key: String,
) -> Result<i64, Box<dyn Error>> {
  let conn = Connection::open(dbfile)?;

  let now = naiow()?;

  let user = conn.execute(
    "INSERT INTO user  (name, hashwd, salt, email, registration_key, createdate)
      from user VALUES (?2, ?3, ?4, ?5, ?6)",
    params![name, hashwd, salt, email, registration_key, now],
  )?;

  Ok(conn.last_insert_rowid())
}

pub fn add_tag(dbfile: &Path, name: &str, user: i64) -> Result<i64, Box<dyn Error>> {
  let conn = Connection::open(dbfile)?;

  println!("adding tag: {}", name);
  conn.execute(
    "INSERT INTO tag (name, user)
                VALUES (?1, ?2)",
    params![name, user],
  )?;

  Ok(conn.last_insert_rowid())
}

pub fn new_blogentry(dbfile: &Path, entry: &NewBlogEntry) -> Result<i64, Box<dyn Error>> {
  let conn = Connection::open(dbfile)?;

  let now = naiow()?;

  println!("adding blogentry: {}", entry.title);
  conn.execute(
    "INSERT INTO blogentry (title, content, user, createdate, changeddate)
                VALUES (?1, ?2, ?3, ?4, ?5)",
    params![entry.title, entry.content, entry.user, now, now],
  )?;

  Ok(conn.last_insert_rowid())
}

pub fn update_blogentry(dbfile: &Path, entry: &UpdateBlogEntry) -> Result<(), Box<dyn Error>> {
  let conn = Connection::open(dbfile)?;

  let now = naiow()?;

  println!("adding blogentry: {}", entry.title);
  conn.execute(
    "UPDATE blogentry SET title = ?1, content = ?2, changeddate = ?3
     WHERE id = ?4",
    params![entry.title, entry.content, now, entry.id],
  )?;

  Ok(())
}

pub fn read_blogentry(dbfile: &Path, id: i64) -> Result<FullBlogEntry, Box<dyn Error>> {
  let conn = Connection::open(dbfile)?;

  let rbe = conn.query_row(
    "SELECT title, content, user, createdate, changeddate
      from blogentry WHERE id = ?1",
    params![id],
    |row| {
      Ok(FullBlogEntry {
        id: id,
        title: row.get(0)?,
        content: row.get(1)?,
        user: row.get(2)?,
        createdate: row.get(3)?,
        changeddate: row.get(4)?,
      })
    },
  )?;

  Ok(rbe)
}

pub fn bloglisting(dbfile: &Path, user: i64) -> rusqlite::Result<Vec<BlogListEntry>> {
  let conn = Connection::open(dbfile)?;

  let mut pstmt = conn.prepare(
    "SELECT id, title, createdate, changeddate
      from blogentry where user = ?1",
  )?;

  let pdfinfo_iter = pstmt.query_map(params![user], |row| {
    Ok(BlogListEntry {
      id: row.get(0)?,
      title: row.get(1)?,
      user: row.get(2)?,
      createdate: row.get(3)?,
      changeddate: row.get(4)?,
    })
  })?;

  let mut pv = Vec::new();

  for rspdfinfo in pdfinfo_iter {
    match rspdfinfo {
      Ok(pdfinfo) => {
        pv.push(pdfinfo);
      }
      Err(_) => (),
    }
  }

  Ok(pv)
}
