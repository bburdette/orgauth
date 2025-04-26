module Orgauth.ChangeRemoteUrl exposing (GDModel, Model, Msg(..), init, update, view)

import Element as E exposing (Element)
import Element.Input as EI
import GenDialog as GD
import Orgauth.Data as Data
import Util


type alias Model =
    { loginData : Data.LoginData
    , pwd : String
    , remoteUrl : String
    }


type Msg
    = PwdChanged String
    | RemoteUrlChanged String
    | OkClick
    | CancelClick
    | Noop


type alias GDModel =
    GD.Model Model Msg Data.ChangeRemoteUrl


init : Data.LoginData -> List (E.Attribute Msg) -> Element () -> GDModel
init loginData buttonStyle underLay =
    { view = view buttonStyle
    , update = update
    , model = { loginData = loginData, pwd = "", remoteUrl = "" }
    , underLay = underLay
    }


view : List (E.Attribute Msg) -> Maybe Util.Size -> Model -> Element Msg
view buttonStyle mbsize model =
    E.column
        [ E.width (mbsize |> Maybe.map .width |> Maybe.withDefault 500 |> E.px)
        , E.height E.shrink
        , E.spacing 10
        ]
        [ EI.currentPassword []
            { onChange = PwdChanged
            , text = model.pwd
            , placeholder = Nothing
            , show = False
            , label = EI.labelLeft [] (E.text "password")
            }
        , EI.text []
            { onChange = RemoteUrlChanged
            , text = model.remoteUrl
            , placeholder =
                model.loginData.remoteUrl
                    |> Maybe.map (\ru -> EI.placeholder [] (E.text ru))
            , label = EI.labelLeft [] (E.text "new remote url")
            }
        , E.row [ E.width E.fill, E.spacing 10 ]
            [ EI.button buttonStyle
                { onPress = Just OkClick, label = E.text "Ok" }
            , EI.button
                buttonStyle
                { onPress = Just CancelClick, label = E.text "Cancel" }
            ]
        ]


update : Msg -> Model -> GD.Transition Model Data.ChangeRemoteUrl
update msg model =
    case msg of
        PwdChanged s ->
            GD.Dialog { model | pwd = s }

        RemoteUrlChanged s ->
            GD.Dialog { model | remoteUrl = s }

        CancelClick ->
            GD.Cancel

        OkClick ->
            GD.Ok
                { pwd = model.pwd
                , remoteUrl = model.remoteUrl
                }

        Noop ->
            GD.Dialog model
