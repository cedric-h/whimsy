port module Main exposing (..)

import Browser
import CodeEditor
import Json.Encode as E
import Html.Styled as Html exposing (Html)
import Html.Styled.Attributes as Attributes exposing (property)
import Html.Styled.Events exposing (onInput)



-- MAIN


port codeChange : E.Value -> Cmd msg


main =
  Browser.element
    { init = init
    , update = update
    , subscriptions = subscriptions
    , view = view >> Html.toUnstyled
    }



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
  Sub.none



-- MODEL


type alias Model =
  { content : String
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
    ( { content = startCode }
    , codeChange <| E.string <| startCode
    )



-- UPDATE


type Msg
  = Change String


update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    Change newContent ->
      ( { model | content = newContent }
      , codeChange <| E.string <| newContent
      )



-- VIEW


view : Model -> Html Msg
view model =
  Html.div [ Attributes.id "elm" ]
    [ CodeEditor.view
        [ CodeEditor.mode "python"
        , CodeEditor.value model.content
        , CodeEditor.onChange Change
        ]
    ]
