extern crate bigdecimal;
use serde::{Deserialize, Serialize};
use crate::schema::{users, quotation_a, quotation_b, quotation_c, quotation2_a, quotation2_b, quotationver_a, quotationver_b, quotationverproject_a, quotationverproject_b, project_a, project_b, project_c, project2_a, project2_b,quotationitem_a,quotationitem_b,quotationitem_c};
use chrono::{NaiveDateTime, NaiveDate};
use diesel::prelude::*;
use bigdecimal::BigDecimal;
use std::default::Default;


#[derive(Debug, Clone, Serialize, Queryable, Insertable, PartialEq,QueryableByName)]
#[table_name = "users"]
pub struct User {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub name: String,
}

#[allow(non_snake_case)]
// #[derive(Debug, Clone, Queryable, PartialEq,QueryableByName)]
#[derive(Debug, Clone, Queryable, PartialEq)]
// #[table_name = "company"]
pub struct Company {
    pub CompanyID: i32,
    pub CompanyCode: String,
    pub CompanyType: String,
    pub CreateOffice : String,
    pub CompanyName: String,
    pub CompanyNameCN: String,
    pub DateCreated: NaiveDateTime,
    pub CreditAmount: Option<BigDecimal>,
    pub IsHeadOffice: bool,
    pub TestSmallInt : i16, 
    pub TestTinyInt : i8, 
    pub TestDate : NaiveDate, 
    pub TestTime : NaiveDateTime, 
    pub TestFloat : f64, 
    pub TestReal : f32, 
    pub TestBigInt : i64, 
    pub TestBin : Vec<u8>,
    pub CreditInstruction : String,
}

#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, PartialEq,Default)]
#[primary_key(QuotationNo)]
#[table_name = "quotation_a"]
pub struct QuotationA {
    pub	QuotationID	:	i32,
    pub	QuotationNo	:	String	,
    pub	LeadSource	:	String	,
    pub	QuotationBy	:	String	,
    pub	QuotationTo	:	String	,
    pub	AddressQuotation	:	String	,
    pub	CityQuotation	:	String	,
    pub	ProvinceQuotation	:	String	,
    pub	CountryQuotation	:	String	,
    pub	PostCodeQuotation	:	String	,
    pub	QuotationContactID	:	i32	,
    pub	ContactQuotation	:	String	,
    pub	TelQuotation	:	String	,
    pub	FaxQuotation	:	String	,
    pub	MobileQuotation	:	String	,
    pub	EmailQuotation	:	String	,
    pub	BillTo	:	String	,
    pub	AddressBill	:	String	,
    pub	CityBill	:	String	,
    pub	ProvinceBill	:	String	,
    pub	CountryBill	:	String	,
    pub	PostCodeBill	:	String	,
    pub	BillingContactID	:	i32	,
    pub	ContactBill	:	String	,
    pub	TelBill	:	String	,
    pub	FaxBill	:	String	,
    pub	MobileBill	:	String	,
    pub	EmailBill	:	String	,
}

#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, PartialEq,Default)]
#[primary_key(QuotationID)]
#[table_name = "quotation_b"]
pub struct QuotationB {
    pub	QuotationID	:	i32,
    pub	CompanyID	:	i32	,
    pub	CompanyCode	:	String	,
    pub	CompanyName	:	String	,
    pub	ContactPersonID	:	i32	,
    pub	SalClient	:	String	,
    pub	FirstNameClient	:	String	,
    pub	LastNameClient	:	String	,
    pub	Salesman	:	String	,
    pub	CSR	:	String	,
    pub	Operator	:	String	,
    pub	CreateDate	:	NaiveDateTime	,
    pub	CreateBy	:	String	,
    pub	OfficeCode	:	String	,
    pub	PaymentParty	:	String	,
    pub	ClientType	:	String	,
    pub	CurrentVersion	:	i32	,    
    pub	OpportunityDescription	:	String	,
    pub	OfficeService	:	String	,
    pub	Seller	:	String	,
    pub	BankClient	:	String	,
    pub	MultiShipment	:	bool	,
    pub	IsSubmitedOnline	:	bool	,
    pub	IsSubmited	:	bool	,
    pub	SubmitDate	:	NaiveDateTime	,
    pub	IsRejectToClient	:	bool	,
}

#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, PartialEq,Default)]
#[primary_key(QuotationID)]
#[table_name = "quotation_c"]
pub struct QuotationC {    
    pub	QuotationID	:	i32,
    pub	IsConfirmedSalesOrder	:	bool	,
    pub	ConfirmedBy	:	String	,
    pub	ConfirmDate	:	NaiveDateTime	,
    pub	ApprovedSalesOrder	:	bool	,
    pub	ApprovedSalesOrderBy	:	String	,
    pub	DateApprovedSalesOrder	:	NaiveDateTime	,
    pub	PrintClientProductCode	:	bool	,
    pub	DepartmentCode	:	String	,    
    pub	Remark	:	String	,
    pub	Other1	:	String	,
    pub	Other2	:	String	,
    pub	Other3	:	String	,
    pub	Other4	:	String	,
    pub	Other5	:	String	,
    pub	Other6	:	String	,
    pub	Other7	:	String	,
    pub	Other8	:	String	,
    pub	Other9	:	String	,
    pub	Other10	:	String	,
    pub	Other11	:	String	,
    pub	Other12	:	String	,
    pub	Other13	:	String	,
    pub	Other14	:	String	,
    pub	Other15	:	String	,
    pub	Other16	:	String	,
    pub	Other17	:	String	,
    pub	Other18	:	String	,
    pub	Other19	:	String	,
    pub	Other20	:	String	,
    // pub	Other21	:	String	,
    // pub	Other22	:	String	,
    // pub	Other23	:	String	,
    // pub	Other24	:	String	,
    // pub	Other25	:	String	,
    // pub	Other26	:	String	,
    // pub	Other27	:	String	,
    // pub	Other28	:	String	,
}


#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, Associations, PartialEq, Default)]
#[primary_key(QuotationID)]
#[belongs_to(QuotationA, foreign_key = "QuotationID")]
#[table_name = "quotation2_a"]
pub struct Quotation2A {
    pub	QuotationID	:	i32	,
    pub	BrokerID	:	i32	,
    pub	BrokerCode	:	String	,
    pub	BrokerName	:	String	,
    pub	AddressBroker	:	String	,
    pub	CityBroker	:	String	,
    pub	ProvinceBroker	:	String	,
    pub	CountryBroker	:	String	,
    pub	PostCodeBroker	:	String	,
    pub	BrokerContactID	:	i32	,
    pub	ContactBroker	:	String	,
    pub	TelBroker	:	String	,
    pub	FaxBroker	:	String	,
    pub	MobileBroker	:	String	,
    pub	EmailBroker	:	String	,
    pub	AgentBook	:	bool	,
    pub	BookingAgentCode	:	String	,
    pub	BookingAgent	:	String	,
    pub	NeedSurvey	:	bool	,
    pub	SurveyDate	:	NaiveDateTime	,
    pub	SurveyTime	:	NaiveDateTime	,
    pub	PreferDestAgent	:	String	,
    pub	EstPackingDate	:	NaiveDateTime	,
    pub	EstPackingTime	:	NaiveDateTime	,
    pub	EstPackingDays	:	i32	,
    pub	NoOfLabors	:	i32	,
    pub	ShipperETD	:	NaiveDateTime	,
    pub	ShipperETA	:	NaiveDateTime	,
    pub	LoadingAt	:	String	,
}

#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, PartialEq,Default)]
#[primary_key(QuotationID)]
#[table_name = "quotation2_b"]
pub struct Quotation2B {     
    pub	QuotationID	:	i32,
    pub	SpecialInstruction	:	String	,
    pub	DeliveryInstruction	:	String	,
    pub	CustomsDocuments	:	String	,
    pub	CustomsDocumentNums	:	String	,
    pub	CountryPacking	:	String	,
    pub	AddressPacking	:	String	,
    pub	CityPacking	:	String	,
    pub	ProvincePacking	:	String	,
    pub	PostCodePacking	:	String	,
    pub	ContactPacking	:	String	,
    pub	TelPacking	:	String	,
    pub	FaxPacking	:	String	,
    pub	HomeTelPacking	:	String	,
    pub	MobilePacking	:	String	,
    pub	EmailPacking	:	String	,    
    pub	Closed	:	bool	,
    pub	DateClosed	:	NaiveDateTime	,
    pub	ClosedBy	:	String	,
    pub	DateAvailable	:	NaiveDateTime	,
    pub	AddedValueTaxNo	:	String	,
    pub	AccountNo	:	String	,
    pub	DateContractStart	:	NaiveDateTime	,
    pub	RentPaymentDay	:	i32	,
    pub	LeaseFlag	:	bool	,
    pub	LeaseTerm	:	String	,        
}


#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, Associations, PartialEq, Default)]
#[primary_key(QuotationID,VersionNo)]
#[belongs_to(QuotationA, foreign_key = "QuotationID")]
#[table_name = "quotationver_a"]
pub struct QuotationVerA {
    pub	QuotationID	:	i32	,
    pub	VersionNo	:	i32	,
    pub	Approved	:	String	,
    pub	DateApproved	:	NaiveDateTime	,
    pub	ApproveAgree	:	String	,
    pub	ApproveSuggestion	:	String	,
    pub	ApprovedEmployee	:	String	,
    pub	Quoted	:	String	,
    pub	QuotationEmployee	:	String	,
    pub	DateQuoted	:	NaiveDateTime	,
    pub	ClientApproved	:	String	,
    pub	ApproveSuggestionClient	:	String	,
    pub	ContactPersonApproved	:	i32	,
    pub	DateClientApproved	:	NaiveDateTime	,
    pub	Remark	:	String	,
    pub	Other1	:	String	,
    pub	Other2	:	String	,
    pub	Other3	:	String	,
    pub	Other4	:	String	,
    pub	Other5	:	String	,
    pub	Other6	:	String	,
    pub	Other7	:	String	,
    pub	Other8	:	String	,
    pub	Other9	:	String	,
    pub	Other10	:	String	,
    pub	Currency	:	String	,
    pub	TotalAmount	:	Option<BigDecimal>	,
    pub	AdditionalTerms	:	String	,
    pub	PaymentMethodCode	:	String	,
    pub	PaymentMethod	:	String	,
}

#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, PartialEq,Default)]
#[primary_key(QuotationID)]
#[table_name = "quotationver_b"]
pub struct QuotationVerB {     
    pub	QuotationID	:	i32,
    pub	VersionNo	:	i32	,
    pub	BreachDuty	:	String	,
    pub	BreachSolve	:	String	,
    pub	QuotationSeed	:	String	,
    pub	Possibility	:	f64,
    pub	Suppliers	:	String	,
    pub	RequestPrepareDelivery	:	bool	,
    pub	DeliveryInstruction	:	bool	,
    pub	RequestApprovePrepareGoods	:	bool	,
    pub	RequestApprovePrepareGoodsBy	:	String	,
    pub	DateRequestApprovePrepareGoods	:	NaiveDateTime	,
    pub	ApprovePrepareGoods	:	bool	,
    pub	ApprovePrepareGoodsAgree	:	bool	,
    pub	DateApprovePrepareGoods	:	NaiveDateTime	,
    pub	ApprovePrepareGoodsBy	:	String	,
    pub	RequestApproveDelivery	:	bool	,
    pub	RequestApproveDeliveryBy	:	String	,
    pub	DateRequestApproveDelivery	:	NaiveDateTime	,
    pub	ApproveDelivery	:	bool	,
    pub	ApproveDeliveryAgree	:	bool	,
    pub	DateApproveDelivery	:	NaiveDateTime	,
    pub	ApproveDeliveryBy	:	String	,
}


#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, Associations, PartialEq, Default)]
#[primary_key(QuotationID,VersionNo,ProjectNo)]
#[belongs_to(QuotationVerA, foreign_key = "QuotationID")]
#[table_name = "quotationverproject_a"]
pub struct QuotationVerProjectA {
    pub	QuotationID	:	i32	,
    pub	VersionNo	:	i32	,
    pub	ProjectNo	:	i32	,
    pub	TotalAmount	:	BigDecimal	,
    pub	TotalCostAmount	:	BigDecimal	,
    pub	Margin	:	BigDecimal	,
    pub	Profit	:	BigDecimal	,
    pub	Remark	:	String	,
    pub	Other1	:	String	,
    pub	Other2	:	String	,
    pub	Other3	:	String	,
    pub	Other4	:	String	,
    pub	Other5	:	String	,
    pub	Other6	:	String	,
    pub	Other7	:	String	,
    pub	Other8	:	String	,
    pub	Other9	:	String	,
    pub	Other10	:	String	,
    pub	InvoiceInsured	:	String	,
    pub	CommissionAmount	:	BigDecimal	,
    pub	TotalAmountSay	:	String	,
    pub	DisplayDetailItems	:	bool	,
    pub	OtherCostAmount	:	BigDecimal	,
    pub	ShareOtherCost	:	bool	,
    pub	CurrencyCost	:	String	,
    pub	TotalCostFixCurrency	:	BigDecimal	,
    pub	IncludeTax	:	bool	,
    pub	TaxRate	:	BigDecimal	,
}


#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, Associations, PartialEq, Default)]
#[primary_key(QuotationID,VersionNo,ProjectNo)]
#[belongs_to(QuotationVerA, foreign_key = "QuotationID")]
#[table_name = "quotationverproject_b"]
pub struct QuotationVerProjectB {    
    pub	QuotationID	:	i32	,
    pub	VersionNo	:	i32	,
    pub	ProjectNo	:	i32	,
    pub	CalcCommWithRate	:	bool	,
    pub	CommissionRate	:	BigDecimal	,
    pub	SettledCommission	:	bool	,
    pub	FreightCalculatedMethod	:	String	,
    pub	FreightRate	:	BigDecimal	,
    pub	AmountFreight	:	BigDecimal	,
    pub	PremiumRate	:	BigDecimal	,
    pub	AmountInsurance	:	BigDecimal	,
    pub	AmountTarrif	:	BigDecimal	,
    pub	AmountReturnTax	:	BigDecimal	,
    pub	AmountPrePayment	:	BigDecimal	,
    pub	AmountDeliveryPayment	:	BigDecimal	,
    pub	AmountPaymentCancelled	:	BigDecimal	,
    pub	IsBorrowCase	:	bool	,
    pub	TotalAmountCase	:	BigDecimal	,
    pub	PriceIncludeFreight	:	bool	,
    pub	HasPrePayment	:	bool	,
    pub	TotalAmountNoTax	:	BigDecimal	,
    pub	TotalAmountTax	:	BigDecimal	,
    pub	AmountCreatePayment	:	BigDecimal	,
    pub	AmountRecivedPayment	:	BigDecimal	,
    pub	DiscountRate	:	BigDecimal	,
    pub	DiscountAmount	:	BigDecimal	,
    pub	Deposit	:	BigDecimal	,
    pub	FixDeposit	:	bool	,        
}


#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, Associations, PartialEq, Default)]
#[primary_key(QuotationID,ProjectNo)]
#[belongs_to(QuotationA, foreign_key = "QuotationID")]
#[table_name = "project_a"]
pub struct ProjectA {
    pub	ID	:	i32	,
    pub	QuotationID	:	i32	,
    pub	ProjectNo	:	i32	,
    pub	RFQID	:	i32	,
    pub	RFQProjectNo	:	i32	,
    pub	ProjectName	:	String	,
    pub	SubProjectFlag	:	String	,
    pub	JobNo	:	String	,
    pub	StatusQuotation	:	String	,
    pub	StatusCheckedBy	:	String	,
    pub	DateCheckStatus	:	NaiveDateTime	,
    pub	LostReason	:	String	,
    pub	WonReason	:	String	,
    pub	PreferDateDelivery	:	NaiveDateTime	,
    pub	Deliveried	:	String	,
    pub	DelieveryCheckedBy	:	String	,
    pub	DateDelivery	:	NaiveDateTime	,
    pub	JobCreated	:	String	,
    pub	JobCreatedBy	:	String	,
    pub	DateJobCreatedBy	:	NaiveDateTime	,
    pub	Remark	:	String	,
    pub	Other1	:	String	,
    pub	Other2	:	String	,
    pub	Other3	:	String	,
    pub	Other4	:	String	,
    pub	Other5	:	String	,
    pub	Other6	:	String	,
    pub	Other7	:	String	,
    pub	Other8	:	String	,
    pub	Other9	:	String	,
    pub	Other10	:	String	,
}


#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, Associations, PartialEq, Default)]
#[primary_key(QuotationID,ProjectNo)]
#[belongs_to(QuotationVerA, foreign_key = "QuotationID")]
#[table_name = "project_b"]
pub struct ProjectB {    
    pub	QuotationID	:	i32	,
    pub	ProjectNo	:	i32	,
    pub	Other11	:	String	,
    pub	Other12	:	String	,
    pub	Other13	:	String	,
    pub	Other14	:	String	,
    pub	Other15	:	String	,
    pub	OrderSeed	:	String	,
    pub	DateShippingInstruct	:	NaiveDateTime	,
    pub	DateShippingAdvise	:	NaiveDateTime	,
    pub	Consignee	:	String	,
    pub	Documents	:	String	,
    pub	ShippingMarks	:	String	,
    pub	Notity	:	String	,
    pub	ShippingRemark	:	String	,
    pub	PostScript	:	String	,
    pub	LoadingPort	:	String	,
    pub	DestPort	:	String	,
    pub	ContainerNo	:	String	,
    pub	OBL	:	String	,
    pub	JobType	:	String	,
    pub	ShippingMethod	:	String	,
    pub	ServiceType	:	String	,
    pub	LoadingType	:	String	,
    pub	DescriptionOfShipment	:	String	,
    pub	SizeOfShipment	:	String	,
    pub	OriginCity	:	String	,
    pub	OriginCountry	:	String	,
    pub	DestCity	:	String	,
    pub	DestCountry	:	String	,
}


#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, Associations, PartialEq, Default)]
#[primary_key(QuotationID,ProjectNo)]
#[belongs_to(QuotationVerA, foreign_key = "QuotationID")]
#[table_name = "project_c"]
pub struct ProjectC {        
    pub	QuotationID	:	i32	,
    pub	ProjectNo	:	i32	,
    pub	VesselName	:	String	,
    pub	VoyageNo	:	String	,
    pub	ETD	:	NaiveDateTime	,
    pub	ETA	:	NaiveDateTime	,
    pub	ShippingAgent	:	String	,
    pub	AdviseRemark	:	String	,
    pub	TotalQtyPackages	:	f64	,
    pub	SayTotalQtyPackages	:	String	,
    pub	UseTonAsWeightUnit	:	bool	,
    pub	TotalNetWeight	:	f64	,
    pub	SayTotalNetWeight	:	String	,
    pub	TotalGrossWeight	:	f64	,
    pub	SayTotalGrossWeight	:	String	,
    pub	TotalVolume	:	f64	,
    pub	SayTotalVolume	:	String	,
    pub	DescriptionOfPacking	:	String	,
    pub	ShippingTime	:	String	,
    pub	DeliveryType	:	String	,
    pub	DeliveryTerm	:	String	,
    pub	PurchaserOrderNo	:	String	,
    pub	Origin	:	String	,
    pub	PaymentMethodCode	:	String	,
    pub	PaymentMethod	:	String	,
    pub	DealAsCommission	:	bool	,       
}

#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, Associations, PartialEq, Default)]
#[primary_key(QuotationID,ProjectNo)]
#[belongs_to(QuotationA, foreign_key = "QuotationID")]
#[table_name = "project2_a"]
pub struct Project2A {
    pub	ID	:	i32	,
    pub	QuotationID	:	i32	,
    pub	ProjectNo	:	i32	,
    pub	RFQID	:	i32	,
    pub	RFQProjectNo	:	i32	,
    pub	CountryDestContact	:	String	,
    pub	AddressDestContact	:	String	,
    pub	CityDestContact	:	String	,
    pub	ProvinceDestContact	:	String	,
    pub	PostCodeDestContact	:	String	,
    pub	ContactDestContact	:	String	,
    pub	TelDestContact	:	String	,
    pub	FaxDestContact	:	String	,
    pub	HomeTelDestContact	:	String	,
    pub	MobileDestContact	:	String	,
    pub	EmailDestContact	:	String	,
    pub	CountryDestDelivery	:	String	,
    pub	AddressDestDelivery	:	String	,
    pub	CityDestDelivery	:	String	,
    pub	ProvinceDestDelivery	:	String	,
    pub	PostCodeDestDelivery	:	String	,
    pub	ContactDestDelivery	:	String	,
    pub	TelDestDelivery	:	String	,
    pub	FaxDestDelivery	:	String	,
    pub	HomeTelDestDelivery	:	String	,
    pub	MobileDestDelivery	:	String	,
    pub	EmailDestDelivery	:	String	,
}

#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, Associations, PartialEq, Default)]
#[primary_key(QuotationID,ProjectNo)]
#[belongs_to(QuotationA, foreign_key = "QuotationID")]
#[table_name = "project2_b"]
pub struct Project2B {    
    pub	QuotationID	:	i32	,
    pub	ProjectNo	:	i32	,
    pub	Allowance	:	String	,
    pub	Storage	:	bool	,
    pub	StorageAt	:	String	,
    pub	LostToCode	:	String	,
    pub	LostToName	:	String	,
    pub	DestAgentCode	:	String	,
    pub	DestAgent	:	String	,
    pub	Insurance	:	String	,
    pub	Inspection	:	String	,
    pub	Container20Qty	:	i32	,
    pub	Container40Qty	:	i32	,
    pub	Container40HQty	:	i32	,
    pub	Container45HQty	:	i32	,
    pub	TruckType	:	String	,
    pub	TruckQty	:	i32	,        
        
}

#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, Associations, PartialEq, Default)]
#[primary_key(QuotationID,VersionNo,ProjectNo,ItemNo)]
#[belongs_to(QuotationVerProjectA, foreign_key = "QuotationID")]
#[table_name = "quotationitem_a"]
pub struct QuotationItemA
{
    pub	QuotationID	:	i32	,
    pub	VersionNo	:	i32	,
    pub	ProjectNo	:	i32	,
    pub	ItemNo	:	i32	,
    pub	Number	:	i32	,
    pub	Article	:	String	,
    pub	ProductCode	:	String	,
    pub	ProductName	:	String	,
    pub	Brand	:	String	,
    pub	ActivityCostCode	:	String	,
    pub	Price	:	BigDecimal	,
    pub	Quantity	:	BigDecimal	,
    pub	UnitOfMeasure	:	String	,
    pub	QuoteAmount	:	BigDecimal	,
    pub	EstUnitCost	:	BigDecimal	,
    pub	EstAmount	:	BigDecimal	,
    pub	ReferNo	:	i32	,
    pub	ItemType	:	i32	,
    pub	Remark	:	String	,
    pub	Other1	:	String	,
    pub	Other2	:	String	,
    pub	Other3	:	String	,
    pub	Other4	:	String	,
    pub	Other5	:	String	,
    pub	RequestID	:	i32	,
    pub	RequestVersionNo	:	i32	,
    pub	RequestProjectNo	:	i32	,
    pub	RequestItemNo	:	i32	,
    pub	QtyInvoiced	:	f64	,
    pub	QtyDeliveried	:	f64	,
    pub	QtyDeliveriedActual	:	f64	,
}

#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, Associations, PartialEq, Default)]
#[primary_key(QuotationID,VersionNo,ProjectNo,ItemNo)]
#[belongs_to(QuotationVerProjectA, foreign_key = "QuotationID")]
#[table_name = "quotationitem_b"]
pub struct QuotationItemB
{    
    pub	QuotationID	:	i32	,
    pub	VersionNo	:	i32	,
    pub	ProjectNo	:	i32	,
    pub	ItemNo	:	i32	,
    pub	UnitBase	:	String	,
    pub	ChangeUnit	:	String	,
    pub	UnitFactor1	:	f64	,
    pub	UnitFactor2	:	f64	,
    pub	TaxRate	:	f64	,
    pub	DateDelivery	:	NaiveDateTime	,
    pub	Comission	:	f64	,
    pub	ComissionAmount	:	f64	,
    pub	QtyOfPackages	:	f64	,
    pub	PackingType	:	String	,
    pub	NetWeightPerPackage	:	f64	,
    pub	TotalNetWeight	:	f64	,
    pub	GrossWeightPerPackage	:	f64	,
    pub	TotalGrossWeight	:	f64	,
    pub	BatchNo	:	String	,
    pub	StockUnitCost	:	BigDecimal	,
    pub	StockCostAmount	:	BigDecimal	,
    // pub	WareHouseCode	:	String	,
    pub	ProducerCode	:	String	,
    pub	Producer	:	String	,
    pub	IsOrigin	:	bool	,
    pub	QtyPrepared	:	BigDecimal	,
    pub	QtyCanDelivery	:	BigDecimal	,
    // pub	IOJobNo	:	String	,
    pub	CustosmNoOfGoods	:	String	,
    pub	Measurement	:	String	,
    pub	TotalVolume	:	BigDecimal	,
    pub	ReturnTaxRate	:	BigDecimal	,
    pub	AmountReturnTax	:	BigDecimal	,
    // pub	TarrifRate	:	BigDecimal	,
    // pub	AmountTarrif	:	BigDecimal	,
}

#[allow(non_snake_case)]
#[derive(Identifiable, Debug, Clone, Queryable, Associations, PartialEq, Default)]
#[primary_key(QuotationID,VersionNo,ProjectNo,ItemNo)]
#[belongs_to(QuotationVerProjectA, foreign_key = "QuotationID")]
#[table_name = "quotationitem_c"]
pub struct QuotationItemC
{    
    pub	QuotationID	:	i32	,
    pub	VersionNo	:	i32	,
    pub	ProjectNo	:	i32	,
    pub	ItemNo	:	i32	,
    pub	PriceCase	:	BigDecimal	,
    pub	AmountCase	:	BigDecimal	,
    pub	PurchaseInquiryNo	:	String	,
    pub	QtyMRPImported	:	BigDecimal	,
    pub	QtyProductionImported	:	BigDecimal	,
    pub	QtyUseStock	:	BigDecimal	,
    pub	QtyArrangedUseStock	:	BigDecimal	,
    pub	QtyDeliveriedUseStock	:	BigDecimal	,
    pub	QtyPerPackage	:	BigDecimal	,
    pub	QtyPerPackageInner	:	BigDecimal	,
    pub	BOMCode	:	String	,
    pub	ProductCodeClient	:	String	,
    // pub	CaseNo	:	String	,
    pub	ProductNameEnglish	:	String	,
    // pub	BarCode	:	String	,
    // pub	CSR	:	String	,
    // pub	PricePerCase	:	BigDecimal	,
    pub	QuotationNo	:	String	,
    pub	IsProduction	:	bool	,
    pub	JobNo	:	String	,
    pub	PriceNoTax	:	BigDecimal	,
    pub	AmountNoTax	:	BigDecimal	,
    pub	AmountTax	:	BigDecimal	,
    pub	CaseVolume	:	BigDecimal	,
    pub	QtyForecast	:	BigDecimal	,
    pub	ChineseRemark	:	String	,
    pub	ServiceProjectID	:	i32	,
    pub	PurchasePrice	:	BigDecimal	,
    pub	PurchaseCurrency	:	String	,
    pub	DiscountPerItem	:	BigDecimal	,

}