use crate::odbc::Odbc;
use diesel::query_builder::locking_clause::{ForShare, ForUpdate, NoModifier, NoWait, SkipLocked};
use diesel::query_builder::{AstPass, QueryFragment};
use diesel::result::QueryResult;

impl QueryFragment<Odbc> for ForUpdate {
    fn walk_ast(&self, mut out: AstPass<Odbc>) -> QueryResult<()> {
        out.push_sql(" FOR UPDATE");
        Ok(())
    }
}

impl QueryFragment<Odbc> for ForShare {
    fn walk_ast(&self, mut out: AstPass<Odbc>) -> QueryResult<()> {
        out.push_sql(" FOR SHARE");
        Ok(())
    }
}

impl QueryFragment<Odbc> for NoModifier {
    fn walk_ast(&self, _out: AstPass<Odbc>) -> QueryResult<()> {
        Ok(())
    }
}

impl QueryFragment<Odbc> for SkipLocked {
    fn walk_ast(&self, mut out: AstPass<Odbc>) -> QueryResult<()> {
        out.push_sql(" SKIP LOCKED");
        Ok(())
    }
}

impl QueryFragment<Odbc> for NoWait {
    fn walk_ast(&self, mut out: AstPass<Odbc>) -> QueryResult<()> {
        out.push_sql(" NOWAIT");
        Ok(())
    }
}
