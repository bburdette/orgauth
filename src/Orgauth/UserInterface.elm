module Orgauth.UserInterface exposing (SendMsg(..), ServerResponse(..), encodeEmail, encodeSendMsg, serverResponseDecoder, showServerResponse)

import Json.Decode as JD
import Json.Encode as JE
import Orgauth.Data as Data


type SendMsg
    = Register Data.Registration
    | Login Data.Login
    | GetInvite Data.GetInvite
    | ReadInvite String
    | RSVP Data.RSVP
    | ResetPassword Data.ResetPassword
    | SetPassword Data.SetPassword
    | Logout
    | ChangePassword Data.ChangePassword
    | ChangeEmail Data.ChangeEmail


type ServerResponse
    = RegistrationSent
    | UserExists
    | UnregisteredUser
    | InvalidUserOrPwd
    | NotLoggedIn
    | LoggedIn Data.LoginData
    | LoggedOut
    | ChangedPassword
    | ChangedEmail
    | BlankUserName
    | BlankPassword
    | ResetPasswordAck
    | SetPasswordAck
    | Invite Data.UserInvite
    | ServerError String


showServerResponse : ServerResponse -> String
showServerResponse sr =
    case sr of
        RegistrationSent ->
            "RegistrationSent"

        UserExists ->
            "UserExists"

        UnregisteredUser ->
            "UnregisteredUser"

        NotLoggedIn ->
            "NotLoggedIn"

        InvalidUserOrPwd ->
            "InvalidUserOrPwd"

        LoggedIn _ ->
            "LoggedIn"

        LoggedOut ->
            "LoggedOut"

        ResetPasswordAck ->
            "ResetPasswordAck"

        SetPasswordAck ->
            "SetPasswordAck"

        ChangedPassword ->
            "ChangedPassword"

        ChangedEmail ->
            "ChangedEmail"

        BlankUserName ->
            "BlankUserName"

        BlankPassword ->
            "BlankPassword"

        Invite _ ->
            "Invite"

        ServerError _ ->
            "ServerError"


encodeSendMsg : SendMsg -> JE.Value
encodeSendMsg sm =
    case sm of
        Register registration ->
            JE.object
                [ ( "what", JE.string "Register" )
                , ( "data", Data.encodeRegistration registration )
                ]

        Login login ->
            JE.object
                [ ( "what", JE.string "Login" )
                , ( "data", Data.encodeLogin login )
                ]

        RSVP rsvp ->
            JE.object
                [ ( "what", JE.string "RSVP" )
                , ( "data", Data.encodeRSVP rsvp )
                ]

        Logout ->
            JE.object
                [ ( "what", JE.string "Logout" )
                ]

        ResetPassword chpwd ->
            JE.object
                [ ( "what", JE.string "ResetPassword" )
                , ( "data", Data.encodeResetPassword chpwd )
                ]

        SetPassword chpwd ->
            JE.object
                [ ( "what", JE.string "SetPassword" )
                , ( "data", Data.encodeSetPassword chpwd )
                ]

        ChangePassword chpwd ->
            JE.object
                [ ( "what", JE.string "ChangePassword" )
                , ( "data", Data.encodeChangePassword chpwd )
                ]

        ChangeEmail chpwd ->
            JE.object
                [ ( "what", JE.string "ChangeEmail" )
                , ( "data", Data.encodeChangeEmail chpwd )
                ]

        ReadInvite token ->
            JE.object
                [ ( "what", JE.string "ReadInvite" )
                , ( "data", JE.string token )
                ]

        GetInvite gi ->
            JE.object
                [ ( "what", JE.string "GetInvite" )
                , ( "data", Data.encodeGetInvite gi )
                ]


encodeEmail : String -> JE.Value
encodeEmail email =
    JE.object
        [ ( "email", JE.string email )
        ]


serverResponseDecoder : JD.Decoder ServerResponse
serverResponseDecoder =
    JD.at [ "what" ]
        JD.string
        |> JD.andThen
            (\what ->
                case what of
                    "RegistrationSent" ->
                        JD.succeed RegistrationSent

                    "UnregisteredUser" ->
                        JD.succeed UnregisteredUser

                    "UserExists" ->
                        JD.succeed UserExists

                    "LoggedIn" ->
                        JD.map LoggedIn (JD.at [ "data" ] Data.decodeLoginData)

                    "LoggedOut" ->
                        JD.succeed LoggedOut

                    "NotLoggedIn" ->
                        JD.succeed NotLoggedIn

                    "InvalidUserOrPwd" ->
                        JD.succeed InvalidUserOrPwd

                    "ResetPasswordAck" ->
                        JD.succeed ResetPasswordAck

                    "SetPasswordAck" ->
                        JD.succeed SetPasswordAck

                    "ChangedPassword" ->
                        JD.succeed ChangedPassword

                    "ChangedEmail" ->
                        JD.succeed ChangedEmail

                    "BlankUserName" ->
                        JD.succeed BlankUserName

                    "BlankPassword" ->
                        JD.succeed BlankPassword

                    "Invite" ->
                        JD.map Invite (JD.at [ "data" ] Data.decodeUserInvite)

                    "ServerError" ->
                        JD.map ServerError (JD.at [ "data" ] JD.string)

                    wat ->
                        JD.succeed
                            (ServerError ("invalid 'what' from server: " ++ wat))
            )
