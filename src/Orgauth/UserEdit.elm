module Orgauth.UserEdit exposing (Command(..), Model, Msg(..), init, initNew, isDirty, update, view)

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
    { name : String
    , admin : Bool
    , active : Bool
    , initialUser : Maybe Data.LoginData
    }


type Msg
    = DoneClick
    | NameEdit String
    | DeleteClick Int
    | ActiveChecked Bool
    | Noop


type Command
    = Done
    | Delete Int
    | None


init : Data.LoginData -> Model
init ld =
    { name = ld.name
    , admin = ld.admin
    , active = ld.active
    , initialUser = Just ld
    }


initNew : Model
initNew =
    { name = ""
    , admin = False
    , active = False
    , initialUser = Nothing
    }


isDirty : Model -> Bool
isDirty model =
    model.initialUser
        |> Maybe.map
            (\initialUser ->
                not
                    (model.name
                        == initialUser.name
                        && model.admin
                        == initialUser.admin
                        && model.active
                        == initialUser.active
                    )
            )
        |> Maybe.withDefault True


view : List (E.Attribute Msg) -> Model -> Element Msg
view buttonStyle model =
    E.column
        [ E.width (E.px 500)
        , E.height E.shrink
        , E.spacing 10
        , E.centerX
        ]
        [ EI.text []
            { onChange = NameEdit
            , text = model.name
            , placeholder = Nothing
            , label = EI.labelLeft [] (E.text "name")
            }
        , E.row [ E.width E.fill, E.spacing 10 ]
            [ EI.checkbox []
                { onChange = ActiveChecked
                , icon = EI.defaultCheckbox
                , checked = model.active
                , label = EI.labelLeft [] (E.text "active")
                }
            , model.initialUser
                |> Maybe.map
                    (\u ->
                        EI.button (E.centerX :: buttonStyle)
                            { onPress = Just <| DeleteClick u.userid, label = E.text "delete" }
                    )
                |> Maybe.withDefault E.none
            , EI.button (E.centerX :: buttonStyle)
                { onPress = Just DoneClick, label = E.text "done" }
            ]
        ]


update : Msg -> Model -> ( Model, Command )
update msg model =
    case msg of
        DoneClick ->
            ( model, Done )

        DeleteClick id ->
            ( model, Delete id )

        ActiveChecked active ->
            ( { model | active = active }, None )

        NameEdit n ->
            ( { model | name = n }, None )

        Noop ->
            ( model, None )
