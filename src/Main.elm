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
    """move(400, 300)

spin(time)

move(-50, -50)
for i in [ i for i in range(20)]:
    #spin(time/2)
    fill(1, 1, 1, 0.1)
    spin(time/(400 * (time % 80 + 1) * 0.05))
    rect(i, i, (i+1)/21 * 100, (i+1)/21 * 100)
"""

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
