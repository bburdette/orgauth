module Orgauth.AdminInterface exposing (SendMsg(..), ServerResponse(..), encodeSendMsg, serverResponseDecoder, showServerResponse)

import Json.Decode as JD
import Json.Encode as JE
import Orgauth.Data as Data


type SendMsg
    = GetUsers
    | DeleteUser Data.UserId
    | UpdateUser Data.LoginData
    | GetInvite Data.GetInvite
    | GetPwdReset Data.UserId


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
                , ( "data", JE.int <| Data.getUserIdVal id )
                ]

        UpdateUser ld ->
            JE.object
                [ ( "what", JE.string "UpdateUser" )
                , ( "data", Data.encodeLoginData ld )
                ]

        GetInvite gi ->
            JE.object
                [ ( "what", JE.string "GetInvite" )
                , ( "data", Data.encodeGetInvite gi )
                ]

        GetPwdReset id ->
            JE.object
                [ ( "what", JE.string "GetPwdReset" )
                , ( "data", JE.int <| Data.getUserIdVal id )
                ]


serverResponseDecoder : JD.Decoder ServerResponse
serverResponseDecoder =
    JD.at [ "what" ]
        JD.string
        |> JD.andThen
            (\what ->
                case what of
                    "Users" ->
                        JD.map Users (JD.at [ "data" ] (JD.list Data.decodeLoginData))

                    "UserDeleted" ->
                        JD.map UserDeleted (JD.at [ "data" ] JD.int)

                    "UserUpdated" ->
                        JD.map UserUpdated (JD.at [ "data" ] Data.decodeLoginData)

                    "UserInvite" ->
                        JD.map UserInvite (JD.at [ "data" ] Data.decodeUserInvite)

                    "PwdReset" ->
                        JD.map PwdReset (JD.at [ "data" ] Data.decodePwdReset)

                    "NotLoggedIn" ->
                        JD.succeed NotLoggedIn

                    "ServerError" ->
                        JD.map ServerError (JD.at [ "data" ] JD.string)

                    wat ->
                        JD.succeed
                            (ServerError ("invalid 'what' from server: " ++ wat))
            )
