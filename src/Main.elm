port module Main exposing (..)

import Browser
import Json.Encode as E
import Html exposing (Html, Attribute, div, input, text)
import Html.Attributes exposing (..)
import Html.Events exposing (onInput)



-- MAIN


port codeChange : E.Value -> Cmd msg


main =
  Browser.element
    { init = init
    , update = update
    , subscriptions = subscriptions
    , view = view
    }



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
  Sub.none



-- MODEL


type alias Model =
  { content : String
  }


init : () -> (Model, Cmd Msg)
init _ =
    ( { content = "" }
    , Cmd.none
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
  div []
    [ input [ placeholder "Text to reverse", value model.content, onInput Change ] []
    , div [] [ text (String.reverse model.content) ]
    ]
