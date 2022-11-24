port module Main exposing (..)

import Browser
import Browser.Dom exposing (Error(..))
import Browser.Navigation as Nav
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (onClick, onInput)
import Http
import Json.Decode as Decode exposing (Decoder, field)
import Json.Encode as Encode
import Time
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



-- PORTS


port connectToGameServer : String -> Cmd msg


port receiveGameServerMessage : (Decode.Value -> msg) -> Sub msg



-- MODEL


type alias Model =
    { state : AuthenticationState }


type AuthenticationState
    = LoggedIn { token : String, gameState : GameState }
    | LoggedOut
        { username : String
        , password : String
        }


type GameState
    = Unknown
    | Waiting Int
    | GameStarted


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
    | GotRegisterResult (Result Http.Error AuthResponse)
    | GotLoginResult (Result Http.Error AuthResponse)
    | GotGameServerMessage Decode.Value
    | Tick Time.Posix


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
            case model.state of
                LoggedOut m ->
                    ( model, login m.username m.password )

                _ ->
                    ( model, Cmd.none )

        Register ->
            case model.state of
                LoggedOut m ->
                    ( model, register m.username m.password )

                _ ->
                    ( model, Cmd.none )

        GotRegisterResult result ->
            case result of
                Ok { token } ->
                    ( Model (LoggedIn { token = token, gameState = Unknown }), connectToGameServer token )

                Err _ ->
                    ( model, Cmd.none )

        GotLoginResult result ->
            case result of
                Ok { token } ->
                    ( Model (LoggedIn { token = token, gameState = Unknown }), connectToGameServer token )

                Err _ ->
                    ( model, Cmd.none )

        GotGameServerMessage val ->
            case ( Decode.decodeValue decodeTimeTillGame val, model.state ) of
                ( Ok (TimeTillGame time), LoggedIn m ) ->
                    ( Model (LoggedIn { m | gameState = Waiting time }), Cmd.none )

                _ ->
                    ( model, Cmd.none )

        Tick _ ->
            case model.state of
                LoggedIn m ->
                    ( Model
                        (LoggedIn
                            { m
                                | gameState =
                                    case m.gameState of
                                        Waiting time ->
                                            if time > 0 then
                                                Waiting (time - 1)

                                            else
                                                GameStarted

                                        _ ->
                                            m.gameState
                            }
                        )
                    , Cmd.none
                    )

                _ ->
                    ( model, Cmd.none )



-- DECODERS


type ServerMessage
    = TimeTillGame Int
    | GameStart
    | GotQuestion Question


type alias Question =
    { question : String
    , options : List String
    }


decodeServerMessage : Decoder ServerMessage
decodeServerMessage =
    Decode.oneOf
        [ decodeTimeTillGame
        , decodeGameStart
        ]


decodeTimeTillGame : Decoder ServerMessage
decodeTimeTillGame =
    field "type" (Decode.succeed "TimeTillGame")
        |> Decode.andThen
            (\_ ->
                Decode.map TimeTillGame
                    (field "time" Decode.int)
            )


decodeGameStart : Decoder ServerMessage
decodeGameStart =
    field "type" (Decode.succeed "GameStart")
        |> Decode.andThen
            (\_ ->
                Decode.succeed GameStart
            )


decodeQuestion : Decoder ServerMessage
decodeQuestion =
    field "type" (Decode.succeed "Question")
        |> Decode.andThen
            (\_ ->
                Decode.map GotQuestion
                    (Decode.map2 Question
                        (field "question" Decode.string)
                        (field "options" (Decode.list Decode.string))
                    )
            )



-- COMMANDS


register : String -> String -> Cmd Msg
register username password =
    Http.post
        { url = "/register"
        , body =
            encodeUsernameAndPassword
                { username = username, password = password }
                |> Http.jsonBody
        , expect = Http.expectJson GotRegisterResult decodeAuthResponse
        }


login : String -> String -> Cmd Msg
login username password =
    Http.post
        { url = "/login"
        , body =
            encodeUsernameAndPassword
                { username = username, password = password }
                |> Http.jsonBody
        , expect = Http.expectJson GotLoginResult decodeAuthResponse
        }


encodeUsernameAndPassword : { username : String, password : String } -> Encode.Value
encodeUsernameAndPassword { username, password } =
    Encode.object
        [ ( "username", Encode.string username )
        , ( "password", Encode.string password )
        ]


type alias AuthResponse =
    { status : String
    , token : String
    }


decodeAuthResponse : Decoder AuthResponse
decodeAuthResponse =
    Decode.map2 AuthResponse
        (field "status" Decode.string)
        (field "token" Decode.string)



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions { state } =
    let
        ticker =
            case state of
                LoggedIn _ ->
                    Time.every 1000 Tick

                _ ->
                    Sub.none

        subs =
            [ receiveGameServerMessage GotGameServerMessage, ticker ]
    in
    Sub.batch subs



-- VIEW


view : Model -> Browser.Document Msg
view model =
    { title = "Segon"
    , body =
        [ case model.state of
            LoggedIn m ->
                viewLoggedIn m

            LoggedOut m ->
                viewLoggedOut m
        ]
    }


viewLoggedIn : { a | token : String, gameState : GameState } -> Html msg
viewLoggedIn { token, gameState } =
    div []
        [ h1 [] [ text "Logged in" ]
        , p [] [ pre [] [ text token ] ]
        , p []
            [ text
                (case gameState of
                    Unknown ->
                        "Unknown"

                    Waiting time ->
                        "Waiting " ++ String.fromInt time

                    GameStarted ->
                        "Game started"
                )
            ]
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
