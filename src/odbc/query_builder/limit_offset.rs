use crate::odbc::Odbc;
use diesel::query_builder::limit_clause::{LimitClause, NoLimitClause};
use diesel::query_builder::limit_offset_clause::{BoxedLimitOffsetClause, LimitOffsetClause};
use diesel::query_builder::offset_clause::{NoOffsetClause, OffsetClause};
use diesel::query_builder::{AstPass, IntoBoxedClause, QueryFragment};
use diesel::result::QueryResult;

impl QueryFragment<Odbc> for LimitOffsetClause<NoLimitClause, NoOffsetClause> {
    fn walk_ast(&self, _out: AstPass<Odbc>) -> QueryResult<()> {
        Ok(())
    }
}

impl<L> QueryFragment<Odbc> for LimitOffsetClause<LimitClause<L>, NoOffsetClause>
where
    LimitClause<L>: QueryFragment<Odbc>,
{
    fn walk_ast(&self, out: AstPass<Odbc>) -> QueryResult<()> {
        self.limit_clause.walk_ast(out)?;
        Ok(())
    }
}

impl<L, O> QueryFragment<Odbc> for LimitOffsetClause<LimitClause<L>, OffsetClause<O>>
where
    LimitClause<L>: QueryFragment<Odbc>,
    OffsetClause<O>: QueryFragment<Odbc>,
{
    fn walk_ast(&self, mut out: AstPass<Odbc>) -> QueryResult<()> {
        self.limit_clause.walk_ast(out.reborrow())?;
        self.offset_clause.walk_ast(out.reborrow())?;
        Ok(())
    }
}

impl<'a> QueryFragment<Odbc> for BoxedLimitOffsetClause<'a, Odbc> {
    fn walk_ast(&self, mut out: AstPass<Odbc>) -> QueryResult<()> {
        match (self.limit.as_ref(), self.offset.as_ref()) {
            (Some(limit), Some(offset)) => {
                limit.walk_ast(out.reborrow())?;
                offset.walk_ast(out.reborrow())?;
            }
            (Some(limit), None) => {
                limit.walk_ast(out.reborrow())?;
            }
            (None, Some(offset)) => {
                // Odbc requires a limit clause in front of any offset clause
                // The documentation proposes the following:
                // > To retrieve all rows from a certain offset up to the end of the
                // > result set, you can use some large number for the second parameter.
                // https://dev.Odbc.com/doc/refman/8.0/en/select.html
                // Therefore we just use u64::MAX as limit here
                // That does not result in any limitations because Odbc only supports
                // up to 64TB of data per table. Assuming 1 bit per row this means
                // 1024 * 1024 * 1024 * 1024 * 8 = 562.949.953.421.312 rows which is smaller
                // than 2^64 = 18.446.744.073.709.551.615
                out.push_sql(" LIMIT 18446744073709551615 ");
                offset.walk_ast(out.reborrow())?;
            }
            (None, None) => {}
        }
        Ok(())
    }
}

impl<'a> IntoBoxedClause<'a, Odbc> for LimitOffsetClause<NoLimitClause, NoOffsetClause> {
    type BoxedClause = BoxedLimitOffsetClause<'a, Odbc>;

    fn into_boxed(self) -> Self::BoxedClause {
        BoxedLimitOffsetClause {
            limit: None,
            offset: None,
        }
    }
}

impl<'a, L> IntoBoxedClause<'a, Odbc> for LimitOffsetClause<LimitClause<L>, NoOffsetClause>
where
    L: QueryFragment<Odbc> + Send + 'a,
{
    type BoxedClause = BoxedLimitOffsetClause<'a, Odbc>;

    fn into_boxed(self) -> Self::BoxedClause {
        BoxedLimitOffsetClause {
            limit: Some(Box::new(self.limit_clause)),
            offset: None,
        }
    }
}

impl<'a, L, O> IntoBoxedClause<'a, Odbc> for LimitOffsetClause<LimitClause<L>, OffsetClause<O>>
where
    L: QueryFragment<Odbc> + Send + 'a,
    O: QueryFragment<Odbc> + Send + 'a,
{
    type BoxedClause = BoxedLimitOffsetClause<'a, Odbc>;

    fn into_boxed(self) -> Self::BoxedClause {
        BoxedLimitOffsetClause {
            limit: Some(Box::new(self.limit_clause)),
            offset: Some(Box::new(self.offset_clause)),
        }
    }
}
