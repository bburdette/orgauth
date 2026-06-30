module Orgauth.UserId exposing (..)

import Orgauth.Data exposing (UserId(..))


makeUserId : Int -> UserId
makeUserId i =
    Uid i


getUserIdVal : UserId -> Int
getUserIdVal (Uid uid) =
    uid
