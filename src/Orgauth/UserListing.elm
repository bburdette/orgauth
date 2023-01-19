module Orgauth.UserListing exposing (Command(..), Model, Msg(..), init, update, view)

import Element as E exposing (Element)
import Element.Background as EBk
import Element.Border as EBd
import Element.Events as EE
import Element.Font as EF
import Element.Input as EI
import Element.Region
import Orgauth.Data as Data
import TangoColors as TC
import Time exposing (Zone)
import Util


type alias Model =
    { users : List Data.LoginData
    }


type Msg
    = DoneClick
    | InviteClick
    | EditPress Data.LoginData
    | Noop


type Command
    = Done
    | InviteUser
    | EditUser Data.LoginData
    | None


init : List Data.LoginData -> Model
init users =
    { users = users
    }


view : List (E.Attribute Msg) -> Model -> Element Msg
view buttonStyle model =
    E.column
        [ E.width (E.px 500)
        , E.height E.shrink
        , E.spacing 10
        , E.centerX
        ]
        [ E.table []
            { data = model.users
            , columns =
                [ { header = E.text "user"
                  , width =
                        E.fill
                  , view =
                        \n ->
                            E.row
                                [ E.mouseDown [ EBk.color TC.blue ]
                                , E.mouseOver [ EBk.color TC.green ]
                                , EE.onClick (EditPress n)
                                ]
                                [ E.text n.name ]
                  }
                ]
            }
        , E.row [ E.width E.fill, E.spacing 10 ]
            [ EI.button (E.centerX :: buttonStyle)
                { onPress = Just InviteClick, label = E.text "invite" }
            , EI.button (E.centerX :: buttonStyle)
                { onPress = Just DoneClick, label = E.text "done" }
            ]
        ]


update : Msg -> Model -> ( Model, Command )
update msg model =
    case msg of
        DoneClick ->
            ( model, Done )

        EditPress ld ->
            ( model, EditUser ld )

        InviteClick ->
            ( model, InviteUser )

        Noop ->
            ( model, None )
