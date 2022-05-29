extern crate num_traits as traits;

use traits::{FromPrimitive, ToPrimitive};
use bigdecimal::BigDecimal;
use crate::safe::AutocommitOn;

use diesel::prelude::*;
use diesel::dsl::*;
use diesel_odbc::connection::RawConnection;

use diesel_odbc::models::*;
use diesel_odbc::schema::quotation::dsl as qa;
use diesel_odbc::schema::quotation2::dsl as q2a;

use diesel_odbc::schema::project::dsl as pa;
use diesel_odbc::schema::project2::dsl as p2a;

use diesel_odbc::schema::quotationver::dsl as qva;
use diesel_odbc::schema::quotationverproject::dsl as qvpa;
use diesel_odbc::schema::quotationitem::dsl as qia;


#[derive(Debug)]
pub struct Quotation{    
    pub fields: QuotationA,
    // pub fields_a: QuotationA,
    // pub fields_b: QuotationB,
    // pub fields_c: QuotationC,
    pub fields2: Quotation2,
    pub list_quotation_ver : Vec<QuotationVer>,
    pub list_project : Vec<Project>,
    pub current_version : *mut QuotationVer,
    pub status : Status,
}


#[derive(Debug)]
pub struct Project{    
    pub fields: ProjectA,    
    pub fields2: Project2A,    
    pub quotation : *mut Quotation,
    pub current_ver_project : *mut QuotationVerProject,
    pub status : Status,
}

#[derive(Debug)]
pub struct QuotationVer{    
    pub fields: QuotationVerA,    
    pub list_quotation_ver_project : Vec<QuotationVerProject>,
    pub quotation : *mut Quotation,
    pub status : Status,
}

#[derive(Debug)]
pub struct QuotationVerProject{    
    pub fields: QuotationVerProjectA,    
    pub project : *mut Project,    
    pub list_quotation_item : Vec<QuotationItem>,  
    pub quotation_ver : *mut QuotationVer,
    pub status : Status,
}

#[derive(Debug)]
pub struct QuotationItem{    
    pub fields: QuotationItemA,    
    pub status : Status,
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

#[derive(Default, Debug)]
pub struct Status{
    pub is_creating : bool,
    pub is_deleted : bool,
}

impl Status{
    pub fn save(&mut self){
        if self.is_creating{
            self.is_creating = false;
        }

        if self.is_deleted{
            self.is_deleted = false;
        }
    }
}


impl Quotation{
    pub fn new() -> Self {
        let mut quotation = Self{
            fields : Default::default(),
            fields2: Default::default(),
            list_quotation_ver: Vec::new(),
            list_project : Vec::new(),
            current_version : 0 as *mut QuotationVer,
            status : Default::default(),
        };        

        let quotation_ptr = &mut quotation as *mut Quotation;

        quotation.list_project.push(Project{
            fields : Default::default(),
            fields2 : Default::default(),
            quotation : quotation_ptr,
            current_ver_project : 0 as *mut QuotationVerProject,
            status : Default::default(),
        });

        let project = quotation.list_project.get_mut(0).unwrap();

        let mut version = QuotationVer{
            fields : Default::default(),
            list_quotation_ver_project : Vec::new(),
            quotation : quotation_ptr,
            status : Default::default(),
        };
        version.fields.VersionNo = 1;

        quotation.list_quotation_ver.push(version);
        let version = quotation.list_quotation_ver.get_mut(0).unwrap();
        version.fields.VersionNo = 1;
        quotation.current_version = version;

        let version_ptr = version as *mut QuotationVer;

        version.list_quotation_ver_project.push(
            QuotationVerProject {
            fields: Default::default(),
            project : project,
            list_quotation_item : Vec::new(),            
            quotation_ver : version_ptr,
            status : Default::default(),
        });

        let ver_project = version.list_quotation_ver_project.get_mut(0).unwrap();
        project.current_ver_project = ver_project as *mut QuotationVerProject;

        quotation        
    }
    
    pub fn new_sales_order(user : User) -> Self {
        let mut quotation = Self::new();        
        
        quotation.fields.CreateBy = user.user_code;
        quotation.fields.OfficeCode = user.current_office.office_code.clone();
        quotation.fields.CreateDate = chrono::Utc::now().naive_utc();
        
        quotation.status.is_creating = true;
        quotation.list_project.iter_mut().for_each(|project|project.status.is_creating = true);        
        quotation.list_quotation_ver.iter_mut().for_each(|version|{
            version.status.is_creating = true;
            version.list_quotation_ver_project.iter_mut().for_each(|ver_project|{
                ver_project.status.is_creating = true;
            });
        }); 

        quotation        
    }


    pub fn load<'env>(conn: &mut RawConnection<'env, AutocommitOn>, quotation_id : i32)->Self{
        
        let mut q = qa::quotation.filter(qa::QuotationID.eq(quotation_id)).load::<QuotationA>(conn).unwrap();
        let mut q2 = q2a::quotation2.filter(q2a::QuotationID.eq(quotation_id)).load::<Quotation2>(conn).unwrap();

        let p_a = pa::project.filter(pa::QuotationID.eq(quotation_id)).load::<ProjectA>(conn).unwrap();
        let mut p2_a = p2a::project2.filter(p2a::QuotationID.eq(quotation_id)).load::<Project2A>(conn).unwrap();

        let qv_a = qva::quotationver.filter(qva::QuotationID.eq(quotation_id)).load::<QuotationVerA>(conn).unwrap();
        let mut qvp_a = qvpa::quotationverproject.filter(qvpa::QuotationID.eq(quotation_id)).load::<QuotationVerProjectA>(conn).unwrap();

        let mut qi_a = qia::quotationitem.filter(qia::QuotationID.eq(quotation_id)).load::<QuotationItemA>(conn).unwrap();

        let mut quotation = Quotation::new();
        if q.len() == 1{
            quotation.fields = q.pop().unwrap();
        }
        if q2.len() == 1{
            quotation.fields2 = q2.pop().unwrap();
        }

        let quotation_ptr = &mut quotation as * mut Quotation;    

        p_a.into_iter().for_each(|pa|{
            let mut p2a : Option<Project2A> = None;
            for i in 0..p2_a.len(){
                if p2_a.get(i).unwrap().ProjectNo == pa.ProjectNo{
                    p2a = Some(p2_a.remove(i));
                    break;
                }
            }

            let project = Project{
                fields : pa,
                fields2: p2a.unwrap(),    
                quotation : quotation_ptr,
                current_ver_project : 0 as *mut QuotationVerProject,
                status : Default::default(),
            };
            
            quotation.list_project.truncate(0);
            quotation.list_project.push(project);
        });

        qv_a.into_iter().for_each(|qva|{
            
            let mut version = QuotationVer{
                fields : qva,
                list_quotation_ver_project : Default::default(),    
                quotation : quotation_ptr,                
                status : Default::default(),
            };

            for i in 0..qvp_a.len(){
                let qvpa = qvp_a.remove(i);
    
                let project_ptr = quotation.list_project.iter_mut().find(|p|(p.fields.ProjectNo == qvpa.ProjectNo)).unwrap() as * mut Project;
                let mut ver_project = QuotationVerProject{
                    fields : qvpa,
                    list_quotation_item : Default::default(),    
                    project : project_ptr, 
                    quotation_ver : &mut version as * mut QuotationVer,  
                    status : Default::default(),
                };
                unsafe{(*project_ptr).current_ver_project = &mut ver_project as * mut QuotationVerProject;}

                for j in 0..qi_a.len(){
                    let qia = qi_a.remove(j);
        
                    let quotation_item = QuotationItem{
                        fields : qia,
                        status : Default::default(),
                    };
                    ver_project.list_quotation_item.push(quotation_item);    
                };
                
                version.list_quotation_ver_project.push(ver_project);    
            };
            
            quotation.list_quotation_ver.truncate(0);
            quotation.list_quotation_ver.push(version);

        });
        
        
        quotation
    }

    pub fn save<'env>(&mut self, conn : &mut RawConnection<'env, AutocommitOn>){
        if self.status.is_creating{
            let q_a = insert_into(qa::quotation).values(&self.fields).load::<QuotationA>(conn).unwrap();
            if q_a.len()==1{
                let q_a = q_a.get(0).unwrap();
                self.fields = q_a.clone();
                self.fields2.QuotationID = q_a.QuotationID;
            }

            let q2_a = insert_into(q2a::quotation2).values(&self.fields2).load::<Quotation2>(conn).unwrap();
            if q2_a.len()==1{
                let q2_a = q2_a.get(0).unwrap();
                self.fields2 = q2_a.clone();                
            }
        }
        else{
            update(qa::quotation.filter(qa::QuotationID.eq(self.fields.QuotationID))).set(&self.fields).load::<QuotationA>(conn).unwrap();
            update(q2a::quotation2.filter(q2a::QuotationID.eq(self.fields.QuotationID))).set(&self.fields2).load::<Quotation2>(conn).unwrap();
        }

        if self.status.is_creating{
            self.status.is_creating = false;
        }

        for version in &mut self.list_quotation_ver
        {
            version.fields.QuotationID = self.fields.QuotationID;
            if version.status.is_creating{
                let qv_a = insert_into(qva::quotationver).values(&version.fields).load::<QuotationVerA>(conn).unwrap();
                if qv_a.len()==1{
                    let qv_a = qv_a.get(0).unwrap();
                    version.fields = qv_a.clone();
                }
            }
            else
            {
                update(qva::quotationver.filter(qva::QuotationID.eq(self.fields.QuotationID))).set(&version.fields).load::<QuotationVerA>(conn).unwrap();
            }
            if version.status.is_creating{
                version.status.is_creating = false;
            }

            for ver_project in &mut version.list_quotation_ver_project
            {
                ver_project.fields.QuotationID = version.fields.QuotationID;
                ver_project.fields.VersionNo = version.fields.VersionNo;
                
                if ver_project.status.is_creating{
                    let qvp_a = insert_into(qvpa::quotationverproject).values(&ver_project.fields).load::<QuotationVerProjectA>(conn).unwrap();
                    if qvp_a.len()==1{
                        let qvp_a = qvp_a.get(0).unwrap();
                        ver_project.fields = qvp_a.clone();
                    }
                }
                else
                {
                    update(qvpa::quotationverproject.filter(qvpa::QuotationID.eq(ver_project.fields.QuotationID))).set(&ver_project.fields).load::<QuotationVerProjectA>(conn).unwrap();
                }

                for item in &mut ver_project.list_quotation_item
                {
                    item.fields.QuotationID = ver_project.fields.QuotationID;
                    item.fields.VersionNo = ver_project.fields.VersionNo;
                    item.fields.ProjectNo = ver_project.fields.ProjectNo;
                    
                    
                    if item.status.is_creating{
                        let qi_a = insert_into(qia::quotationitem).values(&item.fields).load::<QuotationItemA>(conn).unwrap();
                        if qi_a.len()==1{
                            let qi_a = qi_a.get(0).unwrap();
                            item.fields = qi_a.clone();
                        }
                    }
                    else
                    {
                        update(qia::quotationitem.filter(qia::QuotationID.eq(item.fields.QuotationID))).set(&item.fields).load::<QuotationItemA>(conn).unwrap();
                    }

                    if item.status.is_creating{
                        item.status.is_creating = false;
                    }
                }

                if ver_project.status.is_creating{
                    ver_project.status.is_creating = false;
                }

            }

        };      
        
         

        
    }

    pub fn calculate(&mut self){

        let version = unsafe{&mut *self.current_version};
        let _ = version.list_quotation_ver_project.iter_mut().map(|ver_project|
        {
            let tax_rate = BigDecimal::from(1i64) + &ver_project.fields.TaxRate;
            let include_tax = &ver_project.fields.IncludeTax;            
           
            let mut total_amount = BigDecimal::from(0i64);
            let mut total_amount_notax = BigDecimal::from(0i64);
            let mut commission_amount = BigDecimal::from(0i64);
            // let mut other_cost_amount = BigDecimal::from(0);
            let mut total_cost_amount = BigDecimal::from_f64(0.0).unwrap();
            let mut total_qty_packages = BigDecimal::from(0i64);
            let mut total_volume = BigDecimal::from(0i64);
            let mut total_gross_weight = BigDecimal::from(0i64);
            let mut total_net_weight = BigDecimal::from(0i64);

            let _ = ver_project.list_quotation_item.iter_mut().map(|item|
            {                
                if *include_tax {
                    if item.fields.PriceNoTax.to_f64().unwrap() != 0.0{
                        item.fields.Price = (&item.fields.PriceNoTax * &tax_rate).round(4);
                    }
                    else if item.fields.Price.to_f64().unwrap() != 0.0{
                        item.fields.PriceNoTax = (&item.fields.Price / &tax_rate).round(4);
                    }
                }
                else{
                    if item.fields.PriceNoTax.to_f64().unwrap() != 0.0{
                        item.fields.Price = item.fields.PriceNoTax.clone();
                    }
                    else if item.fields.Price.to_f64().unwrap() != 0.0{
                        item.fields.PriceNoTax = item.fields.Price.clone();
                    }
                }
                
                if item.fields.QtyPerPackage.to_f64().unwrap() != 0.0 {
                    item.fields.QtyOfPackages = (&item.fields.Quantity / &item.fields.QtyPerPackage).round(1);
                }

                item.fields.TotalGrossWeight = &item.fields.GrossWeightPerPackage * &item.fields.QtyOfPackages;
                item.fields.TotalNetWeight = &item.fields.GrossWeightPerPackage * &item.fields.QtyOfPackages;

                item.fields.AmountNoTax = (&item.fields.PriceNoTax * &item.fields.Quantity).round(2);
                item.fields.QuoteAmount = (&item.fields.Price * &item.fields.Quantity).round(2);
                item.fields.EstAmount = (&item.fields.EstUnitCost * &item.fields.Quantity).round(2);   
                
                item.fields.ComissionAmount = (&item.fields.Comission * &item.fields.Quantity).round(2);  
                item.fields.AmountReturnTax = (&item.fields.ReturnTaxRate * &item.fields.StockUnitCost).round(2);  

                total_amount += &item.fields.QuoteAmount;
                total_amount_notax += &item.fields.AmountNoTax;
                commission_amount += &item.fields.ComissionAmount;                
                total_cost_amount += &item.fields.EstAmount;

                total_qty_packages += &item.fields.QtyOfPackages;
                total_volume += &item.fields.TotalVolume;
                total_gross_weight += &item.fields.TotalGrossWeight;
                total_net_weight += &item.fields.TotalNetWeight;
            });
            
            ver_project.fields.TotalAmount = total_amount;
            ver_project.fields.TotalAmountNoTax = total_amount_notax;
            ver_project.fields.CommissionAmount = commission_amount;
            // ver_project.fields.OtherCostAmount = BigDecimal::from(0);
            ver_project.fields.TotalCostAmount = total_cost_amount;

            let project = unsafe{&mut *ver_project.project};
            project.fields.TotalQtyPackages = total_qty_packages.to_f64().unwrap();
            project.fields.TotalVolume = total_volume.to_f64().unwrap();
            project.fields.TotalGrossWeight = total_gross_weight.to_f64().unwrap();
            project.fields.TotalNetWeight = total_net_weight.to_f64().unwrap();

        });

    }

    pub fn confirm(&mut self, user : &User){

        if !self.fields.IsConfirmedSalesOrder
        {
            self.fields.ConfirmDate = chrono::Utc::now().naive_utc();
            self.fields.ConfirmedBy = user.user_code.clone();
            self.fields.IsConfirmedSalesOrder = true;
        }
        else
        {
            self.fields.ConfirmDate = chrono::Utc::now().naive_utc();
            self.fields.ConfirmedBy = user.user_code.clone();
            self.fields.IsConfirmedSalesOrder = false;
        }
    }

    pub fn check(&mut self, user : &User){
        if !self.fields.IsConfirmedSalesOrder{
            return;
        }

        if !self.fields.ApprovedSalesOrder
        {
            self.fields.DateApprovedSalesOrder = chrono::Utc::now().naive_utc();
            self.fields.ApprovedSalesOrderBy = user.user_code.clone();
            self.fields.ApprovedSalesOrder = true;            
        }
        else
        {
            self.fields.DateApprovedSalesOrder = chrono::Utc::now().naive_utc();
            self.fields.ApprovedSalesOrderBy = user.user_code.clone();
            self.fields.ApprovedSalesOrder = false;
        }
    }

    pub fn request_prepare_goods(&mut self, user : &User)-> i32 {
        let version = unsafe{&mut *self.current_version};
        if !self.fields.ApprovedSalesOrder && !version.fields.RequestApproveDelivery{
            return 0;
        }

        if self.fields.ApprovedSalesOrder && !version.fields.RequestApprovePrepareGoods
        {
            let mut b_have_available_project = false;

            let _ = self.list_project.iter().map(|project|{
                    //销售合同处于booking状态
                    if project.fields.StatusQuotation == StatusQuotation::BOOKED
                    {
                        b_have_available_project = true;                        
                    }                                
            });
            if !b_have_available_project{
                    return 5;
            }
            
            version.fields.DateRequestApprovePrepareGoods = chrono::Utc::now().naive_utc();
            version.fields.RequestApprovePrepareGoodsBy = user.user_code.clone();
            version.fields.RequestApprovePrepareGoods = true;   
            
            let mut not_receive_payment = false;
            let _ = version.list_quotation_ver_project.iter().map(|ver_project|
            {
                //预收款未到帐，需要审批
                if  ver_project.fields.AmountPrePayment > &ver_project.fields.AmountPaymentCancelled + BigDecimal::from(1i64){
                    not_receive_payment = true;
                }
            });
            if not_receive_payment{
                return 3;
            }

            //预收款已收到或没有预收款，直接设置可以备货标志
            if !version.fields.RequestApproveDelivery
            {
                //自动设置已审批状态
                version.fields.ApprovePrepareGoods = true;
                version.fields.ApprovePrepareGoodsAgree = true;
                version.fields.DateApprovePrepareGoods = chrono::Utc::now().naive_utc();
                version.fields.ApprovePrepareGoodsBy = user.user_code.clone();

                version.fields.RequestPrepareDelivery = true;

                //设置库存占用
                if !self.fields2.Closed{
                    // LockStock();
                }
            }                                    

            version.fields.DateRequestApproveDelivery = chrono::Utc::now().naive_utc();
            version.fields.RequestApproveDeliveryBy = user.user_code.clone();
            version.fields.RequestApproveDelivery = true;
        }
        //已申请备货,在没有请求发货的情况下可以取消请求备货
        else if version.fields.RequestApprovePrepareGoods && !version.fields.RequestApproveDelivery && !version.fields.DeliveryInstruction 
        {
            let mut error_code = 0;
            let _ = version.list_quotation_ver_project.iter().map(|ver_project|
            {
                let project = unsafe{&mut *ver_project.project}; 
                if project.fields2.Storage//只在以仓库做为周转方式的情况下计算
                {
                    //如果已建采购单不能取消备货请求，必须在采购先取消采购项
                    let str_sql = "select isnull(count(*),0) from SalesProduct2Purchase where QuotationID =".to_string() + &self.fields.QuotationID.to_string() + " and ProjectNo = " + &project.fields.ProjectNo.to_string();
                    println!("{}", str_sql);
                    // if ((int)BaseService.ExcuteSqlScalar(strSql) > 0)
                    //     throw new ExceptionBusiness(ExceptionBusiness.cstrSalesDisableCancelPrepardGoodsAfterCreatePurchase, "已生成采购单，不能取消备货，如要取消请先删除采购单。");

                    //已发货不能取消备货
                    let _ = ver_project.list_quotation_item.iter().map(|item|
                    {
                        if item.fields.QtyMRPImported > BigDecimal::from(0i64) {
                            error_code = 4;
                            //throw new ExceptionBusiness("已生成物料需求单，不能取消备货，如要取消请先删除物料需求单。");
                        }                            

                        if item.fields.QtyProductionImported > BigDecimal::from(0i64){
                            error_code = 5;
                            //throw new ExceptionBusiness("已生成生产通知单，不能取消备货，如要取消请先删除生产通知单。");
                        }

                        if item.fields.QtyDeliveriedActual > BigDecimal::from(0i64){
                            error_code = 7;
                            //throw new ExceptionBusiness(ExceptionBusiness.cstrSalesDisableCancelPrepardGoodsAfterCreateDelivery, "已发货，不能取消备货。");
                        }
                    });                    
                }
                else
                {
                    if project.fields.RFQID != 0 && project.fields.RFQProjectNo != 0
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

            version.fields.DateRequestApprovePrepareGoods = chrono::Utc::now().naive_utc();
            version.fields.RequestApprovePrepareGoodsBy = user.user_code.clone();
            version.fields.RequestApprovePrepareGoods = false;
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