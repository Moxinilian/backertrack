module Ledger exposing (LedgerPage(..), ledgerView)

import Html exposing (..)
import Html.Attributes exposing (..)


type LedgerPage
    = AccountList
    | NewAccount
    | TxnList { account_name : String, account_id : Int }
    | NewTxn { account_name : String, account_id : Int }


ledgerView : LedgerPage -> Html msg
ledgerView page =
    div []
        [ nav [ class "breadcrumb" ]
            [ ul [] (ledgerPath page)
            ]
        ]


ledgerPath : LedgerPage -> List (Html msg)
ledgerPath page =
    case page of
        AccountList ->
            [ li [ class "is-active" ] [ a [ href "#" ] [ text "Ledger" ] ] ]

        NewAccount ->
            [ li [] [ a [ href "/ledger" ] [ text "Ledger" ] ], li [ class "is-active" ] [ a [ href "#" ] [ text "New account" ] ] ]

        TxnList { account_name } ->
            [ li [] [ a [ href "/ledger" ] [ text "Ledger" ] ], li [] [ text account_name ] ]

        NewTxn { account_name, account_id } ->
            [ li []
                [ a [ href "/ledger" ]
                    [ text "Ledger" ]
                ]
            , li [] [ a [ href ("/ledger/" ++ String.fromInt account_id) ] [ text account_name ] ]
            , li [] [ text "New transaction" ]
            ]
