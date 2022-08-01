module Orgauth.Invited exposing (Cmd(..), Model, Msg(..), StylePalette, initialModel, invalidUserOrPwd, onWkKeyPress, registrationSent, registrationView, sentView, unregisteredUser, update, userExists, view)

import Common exposing (buttonStyle)
import Dict exposing (Dict)
import Element exposing (..)
import Element.Background as Background
import Element.Border as Border
import Element.Events exposing (onClick)
import Element.Font as Font
import Element.Input as Input
import Html exposing (Html)
import Orgauth.Data as Data
import Random exposing (Seed)
import TangoColors as Color
import Toop
import Util exposing (httpErrorString)
import WindowKeys as WK


type alias StylePalette a =
    { a
        | defaultSpacing : Int
    }


type alias Model =
    { userId : String
    , password : String
    , email : String
    , invite : String
    , responseMessage : String
    , sent : Bool
    , appname : String
    , adminSettings : Data.AdminSettings
    }


type Msg
    = IdUpdate String
    | PasswordUpdate String
    | EmailUpdate String
    | RSVPPressed
    | CancelSent


type Cmd
    = RSVP
    | None


initialModel : Data.UserInvite -> Data.AdminSettings -> String -> Model
initialModel invite adminSettings appname =
    { userId = ""
    , password = ""
    , email = invite.email |> Maybe.withDefault ""
    , invite = invite.token
    , sent = False
    , responseMessage = ""
    , appname = appname
    , adminSettings = adminSettings
    }


userExists : Model -> Model
userExists model =
    { model
        | responseMessage = "can't register - this user id already exixts!"
        , sent = False
    }


unregisteredUser : Model -> Model
unregisteredUser model =
    { model
        | responseMessage = "can't login - this user is not registered"
        , sent = False
    }


registrationSent : Model -> Model
registrationSent model =
    { model
        | responseMessage = "registration sent.  check your spam folder for email from " ++ model.appname ++ "!"
        , sent = False
    }


invalidUserOrPwd : Model -> Model
invalidUserOrPwd model =
    { model
        | responseMessage = "can't login - invalid user or password."
        , sent = False
    }


view : StylePalette a -> Util.Size -> Model -> Element Msg
view style size model =
    column [ width fill, height (px size.height) ]
        [ column
            [ centerX
            , centerY
            , width <| maximum 450 fill
            , height <| maximum 420 fill
            , Background.color (Common.navbarColor 1)
            , Border.rounded 10
            , padding 10
            ]
            [ if model.sent then
                sentView model

              else
                registrationView style model
            ]
        ]


registrationView : StylePalette a -> Model -> Element Msg
registrationView style model =
    column [ Background.color (Common.navbarColor 1), width fill, height fill, spacing style.defaultSpacing, padding 8 ]
        [ text <| "welcome to " ++ model.appname ++ "!"
        , text <| "register your new account below:"
        , Input.text []
            { onChange = IdUpdate
            , text = model.userId
            , placeholder = Nothing
            , label = Input.labelLeft [] <| text "user id:"
            }
        , Input.currentPassword []
            { onChange = PasswordUpdate
            , text = model.password
            , placeholder = Nothing
            , label = Input.labelLeft [] <| text "password: "
            , show = False
            }
        , Input.text []
            { onChange = EmailUpdate
            , text = model.email
            , placeholder = Nothing
            , label = Input.labelLeft [] <| text "email (optional):"
            }
        , text model.responseMessage
        , Input.button (buttonStyle ++ [ width fill, alignBottom ])
            { onPress = Just RSVPPressed
            , label = text "register"
            }
        ]


sentView : Model -> Element Msg
sentView model =
    column [ width fill ]
        [ text
            "Registration sent..."
        ]


onWkKeyPress : WK.Key -> Model -> ( Model, Cmd )
onWkKeyPress key model =
    case Toop.T4 key.key key.ctrl key.alt key.shift of
        Toop.T4 "Enter" False False False ->
            update RSVPPressed model

        _ ->
            ( model, None )


update : Msg -> Model -> ( Model, Cmd )
update msg model =
    case msg of
        IdUpdate id ->
            ( { model | userId = id, sent = False }, None )

        EmailUpdate txt ->
            ( { model | email = txt, sent = False }, None )

        PasswordUpdate txt ->
            ( { model | password = txt, sent = False }, None )

        CancelSent ->
            ( { model | sent = False }, None )

        RSVPPressed ->
            ( { model | sent = True }, RSVP )
