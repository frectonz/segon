module Main exposing (..)

import Browser
import Browser.Navigation as Nav
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (onClick, onInput)
import Http
import Json.Decode as Decode exposing (Decoder, field)
import Json.Encode as Encode
import Url



-- MAIN


main : Program () Model Msg
main =
    Browser.application
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        , onUrlChange = \_ -> NoOp
        , onUrlRequest = \_ -> NoOp
        }



-- MODEL


type alias Model =
    { state : AuthenticationState }


type AuthenticationState
    = LoggedIn
    | LoggedOut
        { username : String
        , password : String
        }


init : () -> Url.Url -> Nav.Key -> ( Model, Cmd Msg )
init _ _ _ =
    ( Model (LoggedOut { username = "", password = "" }), Cmd.none )



-- UPDATE


type Msg
    = NoOp
    | UpadteUsername String
    | UpdatePassword String
    | Login
    | Register
    | GotRegisterResult (Result Http.Error RegisterResponse)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        NoOp ->
            ( model, Cmd.none )

        UpadteUsername newUsername ->
            case model.state of
                LoggedOut m ->
                    ( Model (LoggedOut { m | username = newUsername }), Cmd.none )

                _ ->
                    ( model, Cmd.none )

        UpdatePassword newPassword ->
            case model.state of
                LoggedOut m ->
                    ( Model (LoggedOut { m | password = newPassword }), Cmd.none )

                _ ->
                    ( model, Cmd.none )

        Login ->
            ( model, Cmd.none )

        Register ->
            case model.state of
                LoggedOut m ->
                    ( model, register m.username m.password )

                _ ->
                    ( model, Cmd.none )

        GotRegisterResult result ->
            ( model, Cmd.none )



-- COMMANDS


register : String -> String -> Cmd Msg
register username password =
    let
        encode =
            Encode.object
                [ ( "username", Encode.string username )
                , ( "password", Encode.string password )
                ]
    in
    Http.post
        { url = "http://localhost:3030/register"
        , body = Http.jsonBody encode
        , expect = Http.expectJson GotRegisterResult decodeRegisterResponse
        }


type alias RegisterResponse =
    { status : String
    , token : String
    }


decodeRegisterResponse : Decoder RegisterResponse
decodeRegisterResponse =
    Decode.map2 RegisterResponse
        (field "success" Decode.string)
        (field "token" Decode.string)



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.none



-- VIEW


view : Model -> Browser.Document Msg
view model =
    { title = "Segon"
    , body =
        [ case model.state of
            LoggedIn ->
                viewLoggedIn

            LoggedOut m ->
                viewLoggedOut m
        ]
    }


viewLoggedIn : Html Msg
viewLoggedIn =
    div []
        [ h1 [] [ text "Logged out" ]
        ]


viewLoggedOut : { a | username : String, password : String } -> Html Msg
viewLoggedOut { username, password } =
    div []
        [ h1 [] [ text "Logged out" ]
        , input [ onInput UpadteUsername, value username ] []
        , br [] []
        , input [ onInput UpdatePassword, value password ] []
        , br [] []
        , button [ onClick Login ] [ text "Login" ]
        , button [ onClick Register ] [ text "Register" ]
        ]
