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


port sendGameServerMessage : Encode.Value -> Cmd msg


port confetti : () -> Cmd msg



-- MODEL


type alias Model =
    { state : AuthenticationState }


type AuthenticationState
    = LoggedIn
        { token : String
        , serverMessage : ServerMessage
        , answer : Maybe String
        , lastQuestion : Maybe Question
        }
    | LoggedOut
        { username : String
        , password : String
        }


init : () -> Url.Url -> Nav.Key -> ( Model, Cmd Msg )
init _ _ _ =
    ( Model
        (LoggedOut
            { username = ""
            , password = ""
            }
        )
    , Cmd.none
    )



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
                    ( Model
                        (LoggedIn
                            { token = token
                            , serverMessage = Unknown
                            , answer = Nothing
                            , lastQuestion = Nothing
                            }
                        )
                    , Cmd.batch [ connectToGameServer token, confetti () ]
                    )

                Err _ ->
                    ( model, Cmd.none )

        GotLoginResult result ->
            case result of
                Ok { token } ->
                    ( Model
                        (LoggedIn
                            { token = token
                            , serverMessage = Unknown
                            , answer = Nothing
                            , lastQuestion = Nothing
                            }
                        )
                    , Cmd.batch [ connectToGameServer token, confetti () ]
                    )

                Err _ ->
                    ( model, Cmd.none )

        GotGameServerMessage val ->
            case ( model.state, Decode.decodeValue decodeServerMessage val ) of
                ( LoggedIn m, Ok serverMessage ) ->
                    let
                        lastQuestion =
                            case serverMessage of
                                GotQuestion q ->
                                    Just q

                                _ ->
                                    m.lastQuestion

                        answer =
                            case serverMessage of
                                GotQuestion _ ->
                                    Nothing

                                _ ->
                                    m.answer
                    in
                    ( Model
                        (LoggedIn
                            { m
                                | serverMessage = serverMessage
                                , lastQuestion = lastQuestion
                                , answer = answer
                            }
                        )
                    , Cmd.none
                    )

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
                LoggedIn m ->
                    let
                        encodedAnswer =
                            Encode.object
                                [ ( "type", Encode.string "Answer" )
                                , ( "answer_idx", Encode.string answer )
                                ]
                    in
                    ( Model (LoggedIn { m | answer = Just answer }), sendGameServerMessage encodedAnswer )

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
        { url = "http://localhost:3030/register"
        , body =
            encodeUsernameAndPassword
                { username = username, password = password }
                |> Http.jsonBody
        , expect = Http.expectJson GotRegisterResult decodeAuthResponse
        }


login : String -> String -> Cmd Msg
login username password =
    Http.post
        { url = "http://localhost:3030/login"
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


viewLoggedIn : { a | serverMessage : ServerMessage, answer : Maybe String, lastQuestion : Maybe Question } -> Html Msg
viewLoggedIn { serverMessage, answer, lastQuestion } =
    div [ class "w-screen h-screen flex flex-col gap-4 items-center justify-center" ]
        [ div []
            (case serverMessage of
                Unknown ->
                    []

                TimeTillGame time ->
                    [ div
                        [ class "text-center bg-transparent shadow-lg rounded-full text-white animate-pulse border-2 border-fuchsia-800 w-[200px] h-[200px] flex justify-center items-center flex-col"
                        ]
                        [ p [ class "text-[4rem]" ] [ String.fromInt time |> text ]
                        , p [] [ text "seconds till game" ]
                        ]
                    ]

                GameStart ->
                    [ h1 [ class "text-3xl p-4 text-center text-white drop-shadow-xl" ] [ text "Game will begin shortly..." ] ]

                GotQuestion { question, options } ->
                    [ p [ class "w-[90vw] text-5xl mb-10 text-white break-words text-center" ] [ text question ]
                    , div [ class "w-[80%] flex flex-col gap-4 mx-auto" ]
                        (options
                            |> zip [ "One", "Two", "Three", "Four" ]
                            |> List.map
                                (\( idx, opt ) ->
                                    button
                                        [ onClick (SendAnswer idx)
                                        , type_ "button"
                                        , class
                                            ("bg-white px-6 py-4 rounded-lg drop-shadow-lg cursor-pointer active:scale-90 transition-all"
                                                ++ Maybe.withDefault ""
                                                    (Maybe.map
                                                        (\userAns ->
                                                            if userAns == idx then
                                                                " bg-yellow-500 text-white"

                                                            else
                                                                ""
                                                        )
                                                        answer
                                                    )
                                            )
                                        ]
                                        [ text opt ]
                                )
                        )
                    ]

                GotAnswer serverAns ->
                    let
                        q =
                            lastQuestion
                                |> Maybe.withDefault
                                    { question = ""
                                    , options = [ "", "", "", "" ]
                                    }

                        userAns =
                            answer |> Maybe.withDefault ""
                    in
                    [ p [ class "w-[90vw] text-5xl mb-10 text-white break-words text-center" ] [ text q.question ]
                    , div [ class "w-[80%] flex flex-col gap-4 mx-auto" ]
                        (q.options
                            |> zip [ "One", "Two", "Three", "Four" ]
                            |> List.map
                                (\( idx, opt ) ->
                                    button
                                        [ type_ "button"
                                        , class
                                            ("bg-white px-6 py-4 rounded-lg drop-shadow-lg cursor-pointer active:scale-90 transition-all text-white"
                                                ++ (if serverAns.answer == idx then
                                                        " bg-green-500"

                                                    else if userAns == idx then
                                                        " bg-yellow-500"

                                                    else
                                                        " bg-gray-500"
                                                   )
                                            )
                                        ]
                                        [ text opt ]
                                )
                        )
                    ]

                GameEnd score ->
                    [ h1 [ class "text-3xl p-4 text-center text-white drop-shadow-xl" ]
                        [ "Game ended. Score: " ++ String.fromInt score |> text ]
                    ]
            )
        ]


viewLoggedOut : { a | username : String, password : String } -> Html Msg
viewLoggedOut { username, password } =
    div [ class "w-screen h-screen grid place-items-center" ]
        [ div [ class "flex flex-col gap-2" ]
            [ h1 [ class "text-8xl text-white drop-shadow-xl font-bold -center mb-10" ] [ text "SEGON" ]
            , input
                [ onInput UpadteUsername
                , value username
                , type_ "text"
                , class "bg-transparent text-white accent-white placeholder-white text-xl p-2 rounded-md shadow-md mb-5 "
                , placeholder "Username"
                ]
                []
            , input
                [ onInput UpdatePassword
                , value password
                , type_ "password"
                , class "bg-transparent text-white accent-white placeholder-white text-xl p-2 rounded-md shadow-md mb-5 "
                , placeholder "Password"
                ]
                []
            , button
                [ onClick Login
                , type_ "button"
                , class "inline-block px-6 py-2.5 bg-fuchsia-800 text-white font-bold text-xl leading-tight uppercase rounded shadow-md hover:bg-fuchsia-700 hover:shadow-lg focus:bg-fuchsia-700 focus:shadow-lg focus:outline-none focus:ring-0 active:bg-fuchsia-800 active:shadow-lg transition duration-150 ease-in-out"
                ]
                [ text "Login" ]
            , button
                [ onClick Register
                , type_ "button"
                , class "inline-block px-6 py-2.5 bg-fuchsia-800 text-white font-bold text-xl leading-tight uppercase rounded shadow-md hover:bg-fuchsia-700 hover:shadow-lg focus:bg-fuchsia-700 focus:shadow-lg focus:outline-none focus:ring-0 active:bg-fuchsia-800 active:shadow-lg transition duration-150 ease-in-out"
                ]
                [ text "Register" ]
            ]
        ]
