module Ledger exposing (LedgerPage(..), LedgerState, defaultLedgerPage, ledgerView)

import Html exposing (..)
import Html.Attributes exposing (..)
import Utils exposing (Currency)
import Loading exposing (loadingView)


type alias Account =
    { id : Int
    , name : String
    , balance : Currency
    , opening_balance : Currency
    }


type alias LedgerState =
    { accounts : List Account
    , isLoading : Bool
    , page : LedgerPage
    }


type LedgerPage
    = AccountList
    | NewAccount
    | TxnList Account
    | NewTxn Account


defaultLedgerPage : LedgerState
defaultLedgerPage =
    LedgerState [] True AccountList


ledgerView : LedgerState -> Html msg
ledgerView state =
    if state.isLoading then
        loadingView

    else
        div [ class "columns" ]
            [ div [ class "column is-3" ]
                [ aside [ class "menu is-hidden-mobile" ]
                    [ p [ class "menu-label side-menu-label" ] [ text "Accounts" ]
                    ]
                ]
            , div [ class "column is-9" ]
                [ nav [ class "breadcrumb" ]
                    [ ul [] (ledgerPath state.page)
                    ]
                ]
            ]


ledgerPath : LedgerPage -> List (Html msg)
ledgerPath page =
    case page of
        AccountList ->
            [ li [ class "is-active" ] [ a [ href "#" ] [ text "Ledger" ] ] ]

        NewAccount ->
            [ li [] [ a [ href "/ledger" ] [ text "Ledger" ] ], li [ class "is-active" ] [ a [ href "#" ] [ text "New account" ] ] ]

        TxnList { name } ->
            [ li [] [ a [ href "/ledger" ] [ text "Ledger" ] ], li [ class "is-active" ] [ a [ href "#" ] [ text name ] ] ]

        NewTxn { name, id } ->
            [ li []
                [ a [ href "/ledger" ]
                    [ text "Ledger" ]
                ]
            , li [] [ a [ href ("/ledger/" ++ String.fromInt id) ] [ text name ] ]
            , li [] [ text "New transaction" ]
            ]
