module Orgauth.Data exposing (..)

import Json.Decode as JD
import Json.Encode as JE
import UUID exposing (UUID)
import Util exposing (andMap)



----------------------------------------
-- types sent to or from the server.
----------------------------------------


type alias Registration =
    { uid : String
    , pwd : String
    , email : String
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
    { userid : Int
    , name : String
    , admin : Bool
    , data : JD.Value
    }


type alias AdminSettings =
    { openRegistration : Bool
    }



----------------------------------------
-- Json encoders/decoders
----------------------------------------


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
        |> andMap (JD.field "userid" JD.int)
        |> andMap (JD.field "name" JD.string)
        |> andMap (JD.field "admin" JD.bool)
        |> andMap (JD.field "data" JD.value)


decodeAdminSettings : JD.Decoder AdminSettings
decodeAdminSettings =
    JD.succeed AdminSettings
        |> andMap (JD.field "open_registration" JD.bool)



------------------------------------------------
-- utiltiy fn
------------------------------------------------


toLd : { a | userid : Int, name : String, admin : Bool } -> LoginData
toLd ld =
    { userid = ld.userid
    , name = ld.name
    , admin = ld.admin
    , data = JE.null
    }
