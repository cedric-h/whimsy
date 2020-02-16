port module Main exposing (..)

import Browser
import Browser.Navigation as Nav
import CodeEditor
import Html.Styled as Html exposing (Html)
import Html.Styled.Attributes as Attributes exposing (property)
import Html.Styled.Events exposing (onInput)
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
    , key : Nav.Key
    , url : Url.Url
    }


startCode : String
startCode =
    """import math

move(350, 350)
zoom(20)

spin(time/20)

count = 50
for i in range(2):
    spin(90)
    push()
    for i in range(count):
        fill(0, i/count*.45, (1 - i/count)*1.45, i/count*0.1)
        curve = abs(math.sin((math.pi/1500)*(time+(i*100))))
        spin((curve - .5) * 14)
        push()
        zoom(i * 0.6, 1)
        rect(-.5, -.5, 1, 1)
        pop()
    pop()"""



{--
startCode =
    """fill(0, 1, 1, 0.1)
move(400, 300)

spin(time/10)

for i in range(50):
    spin(time/200)
    #move(i, i)
    move(1, 1)
    push()
    zoom((i+1)/41 * 50, (i+1)/41 * 50)
    rect(-.5*i, -.5*i, 1, 100)
    pop()"""
    --}


init : () -> Url.Url -> Nav.Key -> ( Model, Cmd Msg )
init _ url key =
    let
        -- this URL will point to the actual code for the whim
        dataUrl =
            Url.toString { url | path = "/raw" ++ url.path }
    in
    ( { code = startCode
      , error = Nothing
      , url = url
      , key = key
      }
    , Cmd.batch
        [ codeChange <| E.string <| startCode
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
    | PyError String
    | LinkClicked Browser.UrlRequest
    | UrlChanged Url.Url


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

        _ ->
            ( model
            , Cmd.none
            )



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
                    ]
                , Html.div
                    [ Attributes.id "pyErr" ]
                    [ Html.text (Maybe.withDefault "" model.error) ]
                ]
        ]
    }
