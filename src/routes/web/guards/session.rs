use models::id::SessionId;

use cookie::{Cookie, SameSite};

use rocket::Outcome;
use rocket::request::{self, Request, FromRequest};

use serde_json;

use uuid::Uuid;

use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session<'a, 'r> where 'r: 'a {
  #[serde(skip)]
  pub request: Option<&'a Request<'r>>,
  pub id: SessionId,
  pub data: HashMap<String, String>,
}

impl<'a, 'r> Session<'a, 'r> {
  pub fn new(id: SessionId, request: &'a Request<'r>) -> Self {
    Session {
      request: Some(request),
      id,
      data: Default::default(),
    }
  }

  pub fn add_data<K, V>(&mut self, key: K, value: V)
    where K: Into<String>,
          V: Into<String>,
  {
    self.data.insert(key.into(), value.into());
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for Session<'a, 'r> {
    type Error = String;

    fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
      // get session (private cookie, so encrypted and authed)
      let session: Option<Session> = req
        .cookies()
        .get_private("session")
        .and_then(|x| serde_json::from_str(x.value()).ok());

      if let Some(mut s) = session {
        // return existing session
        s.request = Some(req);
        return Outcome::Success(s);
      }

      // if we're here, there is no valid session, so add one

      // create a session with a random id
      let session = Session::new(SessionId(Uuid::new_v4()), req);

      // return the new session
      Outcome::Success(session)
    }
}

impl<'a, 'r> Drop for Session<'a, 'r> {
  fn drop(&mut self) {
    if let Some(req) = self.request {
      let json = match serde_json::to_string(self) {
        Ok(j) => j,
        Err(e) => {
          println!("could not serialize session: {}", e);
          return;
        },
      };
      // FIXME: don't set cookie unless changed
      let cookie = Cookie::build("session", json)
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Lax)
        .finish();
      req.cookies().add_private(cookie);
    }
  }
}
