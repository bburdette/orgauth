module Orgauth.AdminInterface exposing (SendMsg(..), ServerResponse(..), encodeSendMsg, serverResponseDecoder, showServerResponse)

import Json.Decode as JD
import Json.Encode as JE
import Orgauth.Data as Data


type SendMsg
    = GetUsers
    | DeleteUser Int
    | UpdateUser Data.LoginData
    | GetInvite Data.GetInvite


type ServerResponse
    = Users (List Data.LoginData)
    | UserDeleted Int
    | UserUpdated Data.LoginData
    | ServerError String
    | UserInvite Data.UserInvite
    | NotLoggedIn


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

        ServerError _ ->
            "ServerError"


encodeSendMsg : SendMsg -> JE.Value
encodeSendMsg sm =
    case sm of
        GetUsers ->
            JE.object
                [ ( "what", JE.string "getusers" )
                ]

        DeleteUser id ->
            JE.object
                [ ( "what", JE.string "deleteuser" )
                , ( "data", JE.int id )
                ]

        UpdateUser ld ->
            JE.object
                [ ( "what", JE.string "updateuser" )
                , ( "data", Data.encodeLoginData ld )
                ]

        GetInvite gi ->
            JE.object
                [ ( "what", JE.string "getinvite" )
                , ( "data", Data.encodeGetInvite gi )
                ]


serverResponseDecoder : JD.Decoder ServerResponse
serverResponseDecoder =
    JD.at [ "what" ]
        JD.string
        |> JD.andThen
            (\what ->
                case what of
                    "users" ->
                        JD.map Users (JD.at [ "data" ] (JD.list Data.decodeLoginData))

                    "user deleted" ->
                        JD.map UserDeleted (JD.at [ "data" ] JD.int)

                    "user updated" ->
                        JD.map UserUpdated (JD.at [ "data" ] Data.decodeLoginData)

                    "user invite" ->
                        JD.map UserInvite (JD.at [ "data" ] Data.decodeUserInvite)

                    "not logged in" ->
                        JD.succeed NotLoggedIn

                    "server error" ->
                        JD.map ServerError (JD.at [ "data" ] JD.string)

                    wat ->
                        JD.succeed
                            (ServerError ("invalid 'what' from server: " ++ wat))
            )
