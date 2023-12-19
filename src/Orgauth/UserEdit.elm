module Orgauth.UserEdit exposing (Command(..), Model, Msg(..), init, initNew, isDirty, onUserUpdated, update, view)

import Element as E exposing (Element)
import Element.Background as EBk
import Element.Border as EBd
import Element.Events as EE
import Element.Font as EF
import Element.Input as EI
import Element.Region
import Orgauth.Data as Data exposing (UserId)
import TangoColors as TC
import Time exposing (Zone)
import UUID exposing (UUID)
import Util


type alias Model =
    { name : String
    , uuid : Maybe UUID
    , email : String
    , admin : Bool
    , active : Bool
    , initialUser : Maybe Data.LoginData
    }


type Msg
    = DoneClick
    | RevertClick
    | NameEdit String
    | EmailEdit String
    | DeleteClick UserId
    | ResetPwdClick UserId
    | ActiveChecked Bool
    | AdminChecked Bool
    | SaveClick
    | Noop


type Command
    = Done
    | Delete UserId
    | ResetPwd UserId
    | Save Data.LoginData
    | None


init : Data.LoginData -> Model
init ld =
    { name = ld.name
    , uuid = Just ld.uuid
    , email = ld.email
    , admin = ld.admin
    , active = ld.active
    , initialUser = Just ld
    }


initNew : Model
initNew =
    { name = ""
    , uuid = Nothing
    , email = ""
    , admin = False
    , active = False
    , initialUser = Nothing
    }


onUserUpdated : Model -> Data.LoginData -> Model
onUserUpdated _ ld =
    init ld


isDirty : Model -> Bool
isDirty model =
    model.initialUser
        |> Maybe.map
            (\initialUser ->
                not
                    (model.name
                        == initialUser.name
                        && model.email
                        == initialUser.email
                        && model.admin
                        == initialUser.admin
                        && model.active
                        == initialUser.active
                    )
            )
        |> Maybe.withDefault True


view : List (E.Attribute Msg) -> Model -> Element Msg
view buttonStyle model =
    let
        dirty =
            isDirty model
    in
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
        , EI.text []
            { onChange = EmailEdit
            , text = model.email
            , placeholder = Nothing
            , label = EI.labelLeft [] (E.text "email")
            }
        , E.row [ E.width E.fill, E.spacing 10 ]
            [ EI.checkbox []
                { onChange = ActiveChecked
                , icon = EI.defaultCheckbox
                , checked = model.active
                , label = EI.labelLeft [] (E.text "active")
                }
            , EI.checkbox []
                { onChange = AdminChecked
                , icon = EI.defaultCheckbox
                , checked = model.admin
                , label = EI.labelLeft [] (E.text "admin")
                }
            , model.initialUser
                |> Maybe.map
                    (\u ->
                        EI.button (E.centerX :: buttonStyle)
                            { onPress = Just <| ResetPwdClick u.userid, label = E.text "reset" }
                    )
                |> Maybe.withDefault E.none
            , model.initialUser
                |> Maybe.map
                    (\u ->
                        EI.button (E.centerX :: buttonStyle)
                            { onPress = Just <| DeleteClick u.userid, label = E.text "delete" }
                    )
                |> Maybe.withDefault E.none
            , if dirty then
                EI.button (E.centerX :: buttonStyle ++ [ EBk.color TC.orange ])
                    { onPress = Just SaveClick, label = E.text "save" }

              else
                E.none
            , if dirty then
                EI.button (E.centerX :: buttonStyle)
                    { onPress = Just RevertClick, label = E.text "revert" }

              else
                EI.button (E.centerX :: buttonStyle)
                    { onPress = Just DoneClick, label = E.text "done" }
            ]
        ]


update : Msg -> Model -> ( Model, Command )
update msg model =
    case msg of
        DoneClick ->
            ( model, Done )

        RevertClick ->
            model.initialUser
                |> Maybe.map (\ld -> ( init ld, None ))
                |> Maybe.withDefault ( model, None )

        DeleteClick id ->
            ( model, Delete id )

        ResetPwdClick id ->
            ( model, ResetPwd id )

        SaveClick ->
            model.initialUser
                |> Maybe.map
                    (\ld ->
                                    ( model
                                    , Save
                                        { userid = ld.userid
                                        , uuid = ld.uuid
                                        , name = model.name
                                        , email = model.email
                                        , admin = model.admin
                                        , active = model.active
                                        , data = ld.data
                                        }
                                    )
                    )
                |> Maybe.withDefault ( model, None )

        ActiveChecked active ->
            ( { model | active = active }, None )

        AdminChecked admin ->
            ( { model | admin = admin }, None )

        NameEdit s ->
            ( { model | name = s }, None )

        EmailEdit s ->
            ( { model | email = s }, None )

        Noop ->
            ( model, None )
