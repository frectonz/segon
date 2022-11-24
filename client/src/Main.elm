port module Main exposing (..)

import Browser
import Browser.Navigation as Nav
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (onClick, onInput)
import Http
import Json.Decode as Decode exposing (Decoder, field)
import Json.Encode as Encode
import List.Extra exposing (zip)
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


port sendAnswerToGameServer : Encode.Value -> Cmd msg



-- MODEL


type alias Model =
    { state : AuthenticationState }


type AuthenticationState
    = LoggedIn { token : String, serverMessage : ServerMessage }
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
    | GotRegisterResult (Result Http.Error AuthResponse)
    | GotLoginResult (Result Http.Error AuthResponse)
    | GotGameServerMessage Decode.Value
    | Tick Time.Posix
    | SendAnswer String


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
                    ( Model (LoggedIn { token = token, serverMessage = Unknown }), connectToGameServer token )

                Err _ ->
                    ( model, Cmd.none )

        GotLoginResult result ->
            case result of
                Ok { token } ->
                    ( Model (LoggedIn { token = token, serverMessage = Unknown }), connectToGameServer token )

                Err _ ->
                    ( model, Cmd.none )

        GotGameServerMessage val ->
            case ( model.state, Decode.decodeValue decodeServerMessage val ) of
                ( LoggedIn m, Ok serverMessage ) ->
                    ( Model (LoggedIn { m | serverMessage = serverMessage }), Cmd.none )

                _ ->
                    ( model, Cmd.none )

        Tick _ ->
            case model.state of
                LoggedIn m ->
                    ( Model
                        (LoggedIn
                            { m
                                | serverMessage =
                                    case m.serverMessage of
                                        TimeTillGame time ->
                                            if time > 0 then
                                                TimeTillGame (time - 1)

                                            else
                                                TimeTillGame 0

                                        _ ->
                                            m.serverMessage
                            }
                        )
                    , Cmd.none
                    )

                _ ->
                    ( model, Cmd.none )

        SendAnswer answer ->
            case model.state of
                LoggedIn _ ->
                    let
                        encodedAnswer =
                            Encode.object
                                [ ( "answer_idx", Encode.string answer ) ]
                    in
                    ( model, sendAnswerToGameServer encodedAnswer )

                _ ->
                    ( model, Cmd.none )



-- DECODERS


type ServerMessage
    = Unknown
    | TimeTillGame Int
    | GameStart
    | GotQuestion Question
    | GotAnswer Answer
    | GameEnd Int


type alias Question =
    { question : String
    , options : List String
    }


type alias Answer =
    { status : String
    , answer : String
    }


decodeServerMessage : Decoder ServerMessage
decodeServerMessage =
    field "type" Decode.string
        |> Decode.andThen
            (\msgType ->
                case msgType of
                    "TimeTillGame" ->
                        field "time" Decode.int
                            |> Decode.map TimeTillGame

                    "GameStart" ->
                        Decode.succeed GameStart

                    "Question" ->
                        Decode.map2 Question
                            (field "question" Decode.string)
                            (field "options" (Decode.list Decode.string))
                            |> Decode.map GotQuestion

                    "Answer" ->
                        Decode.map2 Answer
                            (field "status" Decode.string)
                            (field "answer_idx" Decode.string)
                            |> Decode.map GotAnswer

                    "GameEnd" ->
                        field "score" Decode.int
                            |> Decode.map GameEnd

                    _ ->
                        Decode.fail "Unknown message type"
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
                LoggedIn { serverMessage } ->
                    case serverMessage of
                        TimeTillGame _ ->
                            Time.every 1000 Tick

                        _ ->
                            Sub.none

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


viewLoggedIn : { a | token : String, serverMessage : ServerMessage } -> Html Msg
viewLoggedIn { token, serverMessage } =
    div []
        [ h1 [] [ text "Logged in" ]
        , pre [] [ text token ]
        , div []
            (case serverMessage of
                Unknown ->
                    [ "Unknown" |> text ]

                TimeTillGame time ->
                    [ "Waiting " ++ String.fromInt time |> text ]

                GameStart ->
                    [ "Game started" |> text ]

                GotQuestion { question, options } ->
                    [ text question
                    , div []
                        (options
                            |> zip [ "One", "Two", "Three", "Four" ]
                            |> List.map
                                (\( idx, opt ) ->
                                    button [ onClick (SendAnswer idx) ] [ text opt ]
                                )
                        )
                    ]

                GotAnswer { status, answer } ->
                    [ p [] [ "Answer status: " ++ status |> text ]
                    , p [] [ "Answer index: " ++ answer |> text ]
                    ]

                GameEnd score ->
                    [ "Game ended. Score: " ++ String.fromInt score |> text ]
            )
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
