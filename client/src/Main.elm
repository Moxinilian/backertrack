module Main exposing (Model, Msg(..), init, main)

import Auth exposing (..)
import Browser
import Browser.Navigation as Nav
import Html exposing (..)
import Html.Attributes exposing (..)
import Http
import Json.Decode as D
import Ledger exposing (..)
import Loading exposing (loadingView)
import Maybe exposing (Maybe(..))
import Panic exposing (Panic(..), panicView)
import Url


type alias Model =
    { apiServer : String
    , key : Nav.Key
    , currentCategory : Category
    , pages : PageStates
    , panic : Panic
    , auth : Maybe AuthLevel
    , settings : Maybe SettingsData
    }


type alias PageStates =
    { ledgerState : LedgerState }


defaultPageStates : PageStates
defaultPageStates =
    PageStates defaultLedgerPage


type alias SettingsData =
    { defaultPermissions : Permissions
    }


type Category
    = Home
    | Ledger
    | Budget
    | Bounties
    | Settings


type Msg
    = LinkClicked Browser.UrlRequest
    | UrlChanged Url.Url
    | ObtainedSettings (Result Http.Error SettingsData)


main : Program D.Value Model Msg
main =
    Browser.application
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        , onUrlChange = UrlChanged
        , onUrlRequest = LinkClicked
        }


type alias JavaScriptFlags =
    { apiServer : String
    }


flagsDecoder : D.Decoder JavaScriptFlags
flagsDecoder =
    D.map JavaScriptFlags (D.field "apiServer" D.string)


init : D.Value -> Url.Url -> Nav.Key -> ( Model, Cmd Msg )
init flagsRaw url key =
    case D.decodeValue flagsDecoder flagsRaw of
        Err _ ->
            ( Model "" key Home defaultPageStates (Reason "Invalid JavaScript flags") Nothing Nothing, Cmd.none )

        Ok flags ->
            let
                model =
                    Model flags.apiServer key Ledger defaultPageStates None Nothing Nothing
            in
            ( model, fetchSettings model.apiServer )


fetchSettings : String -> Cmd Msg
fetchSettings server =
    Http.get
        { url = server ++ "/settings.json"
        , expect = Http.expectJson ObtainedSettings jsonToSettings
        }


jsonToSettings : D.Decoder SettingsData
jsonToSettings =
    D.map SettingsData (D.field "defaultPermissions" jsonToPermissions)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        LinkClicked urlRequest ->
            case urlRequest of
                Browser.Internal url ->
                    ( model, Nav.pushUrl model.key (Url.toString url) )

                Browser.External href ->
                    ( model, Nav.load href )

        UrlChanged url ->
            ( model, Cmd.none )

        ObtainedSettings res ->
            case res of
                Err e ->
                    ( { model | panic = Reason ("Could not fetch basic settings at " ++ model.apiServer ++ "/settings.json, " ++ Debug.toString e) }, Cmd.none )

                Ok data ->
                    refreshGeneralData (updateSettings model data)


updateSettings : Model -> SettingsData -> Model
updateSettings model data =
    { model | settings = Just data, auth = Just (Maybe.withDefault (AuthLevel "Anonymous" data.defaultPermissions False) model.auth) }


refreshGeneralData : Model -> ( Model, Cmd Msg )
refreshGeneralData model =
    ( model, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.none


view : Model -> Browser.Document Msg
view model =
    { title = "backertrack"
    , body =
        if model.panic == None then
            if model.settings == Nothing then
                [ loadingView ]

            else
                [ navbarView model
                , div [ class "container" ]
                    [ case model.currentCategory of
                        Ledger ->
                            ledgerView model.pages.ledgerState

                        _ ->
                            div [] [ text "Content" ]
                    ]
                ]

        else
            panicView model.panic
    }


navbarView : Model -> Html Msg
navbarView model =
    div [ class "navbar top-navbar" ]
        [ div [ class "container" ]
            [ div [ class "navbar-brand" ]
                [ div
                    [ class "navbar-item" ]
                    [ b [] [ text "backertrack" ]
                    ]
                , div [ class "navbar-burger" ]
                    [ span [] []
                    , span [] []
                    , span [] []
                    ]
                ]
            , div [ class "navbar-menu" ]
                [ div [ class "navbar-start" ] [ div [ class "navbar-item" ] (navbarCategories model) ]
                , div [ class "navbar-end" ]
                    [ div [ class "navbar-item" ] (navbarAuth model)
                    ]
                ]
            ]
        ]


navbarButtonClass : Model -> Category -> Attribute msg
navbarButtonClass model cat =
    if model.currentCategory == cat then
        class "navbar-item is-active"

    else
        class "navbar-item"


navbarCategories : Model -> List (Html msg)
navbarCategories model =
    [ a [ navbarButtonClass model Home, href "#" ] [ text "Home" ]
    , a [ navbarButtonClass model Ledger, href "#" ] [ text "Ledger" ]
    , a [ navbarButtonClass model Budget, href "#" ] [ text "Budget" ]
    , a [ navbarButtonClass model Bounties, href "#" ] [ text "Bounties" ]
    , a [ navbarButtonClass model Settings, href "#" ] [ text "Settings" ]
    ]


navbarAuth : Model -> List (Html msg)
navbarAuth model =
    [ div [ class "button" ]
        [ text "Sign In"
        ]
    ]
