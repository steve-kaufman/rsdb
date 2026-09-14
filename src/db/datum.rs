use std::io::Write;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::{
    ColumnDefinition, DataType, Error, serialize_float, serialize_int, serialize_null,
    serialize_text, serialize_uuid,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Datum {
    Integer(i64),
    Float(f64),
    Text(String),
    Uuid(Uuid),
}

impl Datum {
    pub fn data_type(&self) -> DataType {
        match self {
            Self::Integer(_) => DataType::Integer,
            Self::Float(_) => DataType::Float,
            Self::Text(_) => DataType::Text,
            Self::Uuid(_) => DataType::Uuid,
        }
    }
}

pub fn serialize_datum(
    buf: &mut impl Write,
    col: &ColumnDefinition,
    datum: &Option<Datum>,
) -> Result<(), Error> {
    match datum {
        Some(datum) => serialize_some_datum(buf, col, datum),
        None => serialize_null_datum(buf, col),
    }
}

pub fn serialize_some_datum(
    buf: &mut impl Write,
    col: &ColumnDefinition,
    datum: &Datum,
) -> Result<(), Error> {
    if datum.data_type() != col.data_type {
        return Err(Error::tuple_schema_type_mismatch(col, datum));
    }
    if col.nullable {
        serialize_null(buf, false)?;
    }
    match datum {
        Datum::Integer(n) => serialize_int(buf, *n),
        Datum::Float(n) => serialize_float(buf, *n),
        Datum::Text(text) => serialize_text(buf, text),
        Datum::Uuid(uuid) => serialize_uuid(buf, *uuid),
    }
}

pub fn serialize_null_datum(buf: &mut impl Write, col: &ColumnDefinition) -> Result<(), Error> {
    if col.nullable {
        serialize_null(buf, true)?;
        Ok(())
    } else {
        Err(Error::NullMismatch {
            column_definition: col.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_mismatched_schema_type_integer() {
        for datum in [
            Datum::Uuid(Uuid::new_v4()),
            Datum::Float(123.456),
            Datum::Text("hello".to_string()),
        ] {
            let mut buf = vec![];
            let col = ColumnDefinition {
                name: "id".to_string(),
                nullable: false,
                data_type: DataType::Integer,
            };
            let res = serialize_datum(&mut buf, &col, &Some(datum.clone()));
            assert_eq!(
                res,
                Err(Error::TupleSchemaTypeMismatch {
                    column_definition: col.clone(),
                    datum,
                })
            );
        }
    }

    #[test]
    fn test_serialize_mismatched_schema_type_float() {
        for datum in [
            Datum::Uuid(Uuid::new_v4()),
            Datum::Integer(123),
            Datum::Text("hello".to_string()),
        ] {
            let mut buf = vec![];
            let col = ColumnDefinition {
                name: "id".to_string(),
                nullable: false,
                data_type: DataType::Float,
            };
            let res = serialize_datum(&mut buf, &col, &Some(datum.clone()));
            assert_eq!(
                res,
                Err(Error::TupleSchemaTypeMismatch {
                    column_definition: col.clone(),
                    datum,
                })
            );
        }
    }

    #[test]
    fn test_serialize_mismatched_schema_type_text() {
        for datum in [
            Datum::Uuid(Uuid::new_v4()),
            Datum::Integer(123),
            Datum::Float(123.456),
        ] {
            let mut buf = vec![];
            let col = ColumnDefinition {
                name: "id".to_string(),
                nullable: false,
                data_type: DataType::Text,
            };
            let res = serialize_datum(&mut buf, &col, &Some(datum.clone()));
            assert_eq!(
                res,
                Err(Error::TupleSchemaTypeMismatch {
                    column_definition: col.clone(),
                    datum,
                })
            );
        }
    }

    #[test]
    fn test_serialize_mismatched_schema_type_uuid() {
        for datum in [
            Datum::Integer(123),
            Datum::Float(123.456),
            Datum::Text("hello".to_string()),
        ] {
            let mut buf = vec![];
            let col: ColumnDefinition = ColumnDefinition {
                name: "id".to_string(),
                nullable: false,
                data_type: DataType::Uuid,
            };
            let res = serialize_datum(&mut buf, &col, &Some(datum.clone()));
            assert_eq!(
                res,
                Err(Error::TupleSchemaTypeMismatch {
                    column_definition: col.clone(),
                    datum,
                })
            );
        }
    }

    #[test]
    fn test_null_mismatch() {
        for data_type in [
            DataType::Float,
            DataType::Integer,
            DataType::Uuid,
            DataType::Text,
        ] {
            let mut buf = vec![];
            let col = ColumnDefinition {
                name: "id".to_string(),
                nullable: false,
                data_type,
            };
            let res = serialize_datum(&mut buf, &col, &None);
            assert_eq!(
                res,
                Err(Error::NullMismatch {
                    column_definition: col.clone(),
                })
            );
        }
    }

    #[test]
    fn test_serialize_integer() {
        // Nullable
        let mut actual = vec![];
        let col = ColumnDefinition {
            name: "id".to_string(),
            nullable: true,
            data_type: DataType::Integer,
        };
        let datum = Some(Datum::Integer(123));

        serialize_datum(&mut actual, &col, &datum).unwrap();

        let mut expected = vec![];
        serialize_null(&mut expected, false).unwrap();
        serialize_int(&mut expected, 123).unwrap();

        assert_eq!(actual, expected);

        // Non-Nullable
        let mut actual = vec![];
        let col = ColumnDefinition {
            name: "id".to_string(),
            nullable: false,
            data_type: DataType::Integer,
        };
        let datum = Some(Datum::Integer(123));

        serialize_datum(&mut actual, &col, &datum).unwrap();

        let mut expected = vec![];
        serialize_int(&mut expected, 123).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_float() {
        // Nullable
        let mut actual = vec![];
        let col = ColumnDefinition {
            name: "id".to_string(),
            nullable: true,
            data_type: DataType::Float,
        };
        let datum = Some(Datum::Float(123.456));

        serialize_datum(&mut actual, &col, &datum).unwrap();

        let mut expected = vec![];
        serialize_null(&mut expected, false).unwrap();
        serialize_float(&mut expected, 123.456).unwrap();

        assert_eq!(actual, expected);

        // Non-Nullable
        let mut actual = vec![];
        let col = ColumnDefinition {
            name: "id".to_string(),
            nullable: false,
            data_type: DataType::Float,
        };
        let datum = Some(Datum::Float(123.456));

        serialize_datum(&mut actual, &col, &datum).unwrap();

        let mut expected = vec![];
        serialize_float(&mut expected, 123.456).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_text() {
        // Nullable
        let mut actual = vec![];
        let col = ColumnDefinition {
            name: "id".to_string(),
            nullable: true,
            data_type: DataType::Text,
        };
        let datum = Some(Datum::Text("hello".to_string()));

        serialize_datum(&mut actual, &col, &datum).unwrap();

        let mut expected = vec![];
        serialize_null(&mut expected, false).unwrap();
        serialize_text(&mut expected, "hello").unwrap();

        assert_eq!(actual, expected);

        // Non-Nullable
        let mut actual = vec![];
        let col = ColumnDefinition {
            name: "id".to_string(),
            nullable: false,
            data_type: DataType::Text,
        };
        let datum = Some(Datum::Text("hello".to_string()));

        serialize_datum(&mut actual, &col, &datum).unwrap();

        let mut expected = vec![];
        serialize_text(&mut expected, "hello").unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_uuid() {
        // Nullable
        let mut actual = vec![];
        let col = ColumnDefinition {
            name: "id".to_string(),
            nullable: true,
            data_type: DataType::Uuid,
        };
        let id = Uuid::new_v4();
        let datum = Some(Datum::Uuid(id));

        serialize_datum(&mut actual, &col, &datum).unwrap();

        let mut expected = vec![];
        serialize_null(&mut expected, false).unwrap();
        serialize_uuid(&mut expected, id).unwrap();

        assert_eq!(actual, expected);

        // Non-Nullable
        let mut actual = vec![];
        let col = ColumnDefinition {
            name: "id".to_string(),
            nullable: false,
            data_type: DataType::Uuid,
        };
        let id = Uuid::new_v4();
        let datum = Some(Datum::Uuid(id));

        serialize_datum(&mut actual, &col, &datum).unwrap();

        let mut expected = vec![];
        serialize_uuid(&mut expected, id).unwrap();

        assert_eq!(actual, expected);
    }
}
