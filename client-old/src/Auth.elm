module Auth exposing (AuthLevel, Permissions, jsonToPermissions)

import Json.Decode as D


type alias AuthLevel =
    { label : String
    , permissions : Permissions
    , authentified : Bool
    }


type alias Permissions =
    { readAccounts : Bool
    , readTransactions : Bool
    , manageTeams : List String
    , admin : Bool
    }


jsonToPermissions : D.Decoder Permissions
jsonToPermissions =
    D.map4 Permissions (D.field "readAccounts" D.bool) (D.field "readTransactions" D.bool) (D.field "manageTeams" (D.list D.string)) (D.field "admin" D.bool)
