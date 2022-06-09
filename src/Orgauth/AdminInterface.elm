module Orgauth.AdminInterface exposing (SendMsg(..), ServerResponse(..), encodeSendMsg, serverResponseDecoder, showServerResponse)

import Json.Decode as JD
import Json.Encode as JE
import Orgauth.Data as Data


type SendMsg
    = GetUsers


type ServerResponse
    = Users (List Data.LoginData)
    | ServerError String
    | NotLoggedIn


showServerResponse : ServerResponse -> String
showServerResponse sr =
    case sr of
        NotLoggedIn ->
            "NotLoggedIn"

        Users _ ->
            "Users"

        ServerError _ ->
            "ServerError"


encodeSendMsg : SendMsg -> JE.Value
encodeSendMsg sm =
    case sm of
        GetUsers ->
            JE.object
                [ ( "what", JE.string "getusers" )
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

                    "not logged in" ->
                        JD.succeed NotLoggedIn

                    "server error" ->
                        JD.map ServerError (JD.at [ "data" ] JD.string)

                    wat ->
                        JD.succeed
                            (ServerError ("invalid 'what' from server: " ++ wat))
            )
