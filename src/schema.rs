table! {
    users (id) {
        id -> Text,
        name -> Text,
    }
}

table! {    
    company (CompanyCode) {
        CompanyID -> Integer,self_increase_id=false,
        CompanyCode -> VarChar,       
        CompanyType -> VarChar,
        CreateOffice -> VarChar,
        CompanyName -> VarChar,
        CompanyNameCN -> VarChar,
        DateCreated -> Timestamp,
        // CreditAmount -> Decimal,
        // IsHeadOffice -> Bool,
        // TestSmallInt -> SmallInt, 
        // TestTinyInt -> TinyInt, 
        // TestDate -> Date, 
        // TestTime -> Timestamp, 
        // TestFloat -> Double, 
        // TestReal -> Float, 
        // TestBigInt -> BigInt, 
        // // TestBin -> Binary,
        // CreditInstruction -> Text,

    }
}
