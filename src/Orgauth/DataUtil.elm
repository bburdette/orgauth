module Orgauth.DataUtil exposing (..)

import Json.Encode as JE
import Orgauth.Data exposing (AdminResponse, UserResponse, adminResponseEncoder, userResponseEncoder)


showUserResponse : UserResponse -> String
showUserResponse pr =
    userResponseEncoder pr
        |> JE.encode 2


showAdminResponse : AdminResponse -> String
showAdminResponse pr =
    adminResponseEncoder pr
        |> JE.encode 2
