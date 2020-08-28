port module Main exposing (main)

import BadError
import Browser
import Browser.Navigation
import Cellme.Cellme exposing (Cell, CellContainer(..), CellState, RunState(..), evalCellsFully, evalCellsOnce)
import Cellme.DictCellme exposing (CellDict(..), DictCell, dictCcr, getCd, mkCc)
import Data
import Dict exposing (Dict)
import EditZk
import EditZkListing
import EditZkNote
import EditZkNoteListing
import Element exposing (Element)
import Element.Background as EBk
import Element.Border as EBd
import Element.Font as Font
import Element.Input as EI
import Element.Region
import Html exposing (Attribute, Html)
import Html.Attributes
import Http
import Json.Decode as JD
import Login
import Markdown.Block as Block exposing (Block, Inline, ListItem(..), Task(..))
import Markdown.Html
import Markdown.Parser
import Markdown.Renderer
import PublicInterface as PI
import Random exposing (Seed, initialSeed)
import Schelme.Show exposing (showTerm)
import ShowMessage
import Url exposing (Url)
import Url.Parser as UP exposing ((</>))
import UserInterface as UI
import Util
import View


type Msg
    = LoginMsg Login.Msg
    | BadErrorMsg BadError.Msg
    | ViewMsg View.Msg
    | EditZkMsg EditZk.Msg
    | EditZkListingMsg EditZkListing.Msg
    | EditZkNoteMsg EditZkNote.Msg
    | EditZkNoteListingMsg EditZkNoteListing.Msg
    | ShowMessageMsg ShowMessage.Msg
    | UserReplyData (Result Http.Error UI.ServerResponse)
    | PublicReplyData (Result Http.Error PI.ServerResponse)
    | LoadUrl String
    | InternalUrl Url
    | SelectedText JD.Value
    | Noop


type WaitMode
    = WmView
    | WmZk Data.Zk
    | WmZkl (List Data.ZkListNote) Data.Zk
    | WmZklm (Maybe (List Data.ZkListNote)) (Maybe Data.FullZkNote) Data.Zk (List Data.ZkListNote -> Data.FullZkNote -> Data.Zk -> State)


type State
    = Login Login.Model
    | EditZk EditZk.Model Data.Login
    | EditZkListing EditZkListing.Model Data.Login
    | EditZkNote EditZkNote.Model Data.Login
    | EditZkNoteListing EditZkNoteListing.Model Data.Login
    | View View.Model
    | EView View.Model State
    | BadError BadError.Model State
    | ShowMessage ShowMessage.Model Data.Login
    | PubShowMessage ShowMessage.Model
    | ZkWait State WaitMode
    | Wait State (State -> Msg -> ( State, Cmd Msg ))


type alias Flags =
    { seed : Int
    , location : String
    , useragent : String
    , debugstring : String
    , width : Int
    , height : Int
    }


type alias Model =
    { state : State
    , size : Util.Size
    , location : String
    , navkey : Browser.Navigation.Key
    , seed : Seed
    }


stateLogin : State -> Maybe Data.Login
stateLogin state =
    case state of
        Login lmod ->
            Just { uid = lmod.userId, pwd = lmod.password }

        EditZk _ login ->
            Just login

        EditZkListing _ login ->
            Just login

        EditZkNote _ login ->
            Just login

        EditZkNoteListing _ login ->
            Just login

        View _ ->
            Nothing

        EView _ evstate ->
            stateLogin evstate

        BadError _ bestate ->
            stateLogin bestate

        ShowMessage _ login ->
            Just login

        PubShowMessage _ ->
            Nothing

        ZkWait bwstate _ ->
            stateLogin bwstate

        Wait wstate _ ->
            stateLogin wstate


viewState : Util.Size -> State -> Element Msg
viewState size state =
    case state of
        Login lem ->
            Element.map LoginMsg <| Login.view size lem

        EditZkListing em _ ->
            Element.map EditZkListingMsg <| EditZkListing.view em

        EditZkNote em _ ->
            Element.map EditZkNoteMsg <| EditZkNote.view em

        EditZkNoteListing em _ ->
            Element.map EditZkNoteListingMsg <| EditZkNoteListing.view em

        ShowMessage em _ ->
            Element.map ShowMessageMsg <| ShowMessage.view em

        PubShowMessage em ->
            Element.map ShowMessageMsg <| ShowMessage.view em

        View em ->
            Element.map ViewMsg <| View.view em

        EView em _ ->
            Element.map ViewMsg <| View.view em

        EditZk em _ ->
            Element.map EditZkMsg <| EditZk.view em

        BadError em _ ->
            Element.map BadErrorMsg <| BadError.view em

        ZkWait innerState _ ->
            Element.map (\_ -> Noop) (viewState size innerState)

        Wait innerState _ ->
            Element.map (\_ -> Noop) (viewState size innerState)


view : Model -> { title : String, body : List (Html Msg) }
view model =
    { title = "mah bloag!"
    , body =
        [ Element.layout [] <|
            viewState model.size model.state
        ]
    }


sendUIMsg : String -> Data.Login -> UI.SendMsg -> Cmd Msg
sendUIMsg location login msg =
    Http.post
        { url = location ++ "/user"
        , body =
            Http.jsonBody
                (UI.encodeSendMsg msg
                    login.uid
                    login.pwd
                )
        , expect = Http.expectJson UserReplyData UI.serverResponseDecoder
        }


sendPIMsg : String -> PI.SendMsg -> Cmd Msg
sendPIMsg location msg =
    Http.post
        { url = location ++ "/public"
        , body =
            Http.jsonBody
                (PI.encodeSendMsg msg)
        , expect = Http.expectJson PublicReplyData PI.serverResponseDecoder
        }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case ( msg, model.state ) of
        ( _, Wait wst wfn ) ->
            let
                ( nst, cmd ) =
                    wfn model.state msg
            in
            ( { model | state = nst }, cmd )

        ( SelectedText jv, state ) ->
            case JD.decodeValue JD.string jv of
                Ok str ->
                    case state of
                        EditZkNote emod login ->
                            let
                                ( s, cmd ) =
                                    EditZkNote.gotSelectedText emod str
                            in
                            case cmd of
                                EditZkNote.Save szk ->
                                    ( { model
                                        | state =
                                            Wait
                                                (ShowMessage
                                                    { message = "waiting for zknote id"
                                                    }
                                                    login
                                                )
                                                (\st ms ->
                                                    -- discard
                                                    case ms of
                                                        UserReplyData (Ok (UI.SavedZkNote _)) ->
                                                            ( EditZkNote s login, Cmd.none )

                                                        _ ->
                                                            ( BadError (BadError.initialModel "unexpected message after zknote save") st, Cmd.none )
                                                )
                                      }
                                    , sendUIMsg model.location
                                        login
                                        (UI.SaveZkNote szk)
                                    )

                                _ ->
                                    ( { model | state = EditZkNote s login }, Cmd.none )

                        _ ->
                            ( model, Cmd.none )

                Err e ->
                    ( { model | state = BadError (BadError.initialModel <| JD.errorToString e) model.state }, Cmd.none )

        ( InternalUrl url, _ ) ->
            let
                mblogin =
                    stateLogin model.state

                ( state, cmd ) =
                    parseUrl url
                        |> Maybe.map
                            (routeState model.location model.seed)
                        |> Maybe.withDefault ( model.state, Cmd.none )
            in
            ( { model | state = state }, cmd )

        ( LoginMsg lm, Login ls ) ->
            let
                ( lmod, lcmd ) =
                    Login.update lm ls
            in
            case lcmd of
                Login.None ->
                    ( { model | state = Login lmod }, Cmd.none )

                Login.Register ->
                    ( { model | state = Login lmod }
                    , sendUIMsg model.location
                        { uid =
                            lmod.userId
                        , pwd =
                            lmod.password
                        }
                        (UI.Register ls.email)
                    )

                Login.Login ->
                    ( { model | state = Login lmod }
                    , sendUIMsg model.location
                        { uid =
                            lmod.userId
                        , pwd =
                            lmod.password
                        }
                        UI.Login
                    )

        ( PublicReplyData prd, state ) ->
            case prd of
                Err e ->
                    ( { model | state = BadError (BadError.initialModel <| Util.httpErrorString e) model.state }, Cmd.none )

                Ok piresponse ->
                    case piresponse of
                        PI.ServerError e ->
                            ( { model | state = BadError (BadError.initialModel e) state }, Cmd.none )

                        PI.ZkNote fbe ->
                            ( { model | state = View (View.initFull fbe) }, Cmd.none )

        ( UserReplyData urd, state ) ->
            case urd of
                Err e ->
                    ( { model | state = BadError (BadError.initialModel <| Util.httpErrorString e) model.state }, Cmd.none )

                Ok uiresponse ->
                    case uiresponse of
                        UI.ServerError e ->
                            ( { model | state = BadError (BadError.initialModel e) state }, Cmd.none )

                        UI.RegistrationSent ->
                            ( model, Cmd.none )

                        UI.LoggedIn ->
                            case state of
                                Login lmod ->
                                    -- we're logged in!  Get article listing.
                                    ( { model
                                        | state =
                                            ShowMessage
                                                { message = "loading articles"
                                                }
                                                { uid = lmod.userId, pwd = lmod.password }
                                        , seed = lmod.seed -- save the seed!
                                      }
                                    , sendUIMsg model.location
                                        { uid =
                                            lmod.userId
                                        , pwd =
                                            lmod.password
                                        }
                                        UI.GetZkListing
                                    )

                                _ ->
                                    ( { model | state = BadError (BadError.initialModel "unexpected login reply") state }
                                    , Cmd.none
                                    )

                        UI.ZkListing l ->
                            case state of
                                ShowMessage _ login ->
                                    ( { model | state = EditZkListing { zks = l } login }, Cmd.none )

                                _ ->
                                    ( { model | state = BadError (BadError.initialModel "unexpected login reply") state }
                                    , Cmd.none
                                    )

                        UI.ZkNoteListing l ->
                            case state of
                                ZkWait zwstate wm ->
                                    case ( wm, stateLogin zwstate ) of
                                        ( WmZk zk, Just login ) ->
                                            ( { model | state = EditZkNoteListing { zk = zk, notes = l } login }, Cmd.none )

                                        ( WmZklm Nothing mbzkn zk tostate, Just login ) ->
                                            case mbzkn of
                                                Just zkn ->
                                                    ( { model | state = tostate l zkn zk }
                                                    , Cmd.none
                                                    )

                                                Nothing ->
                                                    ( { model | state = ZkWait zwstate (WmZklm (Just l) mbzkn zk tostate) }, Cmd.none )

                                        _ ->
                                            ( { model | state = BadError (BadError.initialModel "unexpected reply") state }
                                            , Cmd.none
                                            )

                                _ ->
                                    ( { model | state = BadError (BadError.initialModel "unexpected zknote listing") state }
                                    , Cmd.none
                                    )

                        UI.ZkNote zkn ->
                            case state of
                                EditZkListing _ login ->
                                    -- ( { model | state = EditZkNote (EditZkNote.initFull zkn) login }, Cmd.none )
                                    ( { model | state = BadError (BadError.initialModel "zknoteeditunimplmeented") state }
                                    , Cmd.none
                                    )

                                ZkWait bwstate mode ->
                                    case mode of
                                        WmView ->
                                            ( { model | state = EView (View.initFull zkn) bwstate }, Cmd.none )

                                        WmZk zk ->
                                            ( { model | state = BadError (BadError.initialModel "can't edit - no zklist!") state }, Cmd.none )

                                        WmZklm mbzkl mbzkn zk tostate ->
                                            case mbzkl of
                                                Just zkl ->
                                                    ( { model | state = tostate zkl zkn zk }, Cmd.none )

                                                Nothing ->
                                                    ( { model | state = ZkWait bwstate (WmZklm mbzkl (Just zkn) zk tostate) }, Cmd.none )

                                        WmZkl zkl zk ->
                                            case stateLogin state of
                                                Just login ->
                                                    ( { model | state = EditZkNote (EditZkNote.initFull zk zkl zkn) login }, Cmd.none )

                                                Nothing ->
                                                    ( { model | state = BadError (BadError.initialModel "can't edit - not logged in!") state }, Cmd.none )

                                -- ( { model | state = BadError (BadError.initialModel "unexpected message") bwstate }, Cmd.none )
                                _ ->
                                    ( { model | state = BadError (BadError.initialModel <| "unexpected message: zknote") state }, Cmd.none )

                        UI.SavedZk beid ->
                            case state of
                                EditZk emod login ->
                                    ( { model | state = EditZk (EditZk.setId emod beid) login }, Cmd.none )

                                _ ->
                                    ( { model | state = BadError (BadError.initialModel "unexpected message: savedzk") state }, Cmd.none )

                        UI.DeletedZk beid ->
                            case state of
                                ShowMessage _ login ->
                                    ( model
                                    , sendUIMsg model.location login UI.GetZkListing
                                    )

                                _ ->
                                    ( { model | state = BadError (BadError.initialModel "unexpected message") state }, Cmd.none )

                        UI.SavedZkNote beid ->
                            case state of
                                EditZkNote emod login ->
                                    ( { model | state = EditZkNote (EditZkNote.gotId emod beid) login }, Cmd.none )

                                _ ->
                                    -- just ignore if we're not editing a new note.
                                    ( model, Cmd.none )

                        UI.DeletedZkNote beid ->
                            ( model, Cmd.none )

                        UI.UserExists ->
                            ( { model | state = BadError (BadError.initialModel "Can't register - User exists already!") state }, Cmd.none )

                        UI.UnregisteredUser ->
                            ( { model | state = BadError (BadError.initialModel "Unregistered user.  Check your spam folder!") state }, Cmd.none )

                        UI.InvalidUserOrPwd ->
                            ( { model | state = BadError (BadError.initialModel "Invalid username or password.") state }, Cmd.none )

        ( ViewMsg em, View es ) ->
            let
                ( emod, ecmd ) =
                    View.update em es
            in
            case ecmd of
                View.None ->
                    ( { model | state = View emod }, Cmd.none )

                View.Done ->
                    ( { model | state = View emod }, Cmd.none )

        ( ViewMsg em, EView es state ) ->
            let
                ( emod, ecmd ) =
                    View.update em es
            in
            case ecmd of
                View.None ->
                    ( { model | state = EView emod state }, Cmd.none )

                View.Done ->
                    ( { model | state = state }, Cmd.none )

        ( EditZkMsg em, EditZk es login ) ->
            let
                ( emod, ecmd ) =
                    EditZk.update em es
            in
            case ecmd of
                EditZk.Save zk ->
                    ( { model | state = EditZk emod login }
                    , sendUIMsg model.location
                        login
                        (UI.SaveZk zk)
                    )

                EditZk.None ->
                    ( { model | state = EditZk emod login }, Cmd.none )

                EditZk.Done ->
                    ( { model
                        | state =
                            ShowMessage
                                { message = "loading articles"
                                }
                                login
                      }
                    , sendUIMsg model.location
                        login
                        UI.GetZkListing
                    )

                EditZk.Delete id ->
                    -- issue delete and go back to listing.
                    ( { model
                        | state =
                            ShowMessage
                                { message = "loading articles"
                                }
                                login
                      }
                    , sendUIMsg model.location
                        login
                        (UI.DeleteZk id)
                    )

                EditZk.View sbe ->
                    ( { model | state = BadError (BadError.initialModel "EditZk.View sbe -> unimplmeented") model.state }
                    , Cmd.none
                    )

        ( EditZkNoteMsg em, EditZkNote es login ) ->
            let
                ( emod, ecmd ) =
                    EditZkNote.update em es

                backtolisting =
                    ( { model
                        | state =
                            ZkWait
                                (ShowMessage
                                    { message = "loading articles"
                                    }
                                    login
                                )
                                (WmZk emod.zk)
                      }
                    , sendUIMsg model.location
                        login
                        (UI.GetZkNoteListing
                            es.zk.id
                        )
                    )
            in
            case ecmd of
                EditZkNote.SaveExit szk ->
                    ( { model
                        | state =
                            Wait
                                (ShowMessage
                                    { message = "loading articles"
                                    }
                                    login
                                )
                                (\st ms ->
                                    case ms of
                                        UserReplyData (Ok (UI.SavedZkNote id)) ->
                                            ( st
                                            , sendUIMsg model.location
                                                login
                                                (UI.GetZkNoteListing
                                                    es.zk.id
                                                )
                                            )

                                        UserReplyData (Ok (UI.ZkNoteListing l)) ->
                                            ( EditZkNoteListing { zk = es.zk, notes = l } login, Cmd.none )

                                        UserReplyData (Ok (UI.ServerError e)) ->
                                            ( BadError (BadError.initialModel e) st, Cmd.none )

                                        _ ->
                                            ( BadError (BadError.initialModel "unexpected message!") model.state
                                            , Cmd.none
                                            )
                                )
                      }
                    , sendUIMsg model.location
                        login
                        (UI.SaveZkNote szk)
                    )

                EditZkNote.Save szk ->
                    ( { model | state = EditZkNote emod login }
                    , sendUIMsg model.location
                        login
                        (UI.SaveZkNote szk)
                    )

                EditZkNote.None ->
                    ( { model | state = EditZkNote emod login }, Cmd.none )

                EditZkNote.Revert ->
                    backtolisting

                EditZkNote.Delete id ->
                    -- issue delete and go back to listing.
                    let
                        ( m, c ) =
                            backtolisting
                    in
                    ( { m
                        | state =
                            Wait m.state
                                (\state _ ->
                                    ( m.state, c )
                                )
                      }
                    , sendUIMsg model.location
                        login
                        (UI.DeleteZkNote id)
                    )

                EditZkNote.Switch id ->
                    ( { model
                        | state =
                            ZkWait
                                (ShowMessage
                                    { message = "loading articles"
                                    }
                                    login
                                )
                                (WmZklm Nothing Nothing emod.zk (\zkl zkn zk -> EditZkNote (EditZkNote.initFull zk zkl zkn) login))
                      }
                    , Cmd.batch
                        [ sendUIMsg model.location
                            login
                            (UI.GetZkNoteListing
                                es.zk.id
                            )
                        , sendUIMsg model.location
                            login
                            (UI.GetZkNote id)
                        ]
                    )

                EditZkNote.SaveSwitch szkn id ->
                    ( { model
                        | state =
                            ZkWait
                                (ShowMessage
                                    { message = "loading articles"
                                    }
                                    login
                                )
                                (WmZklm Nothing Nothing emod.zk (\zkl zkn zk -> EditZkNote (EditZkNote.initFull zk zkl zkn) login))
                      }
                    , Cmd.batch
                        [ sendUIMsg model.location
                            login
                            (UI.GetZkNoteListing
                                es.zk.id
                            )
                        , sendUIMsg model.location
                            login
                            (UI.GetZkNote id)
                        , sendUIMsg model.location
                            login
                            (UI.SaveZkNote szkn)
                        ]
                    )

                EditZkNote.View sbe ->
                    ( { model | state = BadError (BadError.initialModel "EditZkNote.View sbe -> unimplmeented") model.state }
                    , Cmd.none
                    )

                EditZkNote.GetSelectedText id ->
                    ( { model | state = EditZkNote emod login }
                    , getSelectedText (Just id)
                    )

        ( EditZkListingMsg em, EditZkListing es login ) ->
            let
                ( emod, ecmd ) =
                    EditZkListing.update em es
            in
            case ecmd of
                EditZkListing.New ->
                    ( { model | state = EditZk EditZk.initNew login }, Cmd.none )

                EditZkListing.Example ->
                    ( { model | state = EditZk EditZk.initExample login }, Cmd.none )

                EditZkListing.Selected zk ->
                    ( { model | state = EditZk (EditZk.initFull zk) login }, Cmd.none )

                EditZkListing.Notes zk ->
                    ( { model
                        | state =
                            ZkWait
                                (ShowMessage
                                    { message = "loading articles"
                                    }
                                    login
                                )
                                (WmZk zk)
                      }
                    , sendUIMsg model.location
                        login
                        (UI.GetZkNoteListing zk.id)
                    )

                EditZkListing.View id ->
                    ( { model | state = ZkWait model.state WmView }
                    , sendUIMsg model.location
                        login
                        (UI.GetZk id)
                    )

        ( EditZkNoteListingMsg em, EditZkNoteListing es login ) ->
            let
                ( emod, ecmd ) =
                    EditZkNoteListing.update em es
            in
            case ecmd of
                EditZkNoteListing.New ->
                    ( { model | state = EditZkNote (EditZkNote.initNew emod.zk es.notes) login }, Cmd.none )

                EditZkNoteListing.Example ->
                    ( { model | state = EditZkNote (EditZkNote.initExample emod.zk es.notes) login }, Cmd.none )

                EditZkNoteListing.Selected id ->
                    ( { model
                        | state =
                            ZkWait (ShowMessage { message = "loading zknote" } login)
                                (WmZkl emod.notes emod.zk)
                      }
                    , sendUIMsg model.location
                        login
                        (UI.GetZkNote id)
                    )

                EditZkNoteListing.View id ->
                    ( { model | state = ZkWait model.state WmView }
                    , sendUIMsg model.location
                        login
                        (UI.GetZk id)
                    )

                EditZkNoteListing.Done ->
                    -- back to the Zk listing.
                    ( { model
                        | state =
                            ShowMessage
                                { message = "loading zk listing"
                                }
                                login
                      }
                    , sendUIMsg model.location
                        login
                        UI.GetZkListing
                    )

        ( BadErrorMsg bm, BadError bs prevstate ) ->
            let
                ( bmod, bcmd ) =
                    BadError.update bm bs
            in
            case bcmd of
                BadError.Okay ->
                    ( { model | state = prevstate }, Cmd.none )

        ( _, _ ) ->
            ( model, Cmd.none )


init : Flags -> Url -> Browser.Navigation.Key -> ( Model, Cmd Msg )
init flags url key =
    let
        seed =
            initialSeed (flags.seed + 7)

        ( state, cmd ) =
            parseUrl url
                |> Maybe.map (routeState flags.location seed)
                |> Maybe.withDefault ( initLogin seed, Cmd.none )
    in
    ( { state = state
      , size = { width = flags.width, height = flags.height }
      , location = flags.location
      , navkey = key
      , seed = seed
      }
    , cmd
    )


type Route
    = PublicZk Int
    | Fail


parseUrl : Url -> Maybe Route
parseUrl url =
    UP.parse
        (UP.map (\i -> PublicZk i) <|
            UP.s
                "note"
                </> UP.int
        )
        url


initLogin : Seed -> State
initLogin seed =
    Login <| Login.initialModel Nothing "mahbloag" seed


routeState : String -> Seed -> Route -> ( State, Cmd Msg )
routeState location seed route =
    case route of
        PublicZk id ->
            ( PubShowMessage
                { message = "loading article"
                }
            , sendPIMsg location
                (PI.GetZkNote id)
            )

        Fail ->
            ( initLogin seed, Cmd.none )


urlRequest : Browser.UrlRequest -> Msg
urlRequest ur =
    case ur of
        Browser.Internal url ->
            InternalUrl url

        Browser.External str ->
            LoadUrl str


main : Platform.Program Flags Model Msg
main =
    Browser.application
        { init = init
        , view = view
        , update = update
        , subscriptions =
            \_ -> receiveSelectedText SelectedText
        , onUrlRequest = urlRequest
        , onUrlChange =
            \uc ->
                Noop
        }


port getSelectedText : Maybe String -> Cmd msg


port receiveSelectedText : (JD.Value -> msg) -> Sub msg
