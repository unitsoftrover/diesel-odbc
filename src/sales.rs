extern crate num_traits as traits;
use std::borrow::BorrowMut;

use traits::{FromPrimitive, ToPrimitive};
use bigdecimal::BigDecimal;
use super::safe::AutocommitOn;

use diesel::prelude::*;
use diesel::debug_query;
use diesel::dsl::*;
use diesel_odbc::connection::RawConnection;
use diesel_odbc::Odbc;

use super::models::*;
use super::schema::quotation_a::dsl as qa;
use super::schema::quotation_b::dsl as qb;
use super::schema::quotation_c::dsl as qc;
use super::schema::quotation2_a::dsl as q2a;
use super::schema::quotation2_b::dsl as q2b;

use super::schema::project_a::dsl as pa;
use super::schema::project_b::dsl as pb;
use super::schema::project_c::dsl as pc;
use super::schema::project2_a::dsl as p2a;
use super::schema::project2_b::dsl as p2b;

use super::schema::quotationver_a::dsl as qva;
use super::schema::quotationver_b::dsl as qvb;
use super::schema::quotationverproject_a::dsl as qvpa;
use super::schema::quotationverproject_b::dsl as qvpb;

use super::schema::quotationitem_a::dsl as qia;
use super::schema::quotationitem_b::dsl as qib;
use super::schema::quotationitem_c::dsl as qic;


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
    pub status : Status,
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
    pub status : Status,
}

#[derive(Debug)]
pub struct QuotationVer{    
    pub fields_a: QuotationVerA,    
    pub fields_b: QuotationVerB,
    pub list_quotation_ver_project : Vec<QuotationVerProject>,
    pub quotation : *mut Quotation,
    pub status : Status,
}

#[derive(Debug)]
pub struct QuotationVerProject{    
    pub fields_a: QuotationVerProjectA,    
    pub fields_b: QuotationVerProjectB,
    pub project : *mut Project,    
    pub list_quotation_item : Vec<QuotationItem>,  
    pub quotation_ver : *mut QuotationVer,
    pub status : Status,
}

#[derive(Debug)]
pub struct QuotationItem{    
    pub fields_a: QuotationItemA,    
    pub fields_b: QuotationItemB,
    pub fields_c: QuotationItemC,    
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
            fields_a : Default::default(),
            fields_b : Default::default(),
            fields_c : Default::default(),
            fields2_a: Default::default(),
            fields2_b: Default::default(),
            list_quotation_ver: Vec::new(),
            list_project : Vec::new(),
            current_version : 0 as *mut QuotationVer,
            status : Default::default(),
        };        

        let quotation_ptr = &mut quotation as *mut Quotation;

        quotation.list_project.push(Project{
            fields_a : Default::default(),
            fields_b : Default::default(),
            fields_c : Default::default(),
            fields2_a : Default::default(),
            fields2_b : Default::default(),
            quotation : quotation_ptr,
            current_ver_project : 0 as *mut QuotationVerProject,
            status : Default::default(),
        });

        let project = quotation.list_project.get_mut(0).unwrap();

        let mut version = QuotationVer{
            fields_a : Default::default(),
            fields_b : Default::default(),
            list_quotation_ver_project : Vec::new(),
            quotation : quotation_ptr,
            status : Default::default(),
        };
        version.fields_a.VersionNo = 1;
        version.fields_b.VersionNo = 1;

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
            status : Default::default(),
        });

        let ver_project = version.list_quotation_ver_project.get_mut(0).unwrap();
        project.current_ver_project = ver_project as *mut QuotationVerProject;

        quotation        
    }
    
    pub fn new_sales_order(user : User) -> Self {
        let mut quotation = Self::new();        
        
        quotation.fields_b.CreateBy = user.user_code;
        quotation.fields_a.OfficeCode = user.current_office.office_code.clone();
        quotation.fields_b.CreateDate = chrono::Utc::now().naive_utc();
        
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


    pub fn load<'env>(conn: &RawConnection<'env, AutocommitOn>, quotation_id : i32)->Self{
        
        let mut q_a = qa::quotation_a.filter(qa::QuotationID.eq(quotation_id)).load::<QuotationA>(conn).unwrap();
        let mut q_b = qb::quotation_b.filter(qb::QuotationID.eq(quotation_id)).load::<QuotationB>(conn).unwrap();
        let mut q_c = qc::quotation_c.filter(qc::QuotationID.eq(quotation_id)).load::<QuotationC>(conn).unwrap();
        let mut q2_a = q2a::quotation2_a.filter(q2a::QuotationID.eq(quotation_id)).load::<Quotation2A>(conn).unwrap();
        let mut q2_b = q2b::quotation2_b.filter(q2b::QuotationID.eq(quotation_id)).load::<Quotation2B>(conn).unwrap();

        let p_a = pa::project_a.filter(pa::QuotationID.eq(quotation_id)).load::<ProjectA>(conn).unwrap();
        let mut p_b = pb::project_b.filter(pb::QuotationID.eq(quotation_id)).load::<ProjectB>(conn).unwrap();
        let mut p_c = pc::project_c.filter(pc::QuotationID.eq(quotation_id)).load::<ProjectC>(conn).unwrap();
        let mut p2_a = p2a::project2_a.filter(p2a::QuotationID.eq(quotation_id)).load::<Project2A>(conn).unwrap();
        let mut p2_b = p2b::project2_b.filter(p2b::QuotationID.eq(quotation_id)).load::<Project2B>(conn).unwrap();

        let qv_a = qva::quotationver_a.filter(qva::QuotationID.eq(quotation_id)).load::<QuotationVerA>(conn).unwrap();
        let mut qv_b = qvb::quotationver_b.filter(qvb::QuotationID.eq(quotation_id)).load::<QuotationVerB>(conn).unwrap();
        let mut qvp_a = qvpa::quotationverproject_a.filter(qvpa::QuotationID.eq(quotation_id)).load::<QuotationVerProjectA>(conn).unwrap();
        let mut qvp_b = qvpb::quotationverproject_b.filter(qvpb::QuotationID.eq(quotation_id)).load::<QuotationVerProjectB>(conn).unwrap();

        let mut qi_a = qia::quotationitem_a.filter(qia::QuotationID.eq(quotation_id)).load::<QuotationItemA>(conn).unwrap();
        let mut qi_b = qib::quotationitem_b.filter(qib::QuotationID.eq(quotation_id)).load::<QuotationItemB>(conn).unwrap();
        let mut qi_c = qic::quotationitem_c.filter(qic::QuotationID.eq(quotation_id)).load::<QuotationItemC>(conn).unwrap();

        let mut quotation = Quotation::new();
        if q_a.len() == 1{
            quotation.fields_a = q_a.pop().unwrap();
        }
        if q_b.len() == 1{
            quotation.fields_b = q_b.pop().unwrap();
        }
        if q_c.len() == 1{
            quotation.fields_c = q_c.pop().unwrap();
        }
        if q2_a.len() == 1{
            quotation.fields2_a = q2_a.pop().unwrap();
        }
        if q2_b.len() == 1{
            quotation.fields2_b = q2_b.pop().unwrap();
        }
        
        let quotation_ptr = &mut quotation as * mut Quotation;    

        p_a.into_iter().for_each(|pa|{
            let mut pb : Option<ProjectB> = None;
            for i in 0..p_b.len(){
                if p_b.get(i).unwrap().ProjectNo == pa.ProjectNo{
                    pb = Some(p_b.remove(i));
                    break;
                }
            }

            let mut pc : Option<ProjectC> = None;
            for i in 0..p_c.len(){
                if p_c.get(i).unwrap().ProjectNo == pa.ProjectNo{
                    pc = Some(p_c.remove(i));
                    break;
                }
            }

            let mut p2a : Option<Project2A> = None;
            for i in 0..p2_a.len(){
                if p2_a.get(i).unwrap().ProjectNo == pa.ProjectNo{
                    p2a = Some(p2_a.remove(i));
                    break;
                }
            }

            let mut p2b : Option<Project2B> = None;
            for i in 0..p2_b.len(){
                if p2_b.get(i).unwrap().ProjectNo == pa.ProjectNo{
                    p2b = Some(p2_b.remove(i));
                    break;
                }
            }

            let project = Project{
                fields_a : pa,
                fields_b : pb.unwrap(),
                fields_c : pc.unwrap(),
                fields2_a: p2a.unwrap(),    
                fields2_b: p2b.unwrap(),
                quotation : quotation_ptr,
                current_ver_project : 0 as *mut QuotationVerProject,
                status : Default::default(),
            };
            
            quotation.list_project.truncate(0);
            quotation.list_project.push(project);
        });

        qv_a.into_iter().for_each(|qva|{
            let mut qvb : Option<QuotationVerB> = None;
            for i in 0..qv_b.len(){
                if qv_b.get(i).unwrap().VersionNo == qva.VersionNo{
                    qvb = Some(qv_b.remove(i));
                    break;
                }
            }

            let mut version = QuotationVer{
                fields_a : qva,
                fields_b : qvb.unwrap(),            
                list_quotation_ver_project : Default::default(),    
                quotation : quotation_ptr,                
                status : Default::default(),
            };

            for i in 0..qvp_a.len(){
                let qvpa = qvp_a.remove(i);

                let mut qvpb : Option<QuotationVerProjectB> = None;
                for j in 0..qv_b.len(){
                    let current = qvp_b.get(j).unwrap();
                    if current.VersionNo == qvpa.VersionNo && current.ProjectNo == qvpa.ProjectNo
                    {
                        qvpb = Some(qvp_b.remove(i));
                        break;
                    }
                }
    
                let project_ptr = quotation.list_project.iter_mut().find(|p|(p.fields_a.ProjectNo == qvpa.ProjectNo)).unwrap() as * mut Project;
                let mut ver_project = QuotationVerProject{
                    fields_a : qvpa,
                    fields_b : qvpb.unwrap(),            
                    list_quotation_item : Default::default(),    
                    project : project_ptr, 
                    quotation_ver : &mut version as * mut QuotationVer,  
                    status : Default::default(),
                };
                unsafe{(*project_ptr).current_ver_project = &mut ver_project as * mut QuotationVerProject;}

                for j in 0..qi_a.len(){
                    let qia = qi_a.remove(j);
    
                    let mut qib : Option<QuotationItemB> = None;
                    for k in 0..qi_b.len(){
                        let current = qi_b.get(k).unwrap();
                        if current.VersionNo == qia.VersionNo 
                            && current.ProjectNo == qia.ProjectNo
                            && current.ItemNo == qia.ItemNo
                        {
                            qib = Some(qi_b.remove(k));
                            break;
                        }
                    }

                    let mut qic : Option<QuotationItemC> = None;
                    for k in 0..qi_c.len(){
                        let current = qi_c.get(k).unwrap();
                        if current.VersionNo == qia.VersionNo 
                            && current.ProjectNo == qia.ProjectNo
                            && current.ItemNo == qia.ItemNo
                        {
                            qic = Some(qi_c.remove(k));
                            break;
                        }
                    }
        
                    let quotation_item = QuotationItem{
                        fields_a : qia,
                        fields_b : qib.unwrap(),
                        fields_c : qic.unwrap(),
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

    pub fn save<'env>(&mut self, conn : &RawConnection<'env, AutocommitOn>){
        if self.status.is_creating{
            let q_a = insert_into(qa::quotation_a).values(&self.fields_a).load::<QuotationA>(conn).unwrap();
            if q_a.len()==1{
                let q_a = q_a.get(0).unwrap();
                self.fields_a = q_a.clone();
                self.fields_b.QuotationID = q_a.QuotationID;
                self.fields_c.QuotationID = q_a.QuotationID;
                self.fields2_a.QuotationID = q_a.QuotationID;
                self.fields2_b.QuotationID = q_a.QuotationID;
            }

            let q2_a = insert_into(q2a::quotation2_a).values(&self.fields2_a).load::<Quotation2A>(conn).unwrap();
            if q2_a.len()==1{
                let q2_a = q2_a.get(0).unwrap();
                self.fields2_a = q2_a.clone();                
            }

        }
        else{
            update(qa::quotation_a.filter(qa::QuotationID.eq(self.fields_a.QuotationID))).set(&self.fields_a).load::<QuotationA>(conn).unwrap();
            update(q2a::quotation2_a.filter(q2a::QuotationID.eq(self.fields_a.QuotationID))).set(&self.fields2_a).load::<Quotation2A>(conn).unwrap();
        }

        update(qb::quotation_b.filter(qb::QuotationID.eq(self.fields_a.QuotationID))).set(&self.fields_b).load::<QuotationB>(conn).unwrap();
        update(qc::quotation_c.filter(qc::QuotationID.eq(self.fields_a.QuotationID))).set(&self.fields_c).load::<QuotationC>(conn).unwrap();
        update(q2b::quotation2_b.filter(q2b::QuotationID.eq(self.fields_a.QuotationID))).set(&self.fields2_b).load::<Quotation2B>(conn).unwrap();
        
        for version in &mut self.list_quotation_ver
        {
            version.fields_a.QuotationID = self.fields_a.QuotationID;
            version.fields_b.QuotationID = self.fields_a.QuotationID;

            if version.status.is_creating{
                let qv_a = insert_into(qva::quotationver_a).values(&version.fields_a).load::<QuotationVerA>(conn).unwrap();
                if qv_a.len()==1{
                    let qv_a = qv_a.get(0).unwrap();
                    version.fields_a = qv_a.clone();
                    version.fields_b.VersionNo = qv_a.VersionNo;    
                }
            }
            else
            {
                update(qva::quotationver_a.filter(qva::QuotationID.eq(self.fields_a.QuotationID))).set(&version.fields_a).load::<QuotationVerA>(conn).unwrap();
            }
            update(qvb::quotationver_b.filter(qvb::QuotationID.eq(self.fields_a.QuotationID))).set(&version.fields_b).load::<QuotationVerB>(conn).unwrap();
        };

        
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
            let mut total_qty_packages = BigDecimal::from(0);
            let mut total_volume = BigDecimal::from(0);
            let mut total_gross_weight = BigDecimal::from(0);
            let mut total_net_weight = BigDecimal::from(0);

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
                    item.fields_b.QtyOfPackages = (&item.fields_a.Quantity / &item.fields_c.QtyPerPackage).round(1);
                }

                item.fields_b.TotalGrossWeight = &item.fields_b.GrossWeightPerPackage * &item.fields_b.QtyOfPackages;
                item.fields_b.TotalNetWeight = &item.fields_b.GrossWeightPerPackage * &item.fields_b.QtyOfPackages;

                item.fields_c.AmountNoTax = (&item.fields_c.PriceNoTax * &item.fields_a.Quantity).round(2);
                item.fields_a.QuoteAmount = (&item.fields_a.Price * &item.fields_a.Quantity).round(2);
                item.fields_a.EstAmount = (&item.fields_a.EstUnitCost * &item.fields_a.Quantity).round(2);   
                
                item.fields_b.ComissionAmount = (&item.fields_b.Comission * &item.fields_a.Quantity).round(2);  
                item.fields_b.AmountReturnTax = (&item.fields_b.ReturnTaxRate * &item.fields_b.StockUnitCost).round(2);  

                total_amount += &item.fields_a.QuoteAmount;
                total_amount_notax += &item.fields_c.AmountNoTax;
                commission_amount += &item.fields_b.ComissionAmount;                
                total_cost_amount += &item.fields_a.EstAmount;

                total_qty_packages += &item.fields_b.QtyOfPackages;
                total_volume += &item.fields_b.TotalVolume;
                total_gross_weight += &item.fields_b.TotalGrossWeight;
                total_net_weight += &item.fields_b.TotalNetWeight;
            });
            
            ver_project.fields_a.TotalAmount = total_amount;
            ver_project.fields_b.TotalAmountNoTax = total_amount_notax;
            ver_project.fields_a.CommissionAmount = commission_amount;
            // ver_project.fields_a.OtherCostAmount = BigDecimal::from(0);
            ver_project.fields_a.TotalCostAmount = total_cost_amount;

            let project = unsafe{&mut *ver_project.project};
            project.fields_c.TotalQtyPackages = total_qty_packages.to_f64().unwrap();
            project.fields_c.TotalVolume = total_volume.to_f64().unwrap();
            project.fields_c.TotalGrossWeight = total_gross_weight.to_f64().unwrap();
            project.fields_c.TotalNetWeight = total_net_weight.to_f64().unwrap();

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

                        if item.fields_a.QtyDeliveriedActual > BigDecimal::from(0){
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