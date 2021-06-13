use std::marker::PhantomData;
use super::StatementUse;
use diesel::deserialize::FromSqlRow;
use diesel::result::Error::DeserializationError;
use diesel::result::QueryResult;
use crate::odbc::Mysql;
use super::bind::*;
use super::metadata::*;
use diesel::row::*;
use super::ColumnDescriptor;

pub struct StatementIterator<'b, ST, T, S> {
    stmt: StatementUse<'b, S>,
    _marker: PhantomData<(ST, T)>,   
}

impl<'b, ST, T, S> StatementIterator<'b, ST, T, S> {
    pub fn new(stmt: StatementUse<'b, S>) -> Self {

        StatementIterator {
            stmt: stmt,
            _marker: PhantomData            
        }
    }
}

impl<'b, ST, T, S> Iterator for StatementIterator<'b, ST, T, S>
where
    T: FromSqlRow<ST,  Mysql>,
{
    type Item = QueryResult<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let row = match self.stmt.step() {
            Ok(row) => row,
            Err(e) => return Some(Err(e)),
        };
        row.map(|row| T::build_from_row(&row).map_err(DeserializationError))
    }
}

#[derive(Clone)]
pub struct MysqlRow<'a> {
    pub col_idx: usize,
    pub binds: &'a Binds,
    pub metadata: &'a StatementMetadata,
}

impl<'a> Row<'a, Mysql> for MysqlRow<'a> {
    type Field = MysqlField<'a>;
    type InnerPartialRow = Self;

    fn field_count(&self) -> usize {
        self.binds.len()
    }

    fn get<I>(&self, idx: I) -> Option<Self::Field>
    where
        Self: RowIndex<I>,
    {
        let idx = self.idx(idx)?;
        Some(MysqlField {
            bind: &self.binds[idx],
            metadata: &self.metadata.fields()[idx],
        })
    }

    fn partial_row(&self, range: std::ops::Range<usize>) -> PartialRow<Self::InnerPartialRow> {
        PartialRow::new(self, range)
    }
}

impl<'a> RowIndex<usize> for MysqlRow<'a> {
    fn idx(&self, idx: usize) -> Option<usize> {
        if idx < self.field_count() {
            Some(idx)
        } else {
            None
        }
    }
}

impl<'a, 'b> RowIndex<&'a str> for MysqlRow<'b> {
    fn idx(&self, idx: &'a str) -> Option<usize> {
        self.metadata
            .fields()
            .iter()
            .enumerate()
            .find(|(_, field_meta)| field_meta.name == idx)
            .map(|(idx, _)| idx)
    }
}

pub struct MysqlField<'a> {
    bind: &'a BindData,
    metadata: &'a ColumnDescriptor,
}

impl<'a> Field<'a, Mysql> for MysqlField<'a> {
    fn field_name(&self) -> Option<&'a str> {
        Some(self.metadata.name.as_str())
    }

    fn is_null(&self) -> bool {
        self.bind.is_null()
    }

    fn value(&self) -> Option<diesel::backend::RawValue<'a, Mysql>> {
        self.bind.value()
    }
}