module Orgauth.UserId exposing (..)

import Json.Decode
import Json.Encode


type UserId
    = UserId Int


userIdEncoder : UserId -> Json.Encode.Value
userIdEncoder (UserId id) =
    Json.Encode.object
        [ ( "Uid"
          , Json.Encode.int id
          )
        ]


makeUserId : Int -> UserId
makeUserId i =
    UserId i


getUserIdVal : UserId -> Int
getUserIdVal (UserId uid) =
    uid


userIdDecoder : Json.Decode.Decoder UserId
userIdDecoder =
    Json.Decode.succeed UserId
        |> Json.Decode.andThen (\x -> Json.Decode.map x (Json.Decode.field "Uid" Json.Decode.int))
