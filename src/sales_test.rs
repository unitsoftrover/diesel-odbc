use diesel::prelude::*;
use diesel::dsl::*;
use super::RawConnection;

use diesel_odbc::models::*;
use super::sales;
use super::safe::*;

pub fn test<'env>(conn : &mut RawConnection<'env, AutocommitOn>)
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
        use diesel_odbc::schema::quotation::dsl::*;
        use diesel_odbc::schema::quotation_x::dsl as quotationx;
        let mut quotation1 = sales::Quotation::new_sales_order(user);
        quotation1.fields.QuotationNo = "".to_string();
        quotation1.fields.LeadSource = "sales".to_string();
        quotation1.fields.QuotationBy = "admin".to_string();        
        quotation1.fields.QuotationContactID = Some(1);
        quotation1.fields.CompanyCode = "C0000001".to_string();
        quotation1.fields.CompanyName = "友耐软件".to_string();
        quotation1.save(conn);
        
        let mut quotation_x : QuotationX = Default::default();
        quotation_x.QuotationNo = "".to_string();
        quotation_x.Operator = Some("rover".to_string());
        quotation_x.CompanyID = Some(1);
        quotation_x.QuotationContactID = Some(1);
        quotation_x.BillingContactID = Some(1);
        quotation_x.QuotationBy = "admin".to_string();        
        quotation_x.OfficeCode = "SH".to_string();

        let qa = insert_into(quotationx::quotation_x).values(quotation_x).load::<QuotationX>(conn);   
        let mut qa = qa.unwrap();
        if qa.len() > 0
        {
            let qa = qa.get_mut(0).unwrap();

            println!("quotationID:{} QuotationNo:{}", qa.QuotationID, qa.QuotationNo);

            qa.Operator = Some("rover".to_string());
            qa.QuotationNo = "SH21-Q10000001".to_string();
            // qa.LeadSource = qa.LeadSource.trim().to_string();
            // qa.QuotationContactID = Some(1);
            // qa.BillingContactID = Some(1);
            // qa.QuotationBy = "admin".to_string();
            // qa.AddressBill = "shanghai".to_string();           


            let qa = update(quotationx::quotation_x.filter(quotationx::QuotationNo.eq("SH21-Q0000001"))).set(&*qa).load::<QuotationX>(conn).unwrap();
            let qa = qa.get(0).unwrap();
            println!("qa no:{}", qa.QuotationNo);

            let query = quotation.filter(QuotationNo.eq("SH21-Q0000001"));
            let result = query.load::<QuotationA>(conn).unwrap(); 
            if result.len()>0{
                println!("delete quotation No:{:?}", result.get(0).unwrap().QuotationNo);
            }
        }
    }

    {
        use diesel_odbc::schema::quotation::dsl::*;
        let quotations = quotation.filter(QuotationNo.eq("SH20-Q0000001"))
            .load::<QuotationA>(conn)
            .expect("Error loading quotation");   

        for quotation1 in quotations {
            println!("Quotation No.:{}", quotation1.QuotationNo);

            let quotation2 = Quotation2::belonging_to(&quotation1)
                .get_result::<Quotation2>(conn).unwrap();
            println!("Quotation ID.:{}", quotation2.QuotationID);
        }
    }   
}

