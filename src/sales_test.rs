use diesel::prelude::*;
use diesel::debug_query;
use diesel::dsl::*;

use diesel_odbc::connection::RawConnection;
use diesel_odbc::Odbc;

use super::models::*;
use super::sales;
use super::safe::*;


pub fn test<'env>(conn : &RawConnection<'env, AutocommitOn>)//RawConnection<'env, AutocommitOn>
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
        let mut quotation1 = sales::Quotation::new_sales_order(user);
        println!("bl_quotation:{:?}", quotation1);    
        quotation1.fields_a.LeadSource = "sales".to_string();
        quotation1.fields_a.QuotationBy = "admin".to_string();
        // quotation1.fields_a.QuotationContactID = 100;
        let query = insert_into(quotation_a).values((QuotationNo.eq("SH21-Q1000001"), LeadSource.eq(quotation1.fields_a.LeadSource), QuotationBy.eq(quotation1.fields_a.QuotationBy), QuotationTo.eq("1111")));
        // let debug = debug_query::<Odbc, _>(&query);
        // println!("query:{}", debug.to_string());
        // query.execute(conn).unwrap(); 
        
        // let query = delete(quotation_a.filter(QuotationNo.eq("SH21-Q1000001")));
        // let debug = debug_query::<Odbc, _>(&query);
        // println!("query:{}", debug.to_string());
        
        // let result = query.get_result::<QuotationA>(&conn).unwrap();
        // let result = query.load::<QuotationA>(conn).unwrap();
        // println!("result:{:?}", result[0]);

        let qa = query.load::<QuotationA>(conn).unwrap();   
        if qa.len() > 0
        {
            let qa = qa.get(0).unwrap();
            println!("qa :{:?}", qa);
            
            let quotation = sales::Quotation::load(conn, qa.QuotationID);
            println!("quotation:{:?}", quotation);            


            let query = delete(quotation_a.filter(QuotationID.eq(qa.QuotationID)));
            let debug = debug_query::<Odbc, _>(&query);
            println!("query:{}", debug.to_string());
            
            // let result = query.get_result::<QuotationA>(&conn).unwrap();
            let result = query.load::<QuotationA>(conn).unwrap();
            // query.execute(&conn).unwrap();
            println!("quotation:{:?}", result);
            println!("Ok");
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