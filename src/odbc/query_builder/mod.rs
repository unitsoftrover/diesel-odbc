use super::backend::Mysql;
use diesel::query_builder::QueryBuilder;
use diesel::result::QueryResult;

mod limit_offset;
mod query_fragment_impls;

/// The MySQL query builder
#[allow(missing_debug_implementations)]
#[derive(Default)]
pub struct MysqlQueryBuilder {
    sql: String,
}

impl MysqlQueryBuilder {
    /// Constructs a new query builder with an empty query
    pub fn new() -> Self {
        MysqlQueryBuilder::default()
    }
}

impl QueryBuilder<Mysql> for MysqlQueryBuilder {
    fn push_sql(&mut self, sql: &str) {
        self.sql.push_str(sql);
    }

    fn push_identifier(&mut self, identifier: &str) -> QueryResult<()> {
        self.push_sql("\"");
        self.push_sql(&identifier.replace("\"", "\"\""));
        self.push_sql("\"");
        Ok(())
    }

    fn push_bind_param(&mut self) {
        self.push_sql("?");
    }

    fn finish(&self) -> &String {
        &self.sql
    }

    fn clear(&mut self)->String{
        let old = self.sql.clone();
        self.sql.clear();   
        old
    }

}
