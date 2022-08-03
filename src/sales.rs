extern crate num_traits as traits;
extern crate data_model;

use traits::{FromPrimitive, ToPrimitive};
use bigdecimal::BigDecimal;
use crate::safe::AutocommitOn;
use chrono::NaiveDateTime;

use diesel::prelude::*;
use diesel::dsl::*;
use diesel_odbc::connection::RawConnection;

use data_model::models::*;
use data_model::schema::quotation::dsl as qa;
use data_model::schema::quotation2::dsl as q2a;

use data_model::schema::project::dsl as pa;
use data_model::schema::project2::dsl as p2a;

use data_model::schema::quotationver::dsl as qva;
use data_model::schema::quotationverproject::dsl as qvpa;
use data_model::schema::quotationitem::dsl as qia;


#[derive(Debug)]
pub struct Quotation{    
    pub fields: QuotationA,
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

impl QuotationVer{
    pub fn all_items(&self) -> Vec<QuotationItem>{
        let all_items = Vec::new();
        // for ver_project in self.list_quotation_ver_project.iter(){
        //     all_items.push(ver_project.list_quotation_item);
        // }

        return all_items;
    }
}

#[cfg(feature="mysql")]
type TyConn = diesel::MysqlConnection;

#[cfg(feature="sqlite")]
type TyConn = diesel::SqliteConnection;

#[cfg(feature="postgres")]
type TyConn = diesel::PgConnection;

// #[cfg(feature="odbc")]
// type TyConn<'env> = diesel_odbc::connection::RawConnection<'env, odbc_safe::AutocommitOn>;

// type TyConn<'env> = diesel_odbc::connection::RawConnection<'env, odbc_safe::AutocommitOn>;

#[cfg(not(any(feature="mysql", feature="sqlite", feature="postgres")))]
type TyConn<'env> = diesel_odbc::connection::RawConnection<'env, odbc_safe::AutocommitOn>;

// type TyConn<'env> = diesel_odbc::connection::RawConnection<'env, odbc_safe::AutocommitOn>;


// cfg_if::cfg_if! {
    
//     if #[cfg(feature="sqlite")]{
//         type TyConn = diesel::SqliteConnection;
//     }
//     else if #[cfg(feature="mysql")]{
//         type TyConn = diesel::MysqlConnection;
//     }    
//     else if #[cfg(feature="postgres")]{
//         type TyConn = diesel::PgConnection;
//     }
//     else if #[cfg(feature="odbc")]{
//         type TyConn<'env> = diesel_odbc::connection::RawConnection<'env, odbc_safe::AutocommitOn>;
//     }
//     else{
//         type TyConn<'env> = diesel_odbc::connection::RawConnection<'env, odbc_safe::AutocommitOn>;
//     }
// }


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
        project.fields.ProjectNo = 1;
        project.fields2.ProjectNo = 1;

        let mut version = QuotationVer{
            fields : Default::default(),
            list_quotation_ver_project : Vec::new(),
            quotation : quotation_ptr,
            status : Default::default(),
        };
        version.fields.VersionNo = 1;
        quotation.fields.CurrentVersion = Some(1);

        quotation.list_quotation_ver.push(version);
        let version = quotation.list_quotation_ver.get_mut(0).unwrap();
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
    
    pub fn new_sales_order(user : &User) -> Self {
        let mut quotation = Self::new();        
        
        quotation.fields.CreateBy = user.user_code.clone();
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

        quotation.list_project.truncate(0);
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
            quotation.list_project.push(project);
        });

        quotation.list_quotation_ver.truncate(0);
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

        for project in &mut self.list_project
        {
            project.fields.QuotationID = self.fields.QuotationID;
            project.fields2.QuotationID = self.fields.QuotationID;
            project.fields2.ProjectNo = project.fields.ProjectNo;
            if project.status.is_creating{
                let p = insert_into(pa::project).values(&project.fields).load::<ProjectA>(conn).unwrap();
                if p.len()==1{
                    let p = p.get(0).unwrap();
                    project.fields = p.clone();
                }

                let p2 = insert_into(p2a::project2).values(&project.fields2).load::<Project2A>(conn).unwrap();
                if p2.len()==1{
                    let p2 = p2.get(0).unwrap();
                    project.fields2 = p2.clone();
                }
            }
            else
            {
                update(pa::project.filter(pa::QuotationID.eq(self.fields.QuotationID)).filter(pa::ProjectNo.eq(project.fields.ProjectNo))).set(&project.fields).load::<ProjectA>(conn).unwrap();
                update(p2a::project2.filter(p2a::QuotationID.eq(self.fields.QuotationID)).filter(p2a::ProjectNo.eq(project.fields2.ProjectNo))).set(&project.fields2).load::<Project2A>(conn).unwrap();
            }
            if project.status.is_creating{
                project.status.is_creating = false;
            }
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
                update(qva::quotationver.filter(qva::QuotationID.eq(self.fields.QuotationID)).filter(qva::VersionNo.eq(version.fields.VersionNo))).set(&version.fields).load::<QuotationVerA>(conn).unwrap();
            }
            if version.status.is_creating{
                version.status.is_creating = false;
            }

            for ver_project in &mut version.list_quotation_ver_project
            {
                ver_project.fields.QuotationID = version.fields.QuotationID;
                ver_project.fields.VersionNo = version.fields.VersionNo;
                unsafe{ver_project.fields.ProjectNo = (*ver_project.project).fields.ProjectNo;}
                
                if ver_project.status.is_creating{
                    let qvp_a = insert_into(qvpa::quotationverproject).values(&ver_project.fields).load::<QuotationVerProjectA>(conn).unwrap();
                    if qvp_a.len()==1{
                        let qvp_a = qvp_a.get(0).unwrap();
                        ver_project.fields = qvp_a.clone();
                    }
                }
                else
                {
                    update(qvpa::quotationverproject.filter(qvpa::QuotationID.eq(ver_project.fields.QuotationID)).filter(qvpa::VersionNo.eq(ver_project.fields.VersionNo)).filter(qvpa::ProjectNo.eq(ver_project.fields.ProjectNo))).set(&ver_project.fields).load::<QuotationVerProjectA>(conn).unwrap();
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
                        update(qia::quotationitem.filter(qia::QuotationID.eq(item.fields.QuotationID)).filter(qia::VersionNo.eq(item.fields.VersionNo)).filter(qia::ProjectNo.eq(item.fields.ProjectNo)).filter(qia::ItemNo.eq(item.fields.ItemNo))).set(&item.fields).load::<QuotationItemA>(conn).unwrap();
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

    pub fn create_contract1(&mut self, conn : &mut TyConn)    
    {
        conn.execute(&"select 1").unwrap();
        qa::quotation.filter(qa::QuotationID.eq(self.fields.QuotationID)).load::<QuotationA>(conn).unwrap();
    }


    /// <summary>
    /// 通过销售机会建立合同
    /// 设置合同状态，建立合同号
    /// </summary>
    /// <param name="user"></param>
    pub fn create_contract<'env>(&mut self, user : &User, conn : &mut TyConn)            
    {
        //释放销售机会占用库存
        // ReleaseStock();
        for project in self.list_project.iter_mut(){
            project.fields.JobCreated = "1".to_string();
            project.fields.StatusQuotation = StatusQuotation::BOOKED.to_string();
            project.fields.DateJobCreatedBy = chrono::Local::now().naive_local();
            project.fields.JobCreatedBy = user.user_code.to_string();
        }
        
        let str_sql = "update Project set StatusQuotation = '".to_string() + StatusQuotation::BOOKED + "'"
            + ", JobCreated = 1, DateJobCreatedBy = getdate(), JobCreatedBy = '" + &user.user_code + "'"
            + " where QuotationID = " + &self.fields.QuotationID.to_string() + ";"
            + " Exec MakeJobNo " + &self.fields.QuotationID.to_string() + ", 0, ''";
        conn.execute(&str_sql).unwrap();

        // let vec_project = pa::project.filter(pa::QuotationID.eq(self.fields.QuotationID)).load::<ProjectA>(conn).unwrap();
        // for project in vec_project.iter(){
        //     for item in self.list_project.iter_mut(){
        //         if item.fields.ProjectNo == project.ProjectNo{
        //             item.fields.JobNo = project.JobNo.clone();
        //         }
        //     }
        // }
        

        let quota = qa::quotation.filter(qa::QuotationID.eq(self.fields.QuotationID)).load::<QuotationA>(conn).unwrap();
        if quota.len() == 1{
            self.fields.SalesOrderNo = quota[0].SalesOrderNo.clone();
        }

        for project in self.list_project.iter()
        {
            let str_sql = "update SalesProduct2Purchase set SalesOrderNo=(select JobNo from Project where QuotationID =".to_string() + &self.fields.QuotationID.to_string() + " and ProjectNo = " + &project.fields.ProjectNo.to_string() + ")"
                    + " where QuotationID=" + &self.fields.QuotationID.to_string() + " and ProjectNo=" + &project.fields.ProjectNo.to_string();
            conn.execute(&str_sql).unwrap();
        }

        //锁定销售订单占用库存        
        if unsafe{(*self.current_version).fields.RequestPrepareDelivery}{
            // LockStock();
        }

        //更新QtyLockStock
        let mut str_sql = "".to_string();
        for ver_project in unsafe{(*self.current_version).list_quotation_ver_project.iter()}
        {
            for item in ver_project.list_quotation_item.iter()
            {
                str_sql += &("update QuotationItem set QtyLockStock = ".to_string() + &item.fields.QtyLockStock.to_string() + " where QuotationID="
                        + &item.fields.QuotationID.to_string() + " and VersionNo=" + &item.fields.VersionNo.to_string() + " and ProjectNo=" + &item.fields.ProjectNo.to_string()
                        + " and ItemNo=" + &item.fields.ItemNo.to_string() + " and Number = " + &item.fields.Number.to_string() + ";");
            }
        }
        if str_sql.len() != 0{
            conn.execute(&str_sql).unwrap();
        }
    }

    // 删除销售合同
    // 如果有销售机会则只把状态恢复到销售机会阶段
    pub fn delete_contract<'env>(&mut self, conn: &mut TyConn)
    {
        //已确认，不能删除
        if self.fields.IsConfirmedSalesOrder{
            return;
        }

        //有销售机会
        if self.fields2.NeedSurvey
        {
            //释放销售合同占用库存
            if unsafe{(*self.current_version).fields.RequestPrepareDelivery}{
                // ReleaseStock();
            }

            for project in self.list_project.iter_mut()
            {
                project.fields.StatusQuotation = StatusQuotation::PENDING.to_string();
                project.fields.JobCreated = "1".to_string();
                project.fields.DateJobCreatedBy = NaiveDateTime::default();
                project.fields.JobCreatedBy = "".to_string();
                project.fields.JobNo = "".to_string();
            }

            //锁定销售机会占用库存
            if unsafe{(*self.current_version).fields.RequestPrepareDelivery} 
            {
                // LockStock();
            }

            //如果销售合同数量进行了修改，则在删除时把数量恢复为批准备货的数量，否则会出现销售机会锁定库存数量错误
            for item in unsafe{(*self.current_version).all_items().iter_mut()}
            {
                if item.fields.Quantity != item.fields.QtyApprovePrepare && item.fields.QtyApprovePrepare.to_f64().unwrap() != 0f64
                {
                    item.fields.Quantity = item.fields.QtyApprovePrepare.clone();
                    item.fields.QtyLockStock = item.fields.QtyApprovePrepare.clone();
                }
            }
        }
        else
        {
            //设置采购合同关联的数据            
            for project in self.list_project.iter()
            {
                let str_sql = "update rfqverproject set RefNo = '' where JobNo = '".to_string() + &project.fields.PurchaserOrderNo + "'";
                conn.execute(&str_sql).unwrap();
            }
            // Delete();
        }
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

            let _ : Vec<_> = self.list_project.iter().map(|project|{
                //销售合同处于booking状态
                if project.fields.StatusQuotation == StatusQuotation::BOOKED
                {
                    b_have_available_project = true;                        
                }
                project.fields.JobNo.clone()                                
            }).collect();

            if !b_have_available_project{
                    return 5;
            }
            
            version.fields.DateRequestApprovePrepareGoods = chrono::Utc::now().naive_utc();
            version.fields.RequestApprovePrepareGoodsBy = user.user_code.clone();
            version.fields.RequestApprovePrepareGoods = true;   
            
            let mut not_receive_payment = false;
            let _:Vec<()> = version.list_quotation_ver_project.iter().map(|ver_project|
            {
                //预收款未到帐，需要审批
                if  ver_project.fields.AmountPrePayment > &ver_project.fields.AmountPaymentCancelled + BigDecimal::from(1i64){
                    not_receive_payment = true;
                }
            }).collect();
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
            let _:Vec<_> = version.list_quotation_ver_project.iter().map(|ver_project|
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
                    let _:Vec<_> = ver_project.list_quotation_item.iter().map(|item|
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
                    }).collect();                    
                }
                else
                {
                    if project.fields.RFQID != 0 && project.fields.RFQProjectNo != 0
                    {
                        error_code = 8;
                        //throw new ExceptionBusiness(ExceptionBusiness.cstrSalesDisableCancelPrepardGoodsAfterCreatePurchase, "已生成采购单，不能取消备货，如要取消请先删除采购单。");
                    }
                }
            }).collect();

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