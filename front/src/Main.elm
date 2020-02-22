port module Main exposing (..)

import Browser
import Browser.Navigation as Nav
import CodeEditor
import Html.Styled as Html exposing (Html)
import Html.Styled.Attributes as Attributes exposing (property)
import Html.Styled.Events exposing (onClick, onInput)
import Http
import Json.Decode
import Json.Encode as E
import Url



-- MAIN


port codeChange : E.Value -> Cmd msg


port pyErrors : (E.Value -> msg) -> Sub msg


main =
    Browser.application
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        , onUrlChange = UrlChanged
        , onUrlRequest = LinkClicked
        }



-- MODEL


type alias Model =
    { code : String
    , error : Maybe String
    , notifications : List Notification
    , key : Nav.Key
    , url : Url.Url
    }


init : () -> Url.Url -> Nav.Key -> ( Model, Cmd Msg )
init _ url key =
    let
        -- this URL will point to the actual code for the whim
        dataUrl =
            Url.toString { url | path = "/raw" ++ url.path }
    in
    ( { code = ""
      , error = Nothing
      , notifications = []
      , url = url
      , key = key
      }
    , Cmd.batch
        [ codeChange <| E.string <| ""
        , Http.get
            { url = dataUrl
            , expect =
                Http.expectString
                    (\result ->
                        case result of
                            Ok code ->
                                NewCode code

                            Err e ->
                                NoWhimErr ("No whim at url '" ++ dataUrl ++ "'")
                    )
            }
        ]
    )



-- UPDATE


type Msg
    = NewCode String
    | NoWhimErr String
    | PostWhim
    | Notify Notification
    | PyError String
    | LinkClicked Browser.UrlRequest
    | UrlChanged Url.Url


type Notification
    = Uploaded
    | UploadErr


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        NewCode newCode ->
            ( { model | code = newCode }
            , codeChange <| E.string <| newCode
            )

        NoWhimErr err ->
            ( { model | error = Just err }
            , Cmd.none
            )

        Notify note ->
            ( { model | notifications = note :: model.notifications }
            , Cmd.none
            )

        PostWhim ->
            ( model
            , Http.request
                { method = "PUT"
                , headers = []
                , url = Url.toString model.url
                , body = Http.stringBody "text/plain" model.code
                , expect =
                    Http.expectWhatever
                        (\result ->
                            case result of
                                Ok _ ->
                                    Notify Uploaded

                                Err _ ->
                                    Notify UploadErr
                        )
                , timeout = Nothing
                , tracker = Nothing
                }
            )

        PyError newError ->
            ( { model
                | error =
                    if String.length newError > 0 then
                        Just newError

                    else
                        Nothing
              }
            , Cmd.none
            )

        LinkClicked _ ->
            ( model, Cmd.none )

        UrlChanged _ ->
            ( model, Cmd.none )



-- SUBSCRIPTIONS


handlePortError : Result Json.Decode.Error String -> String
handlePortError result =
    case result of
        Ok value ->
            value

        -- an error fetching an error! today is a bad day!
        Err error ->
            Json.Decode.errorToString error


subscriptions : Model -> Sub Msg
subscriptions model =
    pyErrors (Json.Decode.decodeValue Json.Decode.string >> handlePortError >> PyError)



-- VIEW


view : Model -> Browser.Document Msg
view model =
    { title = "awh.im"
    , body =
        [ Html.toUnstyled <|
            Html.div [ Attributes.id "elm" ]
                [ Html.div [ Attributes.id "codeWrapper" ]
                    [ CodeEditor.view
                        [ CodeEditor.mode "python"
                        , CodeEditor.value model.code
                        , CodeEditor.onChange NewCode
                        ]
                    , Html.button
                        [ Attributes.id "postWhim"
                        , onClick PostWhim
                        ]
                        [ Html.text "Post Whim" ]
                    ]
                , Html.div
                    [ Attributes.id "pyErr" ]
                    [ Html.text (Maybe.withDefault "" model.error) ]
                ]
        ]
    }
