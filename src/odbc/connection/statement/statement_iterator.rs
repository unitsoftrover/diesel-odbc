use std::marker::PhantomData;
use super::StatementUse;
use diesel::deserialize::FromSqlRow;
use diesel::result::Error::DeserializationError;
use diesel::result::QueryResult;
use crate::odbc::Odbc;
use super::bind::*;
use super::metadata::*;
use diesel::row::*;
use super::ColumnDescriptor;

pub struct StatementIterator<'b, ST, T> {
    stmt: StatementUse<'b>,
    _marker: PhantomData<(ST, T)>,   
}

impl<'b, ST, T> StatementIterator<'b, ST, T> {
    pub fn new(stmt: StatementUse<'b>) -> Self {
        let st = StatementIterator {
            stmt: stmt,
            _marker: PhantomData            
        };
        st
    }
}

impl<'b, ST, T> Iterator for StatementIterator<'b, ST, T>
where
    T: FromSqlRow<ST,  Odbc>,
{
    type Item = QueryResult<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let row = match self.stmt.step() {
            Ok(row) => {
                if let Some(row) = row{
                    Some(row)
                }
                else{
                    return None;
                }

            },
            Err(e) => return Some(Err(e)),
        };
        row.map(|row| T::build_from_row(&row).map_err(DeserializationError))
    }
}

#[derive(Clone)]
pub struct OdbcRow<'a> {
    pub col_idx: usize,
    pub binds: &'a Binds,
    pub metadata: &'a StatementMetadata,
}

impl<'a> Row<'a, Odbc> for OdbcRow<'a> {
    type Field = OdbcField<'a>;
    type InnerPartialRow = Self;

    fn field_count(&self) -> usize {
        self.binds.len()
    }

    fn get<I>(&self, idx: I) -> Option<Self::Field>
    where
        Self: RowIndex<I>,
    {
        let idx = self.idx(idx)?;
        Some(OdbcField {
            bind: &self.binds[idx],
            metadata: &self.metadata.fields()[idx],
        })
    }

    fn partial_row(&self, range: std::ops::Range<usize>) -> PartialRow<Self::InnerPartialRow> {
        PartialRow::new(self, range)
    }
}

impl<'a> RowIndex<usize> for OdbcRow<'a> {
    fn idx(&self, idx: usize) -> Option<usize> {
        if idx < self.field_count() {
            Some(idx)
        } else {
            None
        }
    }
}

impl<'a, 'b> RowIndex<&'a str> for OdbcRow<'b> {
    fn idx(&self, idx: &'a str) -> Option<usize> {
        self.metadata
            .fields()
            .iter()
            .enumerate()
            .find(|(_, field_meta)| field_meta.name == idx)
            .map(|(idx, _)| idx)
    }
}

pub struct OdbcField<'a> {
    bind: &'a BindData,
    metadata: &'a ColumnDescriptor,
}

impl<'a> Field<'a, Odbc> for OdbcField<'a> {
    fn field_name(&self) -> Option<&'a str> {
        Some(self.metadata.name.as_str())
    }

    fn is_null(&self) -> bool {
        self.bind.is_null()
    }

    fn value(&self) -> Option<diesel::backend::RawValue<'a, Odbc>> {

        let val = self.bind.value();        
        val
    }

    fn raw_value(&self)->Option<&'a [u8]>{
        self.bind.raw_value()
    }
}