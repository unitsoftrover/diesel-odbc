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
        CreditAmount -> Nullable<Decimal>,
        IsHeadOffice -> Bool,
        TestSmallInt -> SmallInt, 
        TestTinyInt -> TinyInt, 
        TestDate -> Date, 
        TestTime -> Timestamp, 
        TestFloat -> Double, 
        TestReal -> Float, 
        TestBigInt -> BigInt, 
        TestBin -> Binary,
        CreditInstruction -> Text,
    }
}

table! {    
    #[sql_name = "quotation"]
    quotation_a(QuotationNo) {        
        QuotationID	->	Integer,self_increase_id=false,       
        QuotationNo	->	VarChar,
        LeadSource	->	VarChar,
        QuotationBy	->	VarChar,
        QuotationTo	->	VarChar,
        AddressQuotation	->	VarChar,
        CityQuotation	->	VarChar,
        ProvinceQuotation	->	VarChar,
        CountryQuotation	->	VarChar,
        PostCodeQuotation	->	VarChar,
        QuotationContactID	->	Integer,        
        ContactQuotation	->	VarChar,
        TelQuotation	->	VarChar,
        FaxQuotation	->	VarChar,
        MobileQuotation	->	VarChar,
        EmailQuotation	->	VarChar,
        BillTo	->	VarChar,
        AddressBill	->	VarChar,
        CityBill	->	VarChar,
        ProvinceBill	->	VarChar,
        CountryBill	->	VarChar,
        PostCodeBill	->	VarChar,
        BillingContactID	->	Integer,
        ContactBill	->	VarChar,
        TelBill	->	VarChar,
        FaxBill	->	VarChar,
        MobileBill	->	VarChar,
        EmailBill	->	VarChar,
    }
}

table! {    
    #[sql_name = "quotation"]
    quotation_b (QuotationID) {
        QuotationID	->	Integer,      
        CompanyID	->	Integer,
        CompanyCode	->	VarChar,
        CompanyName	->	VarChar,
        ContactPersonID	->	Integer	,
        SalClient	->	VarChar,
        FirstNameClient	->	VarChar,
        LastNameClient	->	VarChar	,
        Salesman	->	VarChar	,
        CSR	->	VarChar,
        Operator	->	VarChar	,
        CreateDate	->	Timestamp	,
        CreateBy	->	VarChar	,
        OfficeCode	->	VarChar	,
        PaymentParty	->	VarChar	,
        ClientType	->	VarChar	,
        CurrentVersion	->	Integer	,
        OpportunityDescription	->	VarChar	,
        OfficeService	->	VarChar	,
        Seller	->	VarChar	,
        BankClient	->	VarChar	,
        MultiShipment	->	Bool	,
        IsSubmitedOnline	->	Bool	,
        IsSubmited	->	Bool	,
        SubmitDate	->	Timestamp	,
        IsRejectToClient	->	Bool	,

    }
}

table! {    
    #[sql_name = "quotation"]
    quotation_c (QuotationID) {
        QuotationID	->	Integer,     
        IsConfirmedSalesOrder	->	Bool	,
        ConfirmedBy	->	VarChar	,
        ConfirmDate	->	Timestamp	,
        ApprovedSalesOrder	->	Bool	,
        ApprovedSalesOrderBy	->	VarChar	,
        DateApprovedSalesOrder	->	Timestamp	,
        PrintClientProductCode	->	Bool	,
        DepartmentCode	->	VarChar	,            
        Remark	->	VarChar	,
        Other1	->	VarChar	,
        Other2	->	VarChar	,
        Other3	->	VarChar	,
        Other4	->	VarChar	,
        Other5	->	VarChar	,
        Other6	->	VarChar	,
        Other7	->	VarChar	,
        Other8	->	VarChar	,
        Other9	->	VarChar	,
        Other10	->	VarChar	,
        Other11	->	VarChar	,
        Other12	->	VarChar	,
        Other13	->	VarChar	,
        Other14	->	VarChar	,
        Other15	->	VarChar	,
        Other16	->	VarChar	,
        Other17	->	VarChar	,
        Other18	->	VarChar	,
        Other19	->	VarChar	,
        Other20	->	VarChar	,
        // Other21	->	VarChar	,
        // Other22	->	VarChar	,
        // Other23	->	VarChar	,
        // Other24	->	VarChar	,
        // Other25	->	VarChar	,
        // Other26	->	VarChar	,
        // Other27	->	VarChar	,
        // Other28	->	VarChar	,   
    }
}


table! { 
    #[sql_name = "quotation2"]
    quotation2_a (QuotationID) {        
        QuotationID	->	Integer	,
        BrokerID	->	Integer	,
        BrokerCode	->	Char	,
        BrokerName	->	Varchar	,
        AddressBroker	->	Varchar	,
        CityBroker	->	Varchar	,
        ProvinceBroker	->	Varchar	,
        CountryBroker	->	Char	,
        PostCodeBroker	->	Char	,
        BrokerContactID	->	Integer	,
        ContactBroker	->	Varchar	,
        TelBroker	->	Char	,
        FaxBroker	->	Char	,
        MobileBroker	->	Char	,
        EmailBroker	->	Varchar	,
        AgentBook	->	Bool	,
        BookingAgentCode	->	Char	,
        BookingAgent	->	Varchar	,
        NeedSurvey	->	Bool	,
        SurveyDate	->	Timestamp	,
        SurveyTime	->	Timestamp	,
        PreferDestAgent	->	Varchar	,
        EstPackingDate	->	Timestamp	,
        EstPackingTime	->	Timestamp	,
        EstPackingDays	->	Integer	,
        NoOfLabors	->	Integer	,
        ShipperETD	->	Timestamp	,
        ShipperETA	->	Timestamp	,
        LoadingAt	->	Varchar	,
    }
}

table! {    
    #[sql_name = "quotation2"]
    quotation2_b (QuotationID) {
        QuotationID	->	Integer,             
        SpecialInstruction	->	Varchar	,
        DeliveryInstruction	->	Varchar	,
        CustomsDocuments	->	Varchar	,
        CustomsDocumentNums	->	Varchar	,
        CountryPacking	->	Char	,      
        AddressPacking	->	Varchar	,
        CityPacking	->	Varchar	,
        ProvincePacking	->	Varchar	,
        PostCodePacking	->	Varchar	,
        ContactPacking	->	Varchar	,
        TelPacking	->	Varchar	,
        FaxPacking	->	Varchar	,
        HomeTelPacking	->	Varchar	,
        MobilePacking	->	Varchar	,
        EmailPacking	->	Varchar	,
        Closed	->	Bool	,
        DateClosed	->	Timestamp	,
        ClosedBy	->	Char	,
        DateAvailable	->	Timestamp	,
        AddedValueTaxNo	->	Varchar	,
        AccountNo	->	Varchar	,
        DateContractStart	->	Timestamp	,
        RentPaymentDay	->	Integer	,
        LeaseFlag	->	Bool	,
        LeaseTerm	->	Varchar	,       
    }
}

table! {
    #[sql_name = "quotationver"]
    quotationver_a(QuotationID, VersionNo)
    {
        QuotationID	->	Integer	,
        VersionNo	->	Integer	,
        Approved	->	Char	,
        DateApproved	->	Timestamp	,
        ApproveAgree	->	Char	,
        ApproveSuggestion	->	Text	,
        ApprovedEmployee	->	Char	,
        Quoted	->	Char	,
        QuotationEmployee	->	Char	,
        DateQuoted	->	Timestamp	,
        ClientApproved	->	Char	,
        ApproveSuggestionClient	->	Varchar	,
        ContactPersonApproved	->	Integer	,
        DateClientApproved	->	Timestamp	,
        Remark	->	Varchar	,
        Other1	->	Varchar	,
        Other2	->	Varchar	,
        Other3	->	Varchar	,
        Other4	->	Varchar	,
        Other5	->	Varchar	,
        Other6	->	Varchar	,
        Other7	->	Varchar	,
        Other8	->	Varchar	,
        Other9	->	Varchar	,
        Other10	->	Varchar	,
        Currency	->	Char	,
        TotalAmount	->	Nullable<Decimal>	,
        AdditionalTerms	->	Varchar	,
        PaymentMethodCode	->	Char	,
        PaymentMethod	->	Varchar	,
    }
}

table! {    
    #[sql_name = "quotationver"]
    quotationver_b (QuotationID,VersionNo) {
        QuotationID	->	Integer	,
        VersionNo	->	Integer	,             
        BreachDuty	->	Varchar	,
        BreachSolve	->	Varchar	,
        QuotationSeed	->	Char	,
        Possibility	->	Float	,
        Suppliers	->	Varchar	,
        RequestPrepareDelivery	->	Bool	,
        DeliveryInstruction	->	Bool	,
        RequestApprovePrepareGoods	->	Bool	,
        RequestApprovePrepareGoodsBy	->	Char	,
        DateRequestApprovePrepareGoods	->	Timestamp	,
        ApprovePrepareGoods	->	Bool	,
        ApprovePrepareGoodsAgree	->	Bool	,
        DateApprovePrepareGoods	->	Timestamp	,
        ApprovePrepareGoodsBy	->	Char	,
        RequestApproveDelivery	->	Bool	,
        RequestApproveDeliveryBy	->	Char	,
        DateRequestApproveDelivery	->	Timestamp	,
        ApproveDelivery	->	Bool	,
        ApproveDeliveryAgree	->	Bool	,
        DateApproveDelivery	->	Timestamp	,
        ApproveDeliveryBy	->	Char	,                
    }
}

table!{
    #[sql_name = "quotationverproject"]
    quotationverproject_a(QuotationID, VersionNo, ProjectNo)
    {
        QuotationID	->	Integer	,
        VersionNo	->	Integer	,
        ProjectNo	->	Integer	,
        TotalAmount	->	Decimal	,
        TotalCostAmount	->	Decimal	,
        Margin	->	Decimal	,
        Profit	->	Decimal	,
        Remark	->	Varchar	,
        Other1	->	Varchar	,
        Other2	->	Varchar	,
        Other3	->	Varchar	,
        Other4	->	Varchar	,
        Other5	->	Varchar	,
        Other6	->	Varchar	,
        Other7	->	Varchar	,
        Other8	->	Varchar	,
        Other9	->	Varchar	,
        Other10	->	Varchar	,
        InvoiceInsured	->	Char	,
        CommissionAmount	->	Decimal	,
        TotalAmountSay	->	Varchar	,
        DisplayDetailItems	->	Bool	,
        OtherCostAmount	->	Decimal	,
        ShareOtherCost	->	Bool	,
        CurrencyCost	->	Varchar	,
        TotalCostFixCurrency	->	Decimal	,
        IncludeTax	->	Bool	,
        TaxRate	->	Decimal	,
    }
}

table!{
    #[sql_name = "quotationverproject"]
    quotationverproject_b(QuotationID, VersionNo, ProjectNo)
    {        
        QuotationID	->	Integer	,
        VersionNo	->	Integer	,
        ProjectNo	->	Integer	,
        CalcCommWithRate	->	Bool	,
        CommissionRate	->	Decimal	,
        SettledCommission	->	Bool	,
        FreightCalculatedMethod	->	Char	,
        FreightRate	->	Decimal	,
        AmountFreight	->	Decimal	,
        PremiumRate	->	Decimal	,
        AmountInsurance	->	Decimal	,
        AmountTarrif	->	Decimal	,
        AmountReturnTax	->	Decimal	,
        AmountPrePayment	->	Decimal	,
        AmountDeliveryPayment	->	Decimal	,
        AmountPaymentCancelled	->	Decimal	,
        IsBorrowCase	->	Bool	,
        TotalAmountCase	->	Decimal	,
        PriceIncludeFreight	->	Bool	,
        HasPrePayment	->	Bool	,
        TotalAmountNoTax	->	Decimal	,
        TotalAmountTax	->	Decimal	,
        AmountCreatePayment	->	Decimal	,
        AmountRecivedPayment	->	Decimal	,
        DiscountRate	->	Decimal	,
        DiscountAmount	->	Decimal	,
        Deposit	->	Decimal	,
        FixDeposit	->	Bool	,                
    }
}


table! {
    #[sql_name = "project"]
    project_a(QuotationID,ProjectNo)
    {
        ID	->	Integer	,
        QuotationID	->	Integer	,
        ProjectNo	->	Integer	,
        RFQID	->	Integer	,
        RFQProjectNo	->	Integer	,
        ProjectName	->	Varchar	,
        SubProjectFlag	->	Char	,
        JobNo	->	Varchar	,
        StatusQuotation	->	Char	,
        StatusCheckedBy	->	Char	,
        DateCheckStatus	->	Timestamp	,
        LostReason	->	Char	,
        WonReason	->	Char	,
        PreferDateDelivery	->	Timestamp	,
        Deliveried	->	Char	,
        DelieveryCheckedBy	->	Char	,
        DateDelivery	->	Timestamp	,
        JobCreated	->	Char	,
        JobCreatedBy	->	Char	,
        DateJobCreatedBy	->	Timestamp	,
        Remark	->	Text	,
        Other1	->	Varchar	,
        Other2	->	Varchar	,
        Other3	->	Varchar	,
        Other4	->	Varchar	,
        Other5	->	Varchar	,
        Other6	->	Varchar	,
        Other7	->	Varchar	,
        Other8	->	Varchar	,
        Other9	->	Text	,
        Other10	->	Text	,
    }

}

table!{
    #[sql_name = "project"]
    project_b(QuotationID, ProjectNo)
    {
        QuotationID	->	Integer	,
        ProjectNo	->	Integer	,
        Other11	->	Text	,
        Other12	->	Text	,
        Other13	->	Text	,
        Other14	->	Text	,
        Other15	->	Text	,
        OrderSeed	->	Char	,
        DateShippingInstruct	->	Timestamp	,
        DateShippingAdvise	->	Timestamp	,
        Consignee	->	Varchar	,
        Documents	->	Text	,
        ShippingMarks	->	Text	,
        Notity	->	Text	,
        ShippingRemark	->	Text	,
        PostScript	->	Text	,
        LoadingPort	->	Varchar	,
        DestPort	->	Varchar	,
        ContainerNo	->	Varchar	,
        OBL	->	Varchar	,
        JobType	->	Char	,
        ShippingMethod	->	Char	,
        ServiceType	->	Char	,
        LoadingType	->	Char	,
        DescriptionOfShipment	->	Char	,
        SizeOfShipment	->	Char	,
        OriginCity	->	Varchar	,
        OriginCountry	->	Char	,
        DestCity	->	Varchar	,
        DestCountry	->	Char	,
    }

}

table!{
    #[sql_name = "project"]
    project_c(QuotationID, ProjectNo)
    {        
        QuotationID	->	Integer	,
        ProjectNo	->	Integer	,
        VesselName	->	Varchar	,
        VoyageNo	->	Varchar	,
        ETD	->	Timestamp	,
        ETA	->	Timestamp	,
        ShippingAgent	->	Text	,
        AdviseRemark	->	Text	,
        TotalQtyPackages	->	Float	,
        SayTotalQtyPackages	->	Varchar	,
        UseTonAsWeightUnit	->	Bool	,
        TotalNetWeight	->	Float	,
        SayTotalNetWeight	->	Varchar	,
        TotalGrossWeight	->	Float	,
        SayTotalGrossWeight	->	Varchar	,
        TotalVolume	->	Float	,
        SayTotalVolume	->	Varchar	,
        DescriptionOfPacking	->	Varchar	,
        ShippingTime	->	Varchar	,
        DeliveryType	->	Char	,
        DeliveryTerm	->	Varchar	,
        PurchaserOrderNo	->	Varchar	,
        Origin	->	Text	,
        PaymentMethodCode	->	Char	,
        PaymentMethod	->	Varchar	,
        DealAsCommission	->	Bool	,
    }
}

table!{
    #[sql_name = "project2"]
    project2_a(QuotationID, ProjectNo){
        ID	->	Integer	,
        QuotationID	->	Integer	,
        ProjectNo	->	Integer	,
        RFQID	->	Integer	,
        RFQProjectNo	->	Integer	,
        CountryDestContact	->	Char	,
        AddressDestContact	->	Varchar	,
        CityDestContact	->	Varchar	,
        ProvinceDestContact	->	Varchar	,
        PostCodeDestContact	->	Varchar	,
        ContactDestContact	->	Varchar	,
        TelDestContact	->	Varchar	,
        FaxDestContact	->	Varchar	,
        HomeTelDestContact	->	Varchar	,
        MobileDestContact	->	Varchar	,
        EmailDestContact	->	Varchar	,
        CountryDestDelivery	->	Char	,
        AddressDestDelivery	->	Varchar	,
        CityDestDelivery	->	Varchar	,
        ProvinceDestDelivery	->	Varchar	,
        PostCodeDestDelivery	->	Varchar	,
        ContactDestDelivery	->	Varchar	,
        TelDestDelivery	->	Varchar	,
        FaxDestDelivery	->	Varchar	,
        HomeTelDestDelivery	->	Varchar	,
        MobileDestDelivery	->	Varchar	,
        EmailDestDelivery	->	Varchar	,
    }
}

table!{
    #[sql_name = "project2"]
    project2_b(QuotationID, ProjectNo)
    {        
        QuotationID	->	Integer	,
        ProjectNo	->	Integer	,
        Allowance	->	Varchar	,
        Storage	->	Bool	,
        StorageAt	->	Char	,
        LostToCode	->	Char	,
        LostToName	->	Varchar	,
        DestAgentCode	->	Char	,
        DestAgent	->	Varchar	,
        Insurance	->	Text	,
        Inspection	->	Text	,
        Container20Qty	->	Integer	,
        Container40Qty	->	Integer	,
        Container40HQty	->	Integer	,
        Container45HQty	->	Integer	,
        TruckType	->	Char	,
        TruckQty	->	Integer	,
    }
}

table!{
    #[sql_name = "quotationitem"]
    quotationitem_a(QuotationID,VersionNo,ProjectNo,ItemNo)
    {
        QuotationID	->	Integer	,
        VersionNo	->	Integer	,
        ProjectNo	->	Integer	,
        ItemNo	->	Integer	,
        Number	->	Integer	,
        Article	->	Text	,
        ProductCode	->	Char	,
        ProductName	->	Varchar	,
        Brand	->	Varchar	,
        ActivityCostCode	->	Char	,
        Price	->	Decimal	,
        Quantity	->	Decimal	,
        UnitOfMeasure	->	Char	,
        QuoteAmount	->	Decimal	,
        EstUnitCost	->	Decimal	,
        EstAmount	->	Decimal	,
        ReferNo	->	Integer	,
        ItemType	->	Integer	,
        Remark	->	Text	,
        Other1	->	Varchar	,
        Other2	->	Varchar	,
        Other3	->	Varchar	,
        Other4	->	Text	,
        Other5	->	Text	,
        RequestID	->	Integer	,
        RequestVersionNo	->	Integer	,
        RequestProjectNo	->	Integer	,
        RequestItemNo	->	Integer	,
        QtyInvoiced	->	Float	,
        QtyDeliveried	->	Float	,
        QtyDeliveriedActual	->	Float	,
    }
}

table!{
    #[sql_name = "quotationitem"]
    quotationitem_b(QuotationID, VersionNo, ProjectNo,ItemNo)
    {        
        QuotationID	->	Integer	,
        VersionNo	->	Integer	,
        ProjectNo	->	Integer	,
        ItemNo	->	Integer	,
        UnitBase	->	Char	,
        ChangeUnit	->	Char	,
        UnitFactor1	->	Float	,
        UnitFactor2	->	Float	,
        TaxRate	->	Float	,
        DateDelivery	->	Timestamp	,
        Comission	->	Float	,
        ComissionAmount	->	Float	,
        QtyOfPackages	->	Float	,
        PackingType	->	Varchar	,
        NetWeightPerPackage	->	Float	,
        TotalNetWeight	->	Float	,
        GrossWeightPerPackage	->	Float	,
        TotalGrossWeight	->	Float	,
        BatchNo	->	Text	,
        StockUnitCost	->	Decimal	,
        StockCostAmount	->	Decimal	,
        // WareHouseCode	->	Char	,
        ProducerCode	->	Char	,
        Producer	->	Text	,
        IsOrigin	->	Bool	,
        QtyPrepared	->	Decimal	,
        QtyCanDelivery	->	Decimal	,
        // IOJobNo	->	Char	,
        CustosmNoOfGoods	->	Varchar	,
        Measurement	->	Varchar	,
        TotalVolume	->	Decimal	,
        ReturnTaxRate	->	Decimal	,
        AmountReturnTax	->	Decimal	,
        // TarrifRate	->	Decimal	,
        // AmountTarrif	->	Decimal	,
    }
}

table!{
    #[sql_name = "quotationitem"]
    quotationitem_c(QuotationID, VersionNo, ProjectNo,ItemNo)
    {
        QuotationID	->	Integer	,
        VersionNo	->	Integer	,
        ProjectNo	->	Integer	,
        ItemNo	->	Integer	,
        PriceCase	->	Decimal	,
        AmountCase	->	Decimal	,
        PurchaseInquiryNo	->	Char	,
        QtyMRPImported	->	Decimal	,
        QtyProductionImported	->	Decimal	,
        QtyUseStock	->	Decimal	,
        QtyArrangedUseStock	->	Decimal	,
        QtyDeliveriedUseStock	->	Decimal	,
        QtyPerPackage	->	Decimal	,
        QtyPerPackageInner	->	Decimal	,
        BOMCode	->	Char	,
        ProductCodeClient	->	Varchar	,
        // CaseNo	->	Varchar	,
        ProductNameEnglish	->	Varchar	,
        // BarCode	->	Varchar	,
        // CSR	->	Char	,
        // PricePerCase	->	Decimal	,
        QuotationNo	->	Varchar	,
        IsProduction	->	Bool	,
        JobNo	->	Char	,
        PriceNoTax	->	Decimal	,
        AmountNoTax	->	Decimal	,
        AmountTax	->	Decimal	,
        CaseVolume	->	Decimal	,
        // QtyForecast	->	Decimal	,
        ChineseRemark	->	Text	,
        ServiceProjectID	->	Integer	,
        PurchasePrice	->	Decimal	,
        PurchaseCurrency	->	Char	,
        DiscountPerItem	->	Decimal	,        
    }
}