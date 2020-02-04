port module Main exposing (..)

import Browser
import CodeEditor
import Json.Decode 
import Json.Encode as E
import Html.Styled as Html exposing (Html)
import Html.Styled.Attributes as Attributes exposing (property)
import Html.Styled.Events exposing (onInput)



-- MAIN


port codeChange : E.Value -> Cmd msg
port pyErrors : (E.Value -> msg) -> Sub msg


main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view >> Html.toUnstyled
        }



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
    pyErrors (Json.Decode.decodeValue Json.Decode.string >> handlePortError >> PyError )



-- MODEL


type alias Model =
    { code : String
    , error : Maybe String
    }

startCode : String
startCode =
    """import math

move(350, 300)
zoom(20)

spin(time/100)

count = 30
for i in range(count):
    fill(0, i/count*.45, (1 - i/count)*1.45, i/count*0.05)
    spin(abs(math.sin((math.pi/1500)*(time+(i*100)))) * 5)
    push()
    zoom(i * 1.05, 1)
    rect(-.5, -.5, 1, 1)
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

init : () -> (Model, Cmd Msg)
init _ =
    (
        { code = startCode
        , error = Nothing
        }
    , codeChange <| E.string <| startCode
    )



-- UPDATE


type Msg
    = Change String
    | PyError String


update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
    case msg of
        Change newCode ->
            ( { model | code = newCode }
            , codeChange <| E.string <| newCode
            )
    
        PyError newError ->
            ( { model | error = if String.length newError > 0 then Just newError else Nothing }
            , Cmd.none
            )



-- VIEW


view : Model -> Html Msg
view model =
    Html.div [ Attributes.id "elm" ]
        [ Html.div [ Attributes.id "codeWrapper" ]
            [ CodeEditor.view
                [ CodeEditor.mode "python"
                , CodeEditor.value model.code
                , CodeEditor.onChange Change
                ]
            ]
        , Html.div
            [ Attributes.id "pyErr" ]
            [ Html.text (Maybe.withDefault "" model.error) ]
        ]
