use std::io::Write;

use crate::db::{Datum, Schema, serialize_datum};

use super::Error;

pub type Tuple = Vec<Option<Datum>>;

pub fn serialize_tuple(buf: &mut impl Write, schema: &Schema, tuple: &Tuple) -> Result<(), Error> {
    if tuple.is_empty() {
        return Err(Error::SerializeEmptyTuple);
    }
    if schema.len() != tuple.len() {
        return Err(Error::TupleSchemaLenMismatch {
            tuple_len: tuple.len(),
            schema_len: schema.len(),
        });
    }
    for (col, datum) in schema.iter().zip(tuple.iter()) {
        serialize_datum(buf, col, datum)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::db::{ColumnDefinition, DataType};

    use super::*;

    #[test]
    fn test_serialize_empty_tuple() {
        let mut buf = Cursor::new(vec![]);
        let res = serialize_tuple(&mut buf, &vec![], &vec![]);
        assert_eq!(res, Err(Error::SerializeEmptyTuple));
    }

    #[test]
    fn test_serialize_mismatched_schema_len() {
        let mut buf = Cursor::new(vec![]);
        let res = serialize_tuple(&mut buf, &vec![], &vec![None]);
        assert_eq!(
            res,
            Err(Error::TupleSchemaLenMismatch {
                tuple_len: 1,
                schema_len: 0
            })
        );

        let mut buf = Cursor::new(vec![]);
        let schema: Schema = vec![
            ColumnDefinition {
                name: "id".to_string(),
                nullable: false,
                data_type: DataType::Integer,
            },
            ColumnDefinition {
                name: "name".to_string(),
                nullable: true,
                data_type: DataType::Text,
            },
        ];
        let res = serialize_tuple(&mut buf, &schema, &vec![Some(Datum::Integer(123))]);
        assert_eq!(
            res,
            Err(Error::TupleSchemaLenMismatch {
                tuple_len: 1,
                schema_len: 2
            })
        );
    }
}
