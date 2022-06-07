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
    = OkClick
    | CancelClick
    | Noop


type Command
    = None


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
        ]
        [ E.table []
            { data = model.users
            , columns =
                [ { header = E.text "name"
                  , width =
                        E.fill
                  , view =
                        \n -> E.text n.name
                  }
                ]
            }
        , E.row [ E.width E.fill, E.spacing 10 ]
            [ EI.button buttonStyle
                { onPress = Just OkClick, label = E.text "Ok" }
            , EI.button
                buttonStyle
                { onPress = Just CancelClick, label = E.text "Cancel" }
            ]
        ]


update : Msg -> Model -> ( Model, Command )
update msg model =
    case msg of
        CancelClick ->
            ( model, None )

        OkClick ->
            ( model, None )

        Noop ->
            ( model, None )
