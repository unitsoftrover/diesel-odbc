use diesel::prelude::*;
use diesel::debug_query;
use diesel::dsl::*;

use diesel_odbc::connection::RawConnection;
use diesel_odbc::Odbc;

use super::models::*;
use super::sales;
use super::safe::*;


pub fn test<'env>(conn : &RawConnection<'env, AutocommitOn>)
{
    let office = sales::Office{
        office_code : "SH".to_string(),
        office_name : "unitsoft".to_string(),
    };
    let user = sales::User{
        user_code : "admin".to_string(),
        user_name : "rover".to_string(),
        office : &office,
        current_office : &office,
    };
    {
        use super::schema::quotation_a::dsl::*;
        use super::schema::quotation_x::dsl as quotationx;
        let mut quotation1 = sales::Quotation::new_sales_order(user);
        // println!("bl_quotation:{:?}", quotation1);    
        quotation1.fields_a.QuotationNo = "".to_string();
        quotation1.fields_a.LeadSource = "sales".to_string();
        quotation1.fields_a.QuotationBy = "admin".to_string();        
        quotation1.fields_a.QuotationContactID = Some(1);
        quotation1.fields_b.CompanyName = "友耐软件".to_string();

        quotation1.save(conn);
        
        let mut quotation_x : QuotationX = Default::default();
        quotation_x.QuotationNo = "SH21-Q1000001".to_string();
        // quotation_x.LeadSource = "".to_string();

        // let query = insert_into(quotation_a).values(quotation1.fields_a);        
        // let query = insert_into(quotation_a).values(QuotationNo.eq("SH21-Q1000001"));
        // let query = insert_into(quotationx::quotation_x).values(quotationx::QuotationContactID.eq(1));
        let query = insert_into(quotationx::quotation_x).values(quotation_x);

        // let debug = debug_query::<Odbc, _>(&query);
        // println!("query:{}", debug.to_string());
        // query.execute(conn).unwrap(); 
        
        // let query = delete(quotation_a.filter(QuotationNo.eq("SH21-Q1000001")));
        // let debug = debug_query::<Odbc, _>(&query);
        // println!("query:{}", debug.to_string());
        
        // let result = query.get_result::<QuotationA>(&conn).unwrap();
        // let result = query.load::<QuotationA>(conn).unwrap();
        // println!("result:{:?}", result[0]);

        // let mut qa = query.load::<QuotationA>(conn).unwrap();
        // let mut qa = query.execute(conn).unwrap();
        // let mut qa = quotationx::quotation_x.filter(quotationx::QuotationNo.eq("SH21-Q0000014")).load::<QuotationX>(conn);   

        let mut qa = quotationx::quotation_x.load::<QuotationX>(conn);   
        let mut qa = qa.unwrap();

        // let mut qa = query.load::<QuotationX>(conn).unwrap();   
        if qa.len() > 0
        {
            let qa = qa.get_mut(0).unwrap();
            qa.Operator = Some("rover".to_string());
            qa.QuotationNo = "SH21-Q10000001".to_string();
            qa.LeadSource = qa.LeadSource.trim().to_string();
            qa.QuotationContactID = Some(1);
            qa.BillingContactID = Some(1);
            qa.QuotationBy = "admin".to_string();

            qa.AddressBill = "shanghai".to_string();
            // let qa = update(quotation_a).set(&*qa).load::<QuotationA>(conn).unwrap();
            // let qa = update(quotationx::quotation_x).set(&*qa).execute(conn).unwrap();
            // let mut qa = quotationx::quotation_x.filter(quotationx::QuotationNo.eq("SH21-Q1000001")).load::<QuotationX>(conn).unwrap();   

            let qa = update(quotationx::quotation_x.filter(quotationx::QuotationNo.eq("SH21-Q0000001"))).set(&*qa).load::<QuotationX>(conn).unwrap();
            let qa = qa.get(0).unwrap();
            // println!("qa no:{}", qa.QuotationNo);

            let query = quotation_a.filter(QuotationNo.eq("SH21-Q0000001"));
            let debug = debug_query::<Odbc, _>(&query);
            let result = query.load::<QuotationA>(conn).unwrap(); 
            println!("delete quotation No:{:?}", result.get(0).unwrap().QuotationNo);
        }
    }

    {
        use super::schema::quotation_a::dsl::*;
        let quotations = quotation_a.filter(QuotationNo.eq("SH20-Q0000001"))
            .load::<QuotationA>(conn)
            .expect("Error loading quotation");   

        for quotation1 in quotations {
            // println!("CompanyID:{}", quotation1.CompanyID);
            // println!("CompanyCode:{}", quotation1.CompanyCode);
            // println!("CompanyName:{}", quotation1.CompanyName);
            // println!("Create Date:{:?}", quotation1.CreateDate);
            println!("Quotation No.:{}", quotation1.QuotationNo);
            // println!("Credit Amount:{}", quotation1.CreditAmount.unwrap_or_default());
            // println!("Is Headquater:{}", company1.IsHeadOffice);
            // println!("Test Date:{}", company1.TestDate);

            // let quotation2 = Quotation2A::belonging_to(&quotation1)
            //     .get_result::<Quotation2A>(&conn).unwrap();
            // println!("Quotation ID.:{}", quotation2.QuotationID);
        }
    }   

    
    // let office = sales::Office{
    //     office_code : "SH".to_string(),
    //     office_name : "unitsoft".to_string(),
    // };
    // let user = sales::User{
    //     user_code : "admin".to_string(),
    //     user_name : "rover".to_string(),
    //     office : &office,
    //     current_office : &office,
    // };

    // use self::schema::quotation_a::dsl::*;
    // let mut quotation = sales::Quotation::new(user);
    // println!("bl_quotation:{:?}", quotation);    
    // quotation.fields_a.LeadSource = "sales".to_string();
    // quotation.fields_a.QuotationBy = "admin".to_string();
    // let query = insert_into(quotation_a).values((QuotationNo.eq("SH21-Q1000002"), LeadSource.eq(quotation.fields_a.LeadSource), QuotationBy.eq(quotation.fields_a.QuotationBy)));
    // let debug = debug_query::<Odbc, _>(&query);
    // println!("query:{}", debug.to_string());

    // // query.execute(&conn).unwrap(); 
    // let qa = query.load::<QuotationA>(&conn).unwrap();    
    // if qa.len() > 0
    // {
    //     let qa = qa.get(0).unwrap();
    //     println!("qa :{:?}", qa);
    //     let query = delete(quotation_a.filter(QuotationID.eq(qa.QuotationID)));
    //     let debug = debug_query::<Odbc, _>(&query);
    //     println!("query:{}", debug.to_string());
    //     query.get_result::<QuotationA>(&conn).unwrap();
    // }

    // let qa = query.get_result::<QuotationA>(&conn).unwrap();    
    // delete(quotation_a.filter(QuotationID.eq(qa.QuotationID))).execute(&conn);

}