module Orgauth.AdminInterface exposing (SendMsg(..), ServerResponse(..), encodeSendMsg, serverResponseDecoder, showServerResponse)

import Json.Decode as JD
import Json.Encode as JE
import Orgauth.Data as Data exposing (UserId(..))


type SendMsg
    = GetUsers
    | DeleteUser UserId
    | UpdateUser Data.LoginData
    | GetInvite Data.GetInvite
    | GetPwdReset UserId


type ServerResponse
    = Users (List Data.LoginData)
    | UserDeleted Int
    | UserUpdated Data.LoginData
    | UserInvite Data.UserInvite
    | PwdReset Data.PwdReset
    | NotLoggedIn
    | ServerError String


showServerResponse : ServerResponse -> String
showServerResponse sr =
    case sr of
        NotLoggedIn ->
            "NotLoggedIn"

        Users _ ->
            "Users"

        UserDeleted _ ->
            "UserDeleted"

        UserUpdated _ ->
            "UserUpdated"

        UserInvite _ ->
            "UserInvite"

        PwdReset _ ->
            "PwdReset"

        ServerError _ ->
            "ServerError"


encodeSendMsg : SendMsg -> JE.Value
encodeSendMsg sm =
    case sm of
        GetUsers ->
            JE.object
                [ ( "what", JE.string "GetUsers" )
                ]

        DeleteUser id ->
            JE.object
                [ ( "what", JE.string "DeleteUser" )
                , ( "data", Data.userIdEncoder id )
                ]

        UpdateUser ld ->
            JE.object
                [ ( "what", JE.string "UpdateUser" )
                , ( "data", Data.loginDataEncoder ld )
                ]

        GetInvite gi ->
            JE.object
                [ ( "what", JE.string "GetInvite" )
                , ( "data", Data.getInviteEncoder gi )
                ]

        GetPwdReset id ->
            JE.object
                [ ( "what", JE.string "GetPwdReset" )
                , ( "data", Data.userIdEncoder id )
                ]


serverResponseDecoder : JD.Decoder ServerResponse
serverResponseDecoder =
    JD.at [ "what" ]
        JD.string
        |> JD.andThen
            (\what ->
                case what of
                    "Users" ->
                        JD.map Users (JD.at [ "data" ] (JD.list Data.loginDataDecoder))

                    "UserDeleted" ->
                        JD.map UserDeleted (JD.at [ "data" ] JD.int)

                    "UserUpdated" ->
                        JD.map UserUpdated (JD.at [ "data" ] Data.loginDataDecoder)

                    "UserInvite" ->
                        JD.map UserInvite (JD.at [ "data" ] Data.userInviteDecoder)

                    "PwdReset" ->
                        JD.map PwdReset (JD.at [ "data" ] Data.pwdResetDecoder)

                    "NotLoggedIn" ->
                        JD.succeed NotLoggedIn

                    "ServerError" ->
                        JD.map ServerError (JD.at [ "data" ] JD.string)

                    wat ->
                        JD.succeed
                            (ServerError ("invalid 'what' from server: " ++ wat))
            )
