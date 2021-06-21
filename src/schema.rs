table! {
    users (id) {
        id -> Text,
        name -> Text,
    }
}

table! {    
    company (CompanyID) {
        CompanyID -> Integer,
        CompanyCode -> VarChar,       
        CompanyName -> VarChar,
        CompanyNameCN -> VarChar,
        DateCreated -> Timestamp,
        CreditAmount -> Decimal,
        // IsHeadOffice -> Bool,
    }
}
