module Orgauth.Data exposing (..)

import Json.Decode as JD
import Json.Encode as JE
import UUID exposing (UUID)
import Util exposing (andMap)



----------------------------------------
-- types sent to or from the server.
----------------------------------------


type UserId
    = UserId Int


type alias Registration =
    { uid : String
    , pwd : String
    , email : String
    }


type alias RSVP =
    { uid : String
    , pwd : String
    , email : String
    , invite : String
    }


type alias GetInvite =
    { email : Maybe String
    , data : Maybe String
    }


type alias Login =
    { uid : String
    , pwd : String
    }


type alias ResetPassword =
    { uid : String
    }


type alias SetPassword =
    { uid : String
    , newpwd : String
    , reset_key : UUID
    }


type alias ChangePassword =
    { oldpwd : String
    , newpwd : String
    }


type alias ChangeEmail =
    { pwd : String
    , email : String
    }


type alias LoginData =
    { userid : UserId
    , name : String
    , email : String
    , admin : Bool
    , active : Bool
    , data : JD.Value
    }


type alias UserInvite =
    { url : String
    , token : String
    , email : Maybe String
    }



{--

Modes:
  - admin invite mode.
    - openRegistration = false
    - nonAdminInvite = false
  - user invite mode.
    - openRegistration = false
    - nonAdminInvite = true
  - open email registration.
    - openRegistration = true
    - nonAdminInvite = true ??  
  - no-email registration.
    - still allow email reset?

    
Q:  email required or no?  

email optional vs email required vs email inactive.

email optional:
  - registration, but confirmation email not sent (?)
  - uid/pwd works right away.
  - email field is optional, shown as optional.
  - email account reset if email present.

email required:
  - magic link sent to confirm account.
  - email field mandatory.
  - email account reset.

email inactive
  - email field hidden.
  - account reset not possible
  - account immediately available


--}


type alias AdminSettings =
    { openRegistration : Bool
    , sendEmails : Bool
    , nonAdminInvite : Bool
    }


type alias PwdReset =
    { userid : UserId
    , url : String
    }



----------------------------------------
-- Json encoders/decoders
----------------------------------------


makeUserId : Int -> UserId
makeUserId i =
    UserId i


getUserIdVal : UserId -> Int
getUserIdVal uid =
    case uid of
        UserId i ->
            i


encodeUserId : UserId -> JE.Value
encodeUserId =
    getUserIdVal >> JE.int


decodeUserId : JD.Decoder UserId
decodeUserId =
    JD.int |> JD.map makeUserId


encodeRegistration : Registration -> JE.Value
encodeRegistration l =
    JE.object
        [ ( "uid", JE.string l.uid )
        , ( "pwd", JE.string l.pwd )
        , ( "email", JE.string l.email )
        ]


encodeLogin : Login -> JE.Value
encodeLogin l =
    JE.object
        [ ( "uid", JE.string l.uid )
        , ( "pwd", JE.string l.pwd )
        ]


encodeRSVP : RSVP -> JE.Value
encodeRSVP l =
    JE.object
        [ ( "uid", JE.string l.uid )
        , ( "pwd", JE.string l.pwd )
        , ( "email", JE.string l.email )
        , ( "invite", JE.string l.invite )
        ]


encodeResetPassword : ResetPassword -> JE.Value
encodeResetPassword l =
    JE.object
        [ ( "uid", JE.string l.uid )
        ]


encodeSetPassword : SetPassword -> JE.Value
encodeSetPassword l =
    JE.object
        [ ( "uid", JE.string l.uid )
        , ( "newpwd", JE.string l.newpwd )
        , ( "reset_key", UUID.toValue l.reset_key )
        ]


encodeChangePassword : ChangePassword -> JE.Value
encodeChangePassword l =
    JE.object
        [ ( "oldpwd", JE.string l.oldpwd )
        , ( "newpwd", JE.string l.newpwd )
        ]


encodeChangeEmail : ChangeEmail -> JE.Value
encodeChangeEmail l =
    JE.object
        [ ( "pwd", JE.string l.pwd )
        , ( "email", JE.string l.email )
        ]


decodeLoginData : JD.Decoder LoginData
decodeLoginData =
    JD.succeed LoginData
        |> andMap (JD.field "userid" JD.int |> JD.map makeUserId)
        |> andMap (JD.field "name" JD.string)
        |> andMap (JD.field "email" JD.string)
        |> andMap (JD.field "admin" JD.bool)
        |> andMap (JD.field "active" JD.bool)
        |> andMap (JD.field "data" JD.value)


encodeLoginData : LoginData -> JE.Value
encodeLoginData ld =
    JE.object
        [ ( "userid", JE.int <| getUserIdVal ld.userid )
        , ( "name", JE.string ld.name )
        , ( "email", JE.string ld.email )
        , ( "admin", JE.bool ld.admin )
        , ( "active", JE.bool ld.active )
        , ( "data", ld.data )
        ]


decodeAdminSettings : JD.Decoder AdminSettings
decodeAdminSettings =
    JD.succeed AdminSettings
        |> andMap (JD.field "open_registration" JD.bool)
        |> andMap (JD.field "send_emails" JD.bool)
        |> andMap (JD.field "non_admin_invite" JD.bool)


encodeGetInvite : GetInvite -> JE.Value
encodeGetInvite gi =
    JE.object <|
        List.filterMap identity
            [ gi.email |> Maybe.map (\e -> ( "email", JE.string e ))
            , gi.data |> Maybe.map (\e -> ( "data", JE.string e ))
            ]


decodeUserInvite : JD.Decoder UserInvite
decodeUserInvite =
    JD.succeed UserInvite
        |> andMap (JD.field "url" JD.string)
        |> andMap (JD.field "token" JD.string)
        |> andMap (JD.maybe (JD.field "email" JD.string))


decodePwdReset : JD.Decoder PwdReset
decodePwdReset =
    JD.succeed PwdReset
        |> andMap (JD.field "userid" JD.int |> JD.map makeUserId)
        |> andMap (JD.field "url" JD.string)



------------------------------------------------
-- utiltiy fn
------------------------------------------------


toLd : { a | userid : UserId, name : String, email : String, admin : Bool, active : Bool } -> LoginData
toLd ld =
    { userid = ld.userid
    , name = ld.name
    , email = ld.email
    , admin = ld.admin
    , active = ld.active
    , data = JE.null
    }
