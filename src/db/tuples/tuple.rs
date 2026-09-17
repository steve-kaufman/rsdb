use std::io::{Read, Write};

use crate::db::{Datum, Error, Schema, deserialize_datum, serialize_datum};

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

pub fn deserialize_tuple(buf: &mut impl Read, schema: &Schema) -> Result<Tuple, Error> {
    if schema.is_empty() {
        return Err(Error::DeserializeEmptySchema);
    }
    let mut tuple: Tuple = vec![];
    for col in schema {
        tuple.push(deserialize_datum(buf, col)?);
    }
    Ok(tuple)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use uuid::Uuid;

    use crate::db::{
        ColumnDefinition, DataType, serialize_float, serialize_int, serialize_null, serialize_text,
        serialize_uuid,
    };

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

    #[test]
    fn test_serialize() {
        let schema: Schema = vec![
            ColumnDefinition {
                name: "id".to_string(),
                nullable: false,
                data_type: DataType::Uuid,
            },
            ColumnDefinition {
                name: "balance".to_string(),
                nullable: false,
                data_type: DataType::Float,
            },
            ColumnDefinition {
                name: "first_name".to_string(),
                nullable: true,
                data_type: DataType::Text,
            },
            ColumnDefinition {
                name: "last_name".to_string(),
                nullable: true,
                data_type: DataType::Text,
            },
            ColumnDefinition {
                name: "email".to_string(),
                nullable: false,
                data_type: DataType::Text,
            },
            ColumnDefinition {
                name: "followers".to_string(),
                nullable: false,
                data_type: DataType::Integer,
            },
        ];

        // All nullables non-null

        let mut actual = vec![];
        let id = Uuid::new_v4();
        let tuple: Tuple = vec![
            Some(Datum::Uuid(id)),
            Some(Datum::Float(123.456)),
            Some(Datum::Text("John".to_string())),
            Some(Datum::Text("Doe".to_string())),
            Some(Datum::Text("john.doe@example.com".to_string())),
            Some(Datum::Integer(123)),
        ];

        serialize_tuple(&mut actual, &schema, &tuple).unwrap();

        let mut expected = vec![];
        serialize_uuid(&mut expected, id).unwrap();
        serialize_float(&mut expected, 123.456).unwrap();
        serialize_null(&mut expected, false).unwrap();
        serialize_text(&mut expected, "John").unwrap();
        serialize_null(&mut expected, false).unwrap();
        serialize_text(&mut expected, "Doe").unwrap();
        serialize_text(&mut expected, "john.doe@example.com").unwrap();
        serialize_int(&mut expected, 123).unwrap();

        assert_eq!(actual, expected);

        // All nullables null

        let mut actual = vec![];
        let id = Uuid::new_v4();
        let tuple: Tuple = vec![
            Some(Datum::Uuid(id)),
            Some(Datum::Float(123.456)),
            None,
            None,
            Some(Datum::Text("john.doe@example.com".to_string())),
            Some(Datum::Integer(123)),
        ];

        serialize_tuple(&mut actual, &schema, &tuple).unwrap();

        let mut expected = vec![];
        serialize_uuid(&mut expected, id).unwrap();
        serialize_float(&mut expected, 123.456).unwrap();
        serialize_null(&mut expected, true).unwrap();
        serialize_null(&mut expected, true).unwrap();
        serialize_text(&mut expected, "john.doe@example.com").unwrap();
        serialize_int(&mut expected, 123).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_deserialize_needs_non_empty_schema() {
        let schema: Schema = vec![];

        let res = deserialize_tuple(&mut Cursor::new(vec![]), &schema);

        assert_eq!(res, Err(Error::DeserializeEmptySchema));
    }

    #[test]
    fn test_deserialize() {
        let schema: Schema = vec![
            ColumnDefinition {
                name: "id".to_string(),
                nullable: false,
                data_type: DataType::Uuid,
            },
            ColumnDefinition {
                name: "balance".to_string(),
                nullable: false,
                data_type: DataType::Float,
            },
            ColumnDefinition {
                name: "first_name".to_string(),
                nullable: true,
                data_type: DataType::Text,
            },
            ColumnDefinition {
                name: "last_name".to_string(),
                nullable: true,
                data_type: DataType::Text,
            },
            ColumnDefinition {
                name: "email".to_string(),
                nullable: false,
                data_type: DataType::Text,
            },
            ColumnDefinition {
                name: "followers".to_string(),
                nullable: false,
                data_type: DataType::Integer,
            },
        ];

        // All nullables non-null

        let id = Uuid::new_v4();

        let mut buf = vec![];
        serialize_uuid(&mut buf, id).unwrap();
        serialize_float(&mut buf, 123.456).unwrap();
        serialize_null(&mut buf, false).unwrap();
        serialize_text(&mut buf, "John").unwrap();
        serialize_null(&mut buf, false).unwrap();
        serialize_text(&mut buf, "Doe").unwrap();
        serialize_text(&mut buf, "john.doe@example.com").unwrap();
        serialize_int(&mut buf, 123).unwrap();

        let actual = deserialize_tuple(&mut Cursor::new(buf), &schema).unwrap();

        let expected: Tuple = vec![
            Some(Datum::Uuid(id)),
            Some(Datum::Float(123.456)),
            Some(Datum::Text("John".to_string())),
            Some(Datum::Text("Doe".to_string())),
            Some(Datum::Text("john.doe@example.com".to_string())),
            Some(Datum::Integer(123)),
        ];

        assert_eq!(actual, expected);

        // All nullables null

        let id = Uuid::new_v4();

        let mut buf = vec![];
        serialize_uuid(&mut buf, id).unwrap();
        serialize_float(&mut buf, 123.456).unwrap();
        serialize_null(&mut buf, true).unwrap();
        serialize_null(&mut buf, true).unwrap();
        serialize_text(&mut buf, "john.doe@example.com").unwrap();
        serialize_int(&mut buf, 123).unwrap();

        let actual = deserialize_tuple(&mut Cursor::new(buf), &schema).unwrap();

        let expected: Tuple = vec![
            Some(Datum::Uuid(id)),
            Some(Datum::Float(123.456)),
            None,
            None,
            Some(Datum::Text("john.doe@example.com".to_string())),
            Some(Datum::Integer(123)),
        ];

        assert_eq!(actual, expected);
    }
}
