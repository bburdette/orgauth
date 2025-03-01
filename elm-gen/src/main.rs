use std::path::Path;

use orgauth::data as od;

fn main() {
  let ed = Path::new("../src/Orgauth/");
  {
    let mut target = vec![];
    // elm_rs provides a macro for conveniently creating an Elm module with everything needed
    elm_rs::export!(
    "Orgauth.Data",
    &mut target,
    {        // generates types and encoders for types implementing ElmEncoder
    encoders: [ od::UserId,
      od::LoginData ,
      od::AdminSettings ,
      od::User ,
      od::PhantomUser ,
      od::UserInvite ,
      od::GetInvite ,
      od::RegistrationData ,
      od::RSVP ,
      od::Login ,
      od::ResetPassword ,
      od::PwdReset ,
      od::SetPassword ,
      od::ChangePassword ,
      od::ChangeEmail ,
      od::UserRequest ,
      od::AuthedRequest ,
      od::UserResponse ,
      od::AdminRequest ,
      od::AdminResponse ,]
    decoders: [ od::UserId,
      od::LoginData ,
      od::AdminSettings ,
      od::User ,
      od::PhantomUser ,
      od::UserInvite ,
      od::GetInvite ,
      od::RegistrationData ,
      od::RSVP ,
      od::Login ,
      od::ResetPassword ,
      od::PwdReset ,
      od::SetPassword ,
      od::ChangePassword ,
      od::ChangeEmail ,
      od::UserRequest ,
      od::AuthedRequest ,
      od::UserResponse ,
      od::AdminRequest ,
      od::AdminResponse ,]
    // generates types and functions for forming queries for types implementing ElmQuery
    queries: [],
    // generates types and functions for forming queries for types implementing ElmQueryField
    query_fields: [],
    });
    let output = String::from_utf8(target).unwrap();
    let outf = ed.join("Data.elm").to_str().unwrap().to_string();
    orgauth::util::write_string(outf.as_str(), output.as_str()).unwrap();

    println!("wrote file: {}", outf);
  }
}
