module Panic exposing (Panic(..), panicView)

import Html exposing (..)

type Panic
    = None
    | Reason String

panicView : Panic -> List (Html msg)
panicView panic =
    case panic of
        None -> [ text "Unexpected panic. This is a bug." ]
        Reason x -> [ text ("Well fuck: " ++ x ++ ".") ]
    