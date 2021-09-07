extern crate num_traits as traits;
use traits::{FromPrimitive, ToPrimitive};
use bigdecimal::BigDecimal;

use super::models::*;

#[derive(Debug)]
pub struct Quotation{    
    pub fields_a: QuotationA,
    pub fields_b: QuotationB,
    pub fields_c: QuotationC,
    pub fields2_a: Quotation2A,
    pub fields2_b: Quotation2B,
    pub list_quotation_ver : Vec<QuotationVer>,
    pub list_project : Vec<Project>,
    pub current_version : *mut QuotationVer,
}


#[derive(Debug)]
pub struct Project{    
    pub fields_a: ProjectA,    
    pub fields_b: ProjectB,
    pub fields_c: ProjectC,
    pub fields2_a: Project2A,    
    pub fields2_b: Project2B,
    pub quotation : *mut Quotation,
    pub current_ver_project : *mut QuotationVerProject,

}

#[derive(Debug)]
pub struct QuotationVer{    
    pub fields_a: QuotationVerA,    
    pub fields_b: QuotationVerB,
    pub list_quotation_ver_project : Vec<QuotationVerProject>,
    pub quotation : *mut Quotation,
}

#[derive(Debug)]
pub struct QuotationVerProject{    
    pub fields_a: QuotationVerProjectA,    
    pub fields_b: QuotationVerProjectB,
    pub project : *mut Project,    
    pub list_quotation_item : Vec<QuotationItem>,  
    pub quotation_ver : *mut QuotationVer,
}

#[derive(Debug)]
pub struct QuotationItem{    
    pub fields_a: QuotationItemA,    
    pub fields_b: QuotationItemB,
    pub fields_c: QuotationItemC,    
}

pub struct Office{    
    pub office_code : String,
    pub office_name : String,
}
pub struct User<'a>{
    pub user_code : String,
    pub user_name : String,
    pub office : &'a Office,
    pub current_office : &'a Office,
}

impl Quotation{
    pub fn new(user : User) -> Self {
        let mut quotation = Quotation{
            fields_a : Default::default(),
            fields_b : Default::default(),
            fields_c : Default::default(),
            fields2_a: Default::default(),
            fields2_b: Default::default(),
            list_quotation_ver: Vec::new(),
            list_project : Vec::new(),
            current_version : 0 as *mut QuotationVer,
        };        

        let quotation_ptr = &mut quotation as *mut Quotation;

        quotation.fields_b.CreateBy = user.user_code;
        quotation.fields_b.OfficeCode = user.current_office.office_code.clone();
        quotation.fields_b.CreateDate = chrono::Utc::now().naive_utc();

        quotation.list_project.push(Project{
            fields_a : Default::default(),
            fields_b : Default::default(),
            fields_c : Default::default(),
            fields2_a : Default::default(),
            fields2_b : Default::default(),
            quotation : quotation_ptr,
            current_ver_project : 0 as *mut QuotationVerProject,
        });

        let project = quotation.list_project.get_mut(0).unwrap();

        let version = QuotationVer{
            fields_a : Default::default(),
            fields_b : Default::default(),
            list_quotation_ver_project : Vec::new(),
            quotation : quotation_ptr,
        };
        quotation.list_quotation_ver.push(version);
        let version = quotation.list_quotation_ver.get_mut(0).unwrap();
        version.fields_a.VersionNo = 1;
        quotation.current_version = version;

        let version_ptr = version as *mut QuotationVer;

        version.list_quotation_ver_project.push(
            QuotationVerProject {
            fields_a: Default::default(),
            fields_b: Default::default(),
            project : project,
            list_quotation_item : Vec::new(),            
            quotation_ver : version_ptr,
        });

        let ver_project = version.list_quotation_ver_project.get_mut(0).unwrap();
        let n = ver_project.list_quotation_item.len();        
        println!("item length:{}", n);


        println!("quotation={:?}", quotation);
        quotation        
    }
    
    pub fn calculate(&mut self){

        let version = unsafe{&mut *self.current_version};
        let _ = version.list_quotation_ver_project.iter_mut().map(|ver_project|
        {
            let tax_rate = BigDecimal::from(1) + &ver_project.fields_a.TaxRate;
            let include_tax = &ver_project.fields_a.IncludeTax;            
           
            let mut total_amount = BigDecimal::from(0);
            let mut total_amount_notax = BigDecimal::from(0);
            let mut commission_amount = BigDecimal::from(0);
            // let mut other_cost_amount = BigDecimal::from(0);
            let mut total_cost_amount = BigDecimal::from_f64(0.0).unwrap();
            let mut total_qty_packages = 0.0;
            let mut total_volume = 0.0;
            let mut total_gross_weight = 0.0;
            let mut total_net_weight = 0.0;

            let _ = ver_project.list_quotation_item.iter_mut().map(|item|
            {                
                if *include_tax {
                    if item.fields_c.PriceNoTax.to_f64().unwrap() != 0.0{
                        item.fields_a.Price = (&item.fields_c.PriceNoTax * &tax_rate).round(4);
                    }
                    else if item.fields_a.Price.to_f64().unwrap() != 0.0{
                        item.fields_c.PriceNoTax = (&item.fields_a.Price / &tax_rate).round(4);
                    }
                }
                else{
                    if item.fields_c.PriceNoTax.to_f64().unwrap() != 0.0{
                        item.fields_a.Price = item.fields_c.PriceNoTax.clone();
                    }
                    else if item.fields_a.Price.to_f64().unwrap() != 0.0{
                        item.fields_c.PriceNoTax = item.fields_a.Price.clone();
                    }
                }
                
                if item.fields_c.QtyPerPackage.to_f64().unwrap() != 0.0 {
                    item.fields_b.QtyOfPackages = (&item.fields_a.Quantity / &item.fields_c.QtyPerPackage).round(1).to_f64().unwrap();
                }

                item.fields_b.TotalGrossWeight = item.fields_b.GrossWeightPerPackage * item.fields_b.QtyOfPackages;
                item.fields_b.TotalNetWeight = item.fields_b.GrossWeightPerPackage * item.fields_b.QtyOfPackages;

                item.fields_c.AmountNoTax = (&item.fields_c.PriceNoTax * &item.fields_a.Quantity).round(2);
                item.fields_a.QuoteAmount = (&item.fields_a.Price * &item.fields_a.Quantity).round(2);
                item.fields_a.EstAmount = (&item.fields_a.EstUnitCost * &item.fields_a.Quantity).round(2);   
                
                item.fields_b.ComissionAmount = (&item.fields_b.Comission * &item.fields_a.Quantity.to_f64().unwrap() * 100.0).round() / 100.0;  
                item.fields_b.AmountReturnTax = (&item.fields_b.ReturnTaxRate * &item.fields_b.StockUnitCost).round(2);  

                total_amount += &item.fields_a.QuoteAmount;
                total_amount_notax += &item.fields_c.AmountNoTax;
                commission_amount += BigDecimal::from_f64(item.fields_b.ComissionAmount).unwrap();                
                total_cost_amount += &item.fields_a.EstAmount;

                total_qty_packages += item.fields_b.QtyOfPackages;
                total_volume += item.fields_b.TotalVolume.to_f64().unwrap();
                total_gross_weight += item.fields_b.TotalGrossWeight;
                total_net_weight += item.fields_b.TotalNetWeight;
            });
            
            ver_project.fields_a.TotalAmount = total_amount;
            ver_project.fields_b.TotalAmountNoTax = total_amount_notax;
            ver_project.fields_a.CommissionAmount = commission_amount;
            // ver_project.fields_a.OtherCostAmount = BigDecimal::from(0);
            ver_project.fields_a.TotalCostAmount = total_cost_amount;

            let project = unsafe{&mut *ver_project.project};
            project.fields_c.TotalQtyPackages = total_qty_packages;
            project.fields_c.TotalVolume = total_volume;
            project.fields_c.TotalGrossWeight = total_gross_weight;
            project.fields_c.TotalNetWeight = total_net_weight;

        });

    }

    pub fn confirm(&mut self, user : &User){

        if !self.fields_c.IsConfirmedSalesOrder
        {
            self.fields_c.ConfirmDate = chrono::Utc::now().naive_utc();
            self.fields_c.ConfirmedBy = user.user_code.clone();
            self.fields_c.IsConfirmedSalesOrder = true;
        }
        else
        {
            self.fields_c.ConfirmDate = chrono::Utc::now().naive_utc();
            self.fields_c.ConfirmedBy = user.user_code.clone();
            self.fields_c.IsConfirmedSalesOrder = false;
        }
    }

    pub fn check(&mut self, user : &User){
        if !self.fields_c.IsConfirmedSalesOrder{
            return;
        }

        if !self.fields_c.ApprovedSalesOrder
        {
            self.fields_c.DateApprovedSalesOrder = chrono::Utc::now().naive_utc();
            self.fields_c.ApprovedSalesOrderBy = user.user_code.clone();
            self.fields_c.ApprovedSalesOrder = true;            
        }
        else
        {
            self.fields_c.DateApprovedSalesOrder = chrono::Utc::now().naive_utc();
            self.fields_c.ApprovedSalesOrderBy = user.user_code.clone();
            self.fields_c.ApprovedSalesOrder = false;
        }
    }

    pub fn request_prepare_goods(&mut self, user : &User)-> i32 {
        let version = unsafe{&mut *self.current_version};
        if !self.fields_c.ApprovedSalesOrder && !version.fields_b.RequestApproveDelivery{
            return 0;
        }

        if self.fields_c.ApprovedSalesOrder && !version.fields_b.RequestApprovePrepareGoods
        {
            let mut b_have_available_project = false;

            let _ = self.list_project.iter().map(|project|{
                    //销售合同处于booking状态
                    if project.fields_a.StatusQuotation == StatusQuotation::BOOKED
                    {
                        b_have_available_project = true;                        
                    }                                
            });
            if !b_have_available_project{
                    return 5;
            }
            
            version.fields_b.DateRequestApprovePrepareGoods = chrono::Utc::now().naive_utc();
            version.fields_b.RequestApprovePrepareGoodsBy = user.user_code.clone();
            version.fields_b.RequestApprovePrepareGoods = true;   
            
            let mut not_receive_payment = false;
            let _ = version.list_quotation_ver_project.iter().map(|ver_project|
            {
                //预收款未到帐，需要审批
                if  ver_project.fields_b.AmountPrePayment > &ver_project.fields_b.AmountPaymentCancelled + BigDecimal::from(1){
                    not_receive_payment = true;
                }
            });
            if not_receive_payment{
                return 3;
            }

            //预收款已收到或没有预收款，直接设置可以备货标志
            if !version.fields_b.RequestApproveDelivery
            {
                //自动设置已审批状态
                version.fields_b.ApprovePrepareGoods = true;
                version.fields_b.ApprovePrepareGoodsAgree = true;
                version.fields_b.DateApprovePrepareGoods = chrono::Utc::now().naive_utc();
                version.fields_b.ApprovePrepareGoodsBy = user.user_code.clone();

                version.fields_b.RequestPrepareDelivery = true;

                //设置库存占用
                if !self.fields2_b.Closed{
                    // LockStock();
                }
            }                                    

            version.fields_b.DateRequestApproveDelivery = chrono::Utc::now().naive_utc();
            version.fields_b.RequestApproveDeliveryBy = user.user_code.clone();
            version.fields_b.RequestApproveDelivery = true;
        }
        //已申请备货,在没有请求发货的情况下可以取消请求备货
        else if version.fields_b.RequestApprovePrepareGoods && !version.fields_b.RequestApproveDelivery && !version.fields_b.DeliveryInstruction 
        {
            let mut error_code = 0;
            let _ = version.list_quotation_ver_project.iter().map(|ver_project|
            {
                let project = unsafe{&mut *ver_project.project}; 
                if project.fields2_b.Storage//只在以仓库做为周转方式的情况下计算
                {
                    //如果已建采购单不能取消备货请求，必须在采购先取消采购项
                    let str_sql = "select isnull(count(*),0) from SalesProduct2Purchase where QuotationID =".to_string() + &self.fields_a.QuotationID.to_string() + " and ProjectNo = " + &project.fields_a.ProjectNo.to_string();
                    println!("{}", str_sql);
                    // if ((int)BaseService.ExcuteSqlScalar(strSql) > 0)
                    //     throw new ExceptionBusiness(ExceptionBusiness.cstrSalesDisableCancelPrepardGoodsAfterCreatePurchase, "已生成采购单，不能取消备货，如要取消请先删除采购单。");

                    //已发货不能取消备货
                    let _ = ver_project.list_quotation_item.iter().map(|item|
                    {
                        if item.fields_c.QtyMRPImported > BigDecimal::from(0) {
                            error_code = 4;
                            //throw new ExceptionBusiness("已生成物料需求单，不能取消备货，如要取消请先删除物料需求单。");
                        }                            

                        if item.fields_c.QtyProductionImported > BigDecimal::from(0){
                            error_code = 5;
                            //throw new ExceptionBusiness("已生成生产通知单，不能取消备货，如要取消请先删除生产通知单。");
                        }

                        if item.fields_a.QtyDeliveriedActual > 0.0{
                            error_code = 7;
                            //throw new ExceptionBusiness(ExceptionBusiness.cstrSalesDisableCancelPrepardGoodsAfterCreateDelivery, "已发货，不能取消备货。");
                        }
                    });                    
                }
                else
                {
                    if project.fields_a.RFQID != 0 && project.fields_a.RFQProjectNo != 0
                    {
                        error_code = 8;
                        //throw new ExceptionBusiness(ExceptionBusiness.cstrSalesDisableCancelPrepardGoodsAfterCreatePurchase, "已生成采购单，不能取消备货，如要取消请先删除采购单。");
                    }
                }
            });

            if error_code != 0
            {
                return error_code;
            }

            version.fields_b.DateRequestApprovePrepareGoods = chrono::Utc::now().naive_utc();
            version.fields_b.RequestApprovePrepareGoodsBy = user.user_code.clone();
            version.fields_b.RequestApprovePrepareGoods = false;
        }

        return 1;
    }


}


#[allow(non_snake_case)]
pub mod StatusQuotation{
    pub const BOOKED : &'static str = "Booked";
    pub const LOST : &'static str = "Lost";
    pub const CANCEL : &'static str = "Cancel";
    pub const PENDING : &'static str = "Pending";
}