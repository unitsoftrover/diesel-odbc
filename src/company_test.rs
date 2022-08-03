use diesel::prelude::*;
use diesel::debug_query;
use diesel::dsl::*;
use chrono::{NaiveDateTime};
use bigdecimal::*;

// use diesel_odbc::connection::statement::{Statement, ResultSetState};
use diesel_odbc::connection::RawConnection;
use diesel_odbc::Odbc;

use super::models::*;
use super::safe::*;

pub fn test<'env>(conn : &mut RawConnection<'env, AutocommitOn>)
{
    let query = insert_into(company)
    .values((CompanyCode.eq("00000000"), CompanyName.eq("unitsoft"), DateCreated.eq(NaiveDateTime::parse_from_str("2020-1-1 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()), CompanyType.eq("C"), CreateOffice.eq("SH"), CreditAmount.eq(BigDecimal::from_f64(100000.0))));

    let debug = debug_query::<Odbc, _>(&query);
    println!("query:{}", debug.to_string());   

    let _one_company = query
        .load::<Company>(conn).unwrap();
    let mut company_id = 0;
    let _ = _one_company.iter().map(|com|{
        println!("CompanyID:{}", com.CompanyID);
        println!("inserted id:{:?}", com.CompanyID);
        let sql = format!("exec MakeCode 'CompanyCode', {}", com.CompanyID);
        let _ = conn.execute(sql.as_str());
        company_id = com.CompanyID;        
    }).collect::<()>();   

    

    // {
    //     let stmt = Statement::with_parent(conn).unwrap();
    //     let stmt = stmt.prepare("select count(*) from company").unwrap();        
    //     let stmt = stmt.execute().unwrap();    

    //     match stmt{
    //         ResultSetState::Data(mut st)=>{
    //             // let row_count = st.affected_row_count().unwrap();
    //             // println!("row count: {}", row_count);

    //             if let Some(mut cursor) = st.fetch().unwrap(){
    //                 if let Some(count) = cursor.get_data(1).unwrap() as Option<i64>{
    //                     println!("count:{}", count);
    //                 }

    //                 // let row_count = st.affected_row_count().unwrap();
    //                 // println!("row count2: {}", row_count);

    //             }
    //         },
    //         ResultSetState::NoData(_st)=>{},
    //     }
    // }


    // {     
    //     let stmt = Statement::with_parent(conn).unwrap();
    //     let stmt = stmt.prepare("select top 1 CompanyID,CompanyCode,CompanyName from company").unwrap();                   
    //     let stmt = stmt.execute().unwrap();       
    //     match stmt{
    //         ResultSetState::Data(mut st)=>{
    //             while let Some(mut cursor) = st.fetch().unwrap(){                    

    //                 if let Some(val) = cursor.get_data(1).unwrap() as Option<i64>{
    //                     println!("CompanyID:{}", val);
    //                 }
    //                 if let Some(val) = cursor.get_data(2).unwrap() as Option<String>{                    
    //                     println!("Company Code:{}", val);
    //                 }
    //                 if let Some(val) = cursor.get_data(3).unwrap() as Option<String>{
    //                     println!("Company Name:{}", val);
    //                 }                                     
    //             }
    //         },
    //         ResultSetState::NoData(_st)=>{},
    //     }    
    // }

    use super::schema::company::dsl::*;
    // let mut query_builder = odbc::OdbcQueryBuilder::new();    
    // let ast_pass = AstPass::<odbc::Odbc>::to_sql(&mut query_builder);    
    // let primary_key = company.primary_key();    
    // primary_key.walk_ast(ast_pass).unwrap();
    
    let results = company
        .filter(CompanyCode.eq("C0000005"))              
        .load::<Company>(conn)
        .expect("Error loading company");
    
    println!("Displaying {} company", results.len());
    for company1 in results {
        println!("CompanyID:{}", company1.CompanyID);
        println!("CompanyCode:{}", company1.CompanyCode);
        println!("CompanyName:{}", company1.CompanyName);
        println!("Create Date:{:?}", company1.DateCreated);
        println!("Credit Amount:{}", company1.CreditAmount.unwrap_or_default());
        // println!("Is Headquater:{}", company1.IsHeadOffice);
        // println!("Test Date:{}", company1.TestDate);
    }    






    // let company1 = delete(company.filter(CompanyID.eq(company_id))).get_result::<Company>(conn);
    // company1.map(|com|{
    //     println!("company code:{} company name:{}", com.CompanyCode, com.CompanyName)
    // }).unwrap();

    // replace_into(company)
    //  .values((CompanyCode.eq("00000000"), CompanyName.eq("unitsoft"), DateCreated.eq(NaiveDateTime::parse_from_str("2020-1-1 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap()), CompanyType.eq("C"), CreateOffice.eq("SH")))
    //  .execute(conn)
    //  .unwrap();

    // let stmt = conn.prepare_query1(&"select ident_current('company')".to_owned());
    // if let Ok(rs) = stmt.execute(){
    //     match rs{
    //         ResultSetState::Data(mut st)=>{
    //             let mut cur = st.fetch().unwrap().unwrap();
    //             if let Some(company_id) = cur.get_data::<i64>(1).unwrap(){
    //                 println!("inserted id:{:?}", company_id);
    //                 let sql = format!("exec MakeCode 'CompanyCode', {}", company_id);
    //                 let _ = conn.execute(sql.as_str());
    //             }
    //             else
    //             {
    //                 println!("no inserted id.")
    //             }
    //         },
    //         ResultSetState::NoData(_st)=>{

    //         }
    //     }        
    // }

    let query = company.select(CompanyCode).filter(CompanyID.eq(1));
    let query_str = debug_query::<Odbc, _>(&query).to_string();
    println!("query：{:?}", query_str);
    let company_code = query.load::<String>(conn).unwrap();
    println!("company code:{}", company_code[0]);        

    diesel::update(company.filter(CompanyID.eq(1)))
    .set(CompanyName.eq("unitsoft_test"))
    .execute(conn)
    .unwrap();            

    let query = company.select(CompanyName).filter(CompanyID.eq(1));
    let company_name = query.load::<String>(conn).unwrap();
    println!("new company name:{}", company_name[0]); 

    let company1:Company = diesel::update(company.filter(CompanyID.eq(1)))
    .set(CompanyName.eq("unitsoft_new"))
    .get_result(conn)
    .unwrap();            

    println!("new company name:{}", company1.CompanyName); 

    // let sum1 : Option<BigDecimal> = company.select(sum(CreditAmount)).get_result(conn).unwrap();
    // println!("sum:{:?}", sum1);

    // let avg : Option<BigDecimal> = company.select(avg(CreditAmount)).get_result(conn).unwrap();
    // println!("avg:{:?}", avg);

    // let count : i64= company.select(count(CompanyID)).get_result(conn).unwrap();
    // println!("count:{}", count);

    // let count : Option<BigDecimal> = company.select(sum(CreditAmount)).get_result(conn).unwrap();
    // println!("sum: {}", count.unwrap());

    // let company_list = sql_query("SELECT CompanyID,CompanyCode,CompanyType,CreateOffice,CompanyName,CompanyNameCN,DateCreated,CreditAmount,IsHeadOffice,TestSmallInt,TestTinyInt,TestDate,TestTime FROM company ORDER BY CompanyCode")
    // .load::<Company>(conn).unwrap();
    // println!("company list:{:?}", company_list);

    // let stmt = conn.prepare_query1(&"SELECT CompanyName FROM company where CompanyID=1 ORDER BY CompanyCode".to_owned());
    // if let Ok(rs) = stmt.execute(){
    //     match rs {
    //         ResultSetState::Data(mut st)=>{
    //             let mut cur = st.fetch().unwrap().unwrap();
    //             let company_name:String = cur.get_data(1).unwrap().unwrap();
    //             println!("company name:{:?}", company_name);
    //         },
    //         ResultSetState::NoData(_st)=>{

    //         }
    //     }        
    // }


}