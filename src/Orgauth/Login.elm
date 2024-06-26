module Orgauth.Login exposing (..)

import Common exposing (buttonStyle)
import Dict exposing (Dict)
import Element exposing (..)
import Element.Background as Background
import Element.Border as Border
import Element.Input as Input
import Orgauth.Data as Data
import Random exposing (Seed, int, step)
import Toop
import Util
import WindowKeys as WK


type Mode
    = RegistrationMode
    | LoginMode
    | ResetMode


type alias StylePalette a =
    { a
        | defaultSpacing : Int
    }


type alias Model =
    { userId : String
    , password : String
    , email : String
    , remoteUrl : String
    , captcha : String
    , captchaQ : ( String, Int )
    , seed : Seed
    , mode : Mode
    , sent : Bool
    , responseMessage : String
    , postLoginUrl : Maybe ( List String, Dict String String )
    , appname : String
    , adminSettings : Data.AdminSettings
    }


type Msg
    = IdUpdate String
    | CaptchaUpdate String
    | PasswordUpdate String
    | EmailUpdate String
    | RemoteUrlUpdate String
    | SetMode Mode
    | LoginPressed
    | RegisterPressed
    | ResetPressed
    | CancelSent


type Cmd
    = Login
    | Register
    | Reset
    | None


captchaQ : Seed -> ( Seed, String, Int )
captchaQ seed =
    let
        ( a, seed1 ) =
            step (int 0 100) seed

        ( b, seed2 ) =
            step (int 0 100) seed1
    in
    ( seed2
    , "Whats " ++ String.fromInt a ++ " + " ++ String.fromInt b ++ "?"
    , a + b
    )


initialModel : Maybe { uid : String, pwd : String } -> Data.AdminSettings -> String -> Seed -> Model
initialModel mblogin adminSettings appname seed =
    let
        ( newseed, cq, cans ) =
            captchaQ seed

        ( uid, pwd ) =
            case mblogin of
                Just info ->
                    ( info.uid, info.pwd )

                Nothing ->
                    ( "", "" )
    in
    { userId = uid
    , password = pwd
    , email = ""
    , remoteUrl = ""
    , captcha = ""
    , captchaQ = ( cq, cans )
    , seed = newseed
    , mode = LoginMode
    , sent = False
    , responseMessage = ""
    , postLoginUrl = Nothing
    , appname = appname
    , adminSettings = adminSettings
    }


makeUrlP : Model -> ( String, Dict String String )
makeUrlP model =
    case model.mode of
        RegistrationMode ->
            ( "/registration", Dict.empty )

        LoginMode ->
            ( "/login", Dict.empty )

        ResetMode ->
            ( "/reset", Dict.empty )


urlToState : List String -> Dict String String -> Model -> Model
urlToState segments parms model =
    { model
        | mode =
            case List.head segments of
                Just "login" ->
                    LoginMode

                Just "reset" ->
                    ResetMode

                Just "registration" ->
                    RegistrationMode

                _ ->
                    model.mode
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
        , sent = True -- was false??
    }


invalidUserOrPwd : Model -> Model
invalidUserOrPwd model =
    { model
        | responseMessage = "can't login - invalid user or password."
        , sent = False
    }


blankPassword : Model -> Model
blankPassword model =
    { model
        | responseMessage = "password cannot be empty!"
        , sent = False
    }


blankUserName : Model -> Model
blankUserName model =
    { model
        | responseMessage = "user name cannot be empty!"
        , sent = False
    }


view : StylePalette a -> Util.Size -> Model -> Element Msg
view style size model =
    column [ width fill, height (px size.height) ]
        [ column
            [ centerX
            , centerY
            , width <| maximum 450 fill
            , height <| maximum 450 fill
            , Background.color (Common.navbarColor 1)
            , Border.rounded 10
            , padding 10
            ]
            [ row [ width fill ]
                [ Common.navbar 0
                    model.mode
                    SetMode
                    (List.filterMap identity
                        [ Just ( LoginMode, "log in" )
                        , if model.adminSettings.openRegistration then
                            Just ( RegistrationMode, "register" )

                          else
                            Nothing
                        , Just ( ResetMode, "reset" )
                        ]
                    )
                ]
            , if model.sent then
                sentView model

              else
                case model.mode of
                    LoginMode ->
                        loginView style model

                    ResetMode ->
                        resetView style model

                    RegistrationMode ->
                        registrationView style model
            ]
        ]


loginView : StylePalette a -> Model -> Element Msg
loginView style model =
    column
        [ spacing style.defaultSpacing
        , width fill
        , height fill
        , padding 10
        , Background.color (Common.navbarColor 1)
        ]
        [ text <| "welcome to " ++ model.appname ++ "!"
        , text <| "log in below:"
        , Input.username [ width fill ]
            { onChange = IdUpdate
            , text = model.userId
            , placeholder = Nothing
            , label = Input.labelLeft [] <| text "user id:"
            }
        , Input.currentPassword [ width fill ]
            { onChange = PasswordUpdate
            , text = model.password
            , placeholder = Nothing
            , label = Input.labelLeft [] <| text "password: "
            , show = False
            }
        , text model.responseMessage
        , Input.button (buttonStyle ++ [ width fill, alignBottom ])
            { onPress = Just LoginPressed
            , label = text "log in"
            }
        ]


resetView : StylePalette a -> Model -> Element Msg
resetView style model =
    column
        [ spacing style.defaultSpacing
        , width fill
        , height fill
        , padding 10
        , Background.color (Common.navbarColor 1)
        ]
        [ text <| "forgot your password?"
        , Input.text [ width fill ]
            { onChange = IdUpdate
            , text = model.userId
            , placeholder = Nothing
            , label = Input.labelLeft [] <| text "user id:"
            }
        , text model.responseMessage
        , Input.button (buttonStyle ++ [ width fill, alignBottom ])
            { onPress = Just ResetPressed
            , label = text "send reset email"
            }
        ]


registrationView : StylePalette a -> Model -> Element Msg
registrationView style model =
    column [ Background.color (Common.navbarColor 1), width fill, height fill, spacing style.defaultSpacing, padding 8 ]
        [ text <| "welcome to " ++ model.appname ++ "!"
        , text <| "register your new account below:"
        , Input.username []
            { onChange = IdUpdate
            , text = model.userId
            , placeholder = Nothing
            , label = Input.labelLeft [] <| text "user id:"
            }
        , Input.newPassword []
            { onChange = PasswordUpdate
            , text = model.password
            , placeholder = Nothing
            , label = Input.labelLeft [] <| text "password: "
            , show = False
            }

        -- TODO prevent both email and remoteUrl??
        , if model.adminSettings.sendEmails then
            Input.email []
                { onChange = EmailUpdate
                , text = model.email
                , placeholder = Nothing
                , label = Input.labelLeft [] <| text "email:"
                }

          else
            none
        , if model.adminSettings.remoteRegistration then
            Input.email []
                { onChange = RemoteUrlUpdate
                , text = model.remoteUrl
                , placeholder = Nothing
                , label = Input.labelLeft [] <| text "remote url:"
                }

          else
            none
        , Input.text []
            { onChange = CaptchaUpdate
            , text = model.captcha
            , placeholder = Nothing
            , label = Input.labelLeft [] <| text <| Tuple.first model.captchaQ
            }
        , text model.responseMessage
        , Input.button (buttonStyle ++ [ width fill, alignBottom ])
            { onPress = Just RegisterPressed
            , label = text "register"
            }
        ]


sentView : Model -> Element Msg
sentView model =
    column [ width fill ]
        [ text
            (case model.mode of
                LoginMode ->
                    "Logging in..."

                RegistrationMode ->
                    "Registration sent..."

                ResetMode ->
                    "Reset sent..."
            )
        ]


onWkKeyPress : WK.Key -> Model -> ( Model, Cmd )
onWkKeyPress key model =
    case Toop.T4 key.key key.ctrl key.alt key.shift of
        Toop.T4 "Enter" False False False ->
            case model.mode of
                LoginMode ->
                    update LoginPressed model

                RegistrationMode ->
                    update RegisterPressed model

                ResetMode ->
                    update ResetPressed model

        _ ->
            ( model, None )


update : Msg -> Model -> ( Model, Cmd )
update msg model =
    case msg of
        IdUpdate id ->
            ( { model | userId = id, sent = False }, None )

        PasswordUpdate txt ->
            ( { model | password = txt, sent = False }, None )

        EmailUpdate txt ->
            ( { model | email = txt, sent = False }, None )

        RemoteUrlUpdate txt ->
            ( { model | remoteUrl = txt, sent = False }, None )

        CaptchaUpdate txt ->
            ( { model | captcha = txt, sent = False }, None )

        SetMode mode ->
            ( { model | mode = mode, sent = False, responseMessage = "" }, None )

        CancelSent ->
            ( { model | sent = False }, None )

        RegisterPressed ->
            let
                ( newseed, cq, cans ) =
                    captchaQ model.seed

                newmod =
                    { model
                        | seed = newseed
                        , captchaQ = ( cq, cans )
                    }
            in
            if String.toInt model.captcha == (Just <| Tuple.second model.captchaQ) then
                ( newmod, Register )

            else
                ( { newmod | responseMessage = "check your math!" }, None )

        LoginPressed ->
            ( { model | sent = True }, Login )

        ResetPressed ->
            ( { model | sent = True }, Reset )
