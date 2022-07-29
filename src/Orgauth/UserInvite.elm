module Orgauth.UserInvite exposing (Command(..), Model, Msg(..), init, update, view)

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
    { invite : Data.UserInvite
    }


type Msg
    = DoneClick
    | Noop


type Command
    = Done
    | None


init : Data.UserInvite -> Model
init ui =
    { invite = ui
    }


view : List (E.Attribute Msg) -> Model -> Element Msg
view buttonStyle model =
    E.column
        [ E.width (E.px 500)
        , E.height E.shrink
        , E.spacing 10
        , E.centerX
        ]
        [ E.text "Send this url to the invited user!"
        , EI.text []
            { onChange = always Noop
            , text = model.invite.url
            , placeholder = Nothing
            , label = EI.labelHidden "invite url"
            }
        , E.row [ E.width E.fill, E.spacing 10 ]
            [ EI.button (E.centerX :: buttonStyle)
                { onPress = Just DoneClick, label = E.text "done" }
            ]
        ]


update : Msg -> Model -> ( Model, Command )
update msg model =
    case msg of
        DoneClick ->
            ( model, Done )

        Noop ->
            ( model, None )
