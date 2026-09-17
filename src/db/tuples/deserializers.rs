use std::io::Read;

use uuid::Uuid;

use crate::db::Error;

pub fn is_null(reader: &mut impl Read) -> Result<bool, Error> {
    let mut buf: [u8; 1] = [0; 1];
    reader
        .read_exact(&mut buf)
        .map_err(|e| Error::IO(e.to_string()))?;
    match buf[0] {
        0u8 => Ok(true),
        1u8 => Ok(false),
        _ => Err(Error::InvalidNullCheckByte(buf[0])),
    }
}

pub fn deserialize_int(reader: &mut impl Read) -> Result<i64, Error> {
    let mut buf: [u8; 8] = [0; 8];
    reader
        .read_exact(&mut buf)
        .map_err(|e| Error::IO(e.to_string()))?;
    Ok(i64::from_le_bytes(buf))
}

pub fn deserialize_float(reader: &mut impl Read) -> Result<f64, Error> {
    let mut buf: [u8; 8] = [0; 8];
    reader
        .read_exact(&mut buf)
        .map_err(|e| Error::IO(e.to_string()))?;
    Ok(f64::from_le_bytes(buf))
}

pub fn deserialize_text(reader: &mut impl Read) -> Result<String, Error> {
    let mut len_buf: [u8; 2] = [0; 2];
    reader
        .read_exact(&mut len_buf)
        .map_err(|e| Error::IO(e.to_string()))?;
    let len = u16::from_le_bytes(len_buf) as usize;
    if len == 0 {
        return Ok("".to_string());
    }
    let mut text_buf = Vec::<u8>::with_capacity(len);
    let bytes_read = reader
        .take(len as u64)
        .read_to_end(&mut text_buf)
        .map_err(|e| Error::IO(e.to_string()))?;
    if bytes_read != len {
        return Err(Error::NotEnoughTextBytes { len, bytes_read });
    }
    String::from_utf8(text_buf).map_err(|e| Error::InvalidUtf8(e.to_string()))
}

pub fn deserialize_uuid(reader: &mut impl Read) -> Result<Uuid, Error> {
    let mut buf: [u8; 16] = [0; 16];
    reader
        .read_exact(&mut buf)
        .map_err(|e| Error::IO(e.to_string()))?;
    Ok(Uuid::from_bytes_le(buf))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn is_io_error(error: Error) -> bool {
        matches!(error, Error::IO(_))
    }

    #[test]
    fn test_is_null() {
        // Needs 1 byte
        assert!(is_io_error(is_null(&mut Cursor::new(&[])).unwrap_err()));

        // Byte must be 0 or 1
        assert_eq!(
            is_null(&mut Cursor::new(&[2])),
            Err(Error::InvalidNullCheckByte(2))
        );
        assert_eq!(
            is_null(&mut Cursor::new(&[255])),
            Err(Error::InvalidNullCheckByte(255))
        );

        // Exactly 1 byte
        assert_eq!(is_null(&mut Cursor::new(&[0])), Ok(true));
        assert_eq!(is_null(&mut Cursor::new(&[1])), Ok(false));

        // More than 1 byte
        assert_eq!(is_null(&mut Cursor::new(&[0, 1, 2, 3])), Ok(true));
        assert_eq!(is_null(&mut Cursor::new(&[1, 123])), Ok(false));
    }

    #[test]
    fn test_deserialize_int() {
        // Needs 8 bytes
        assert!(is_io_error(
            deserialize_int(&mut Cursor::new(&[1])).unwrap_err()
        ));
        assert!(is_io_error(
            deserialize_int(&mut Cursor::new(&[1, 2, 3, 4, 5, 6, 7])).unwrap_err()
        ));

        // Exactly 8 bytes
        assert_eq!(deserialize_int(&mut Cursor::new(0i64.to_le_bytes())), Ok(0));
        assert_eq!(
            deserialize_int(&mut Cursor::new(i64::MIN.to_le_bytes())),
            Ok(i64::MIN)
        );
        assert_eq!(
            deserialize_int(&mut Cursor::new(i64::MAX.to_le_bytes())),
            Ok(i64::MAX)
        );
        assert_eq!(
            deserialize_int(&mut Cursor::new(12345i64.to_le_bytes())),
            Ok(12345)
        );

        // More than 8 bytes
        assert_eq!(
            deserialize_int(&mut Cursor::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])),
            Ok(i64::from_le_bytes([1, 2, 3, 4, 5, 6, 7, 8]))
        );
    }

    #[test]
    fn test_deserialize_float() {
        // Needs 8 bytes
        assert!(is_io_error(
            deserialize_float(&mut Cursor::new(&[1])).unwrap_err()
        ));
        assert!(is_io_error(
            deserialize_float(&mut Cursor::new(&[1, 2, 3, 4, 5, 6, 7])).unwrap_err()
        ));

        // Exactly 8 bytes
        assert_eq!(
            deserialize_float(&mut Cursor::new(0f64.to_le_bytes())),
            Ok(0.)
        );
        assert_eq!(
            deserialize_float(&mut Cursor::new(f64::MIN.to_le_bytes())),
            Ok(f64::MIN)
        );
        assert_eq!(
            deserialize_float(&mut Cursor::new(f64::MAX.to_le_bytes())),
            Ok(f64::MAX)
        );
        assert_eq!(
            deserialize_float(&mut Cursor::new(12345.0f64.to_le_bytes())),
            Ok(12345.)
        );

        // More than 8 bytes
        assert_eq!(
            deserialize_float(&mut Cursor::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])),
            Ok(f64::from_le_bytes([1, 2, 3, 4, 5, 6, 7, 8]))
        );
    }

    #[test]
    fn test_deserialize_text() {
        // Needs at least 2 bytes
        assert!(is_io_error(
            deserialize_text(&mut Cursor::new(&[])).unwrap_err()
        ));
        assert!(is_io_error(
            deserialize_text(&mut Cursor::new(&[1])).unwrap_err()
        ));

        // len 0 returns ""
        assert_eq!(
            deserialize_text(&mut Cursor::new(&[0, 0])),
            Ok("".to_string())
        );
        assert_eq!(
            deserialize_text(&mut Cursor::new(&[0, 0, 1, 2, 3])),
            Ok("".to_string())
        );

        // len 'n' requires 'n' additional bytes
        assert_eq!(
            deserialize_text(&mut Cursor::new(1i16.to_le_bytes())),
            Err(Error::NotEnoughTextBytes {
                len: 1,
                bytes_read: 0
            })
        );
        assert_eq!(
            deserialize_text(&mut Cursor::new(
                [&1i16.to_le_bytes(), "a".as_bytes()].concat()
            )),
            Ok("a".to_string())
        );
        assert_eq!(
            deserialize_text(&mut Cursor::new(
                [&1i16.to_le_bytes(), "abcd".as_bytes()].concat()
            )),
            Ok("a".to_string())
        );
        assert_eq!(
            deserialize_text(&mut Cursor::new(
                [&5i16.to_le_bytes(), "abcd".as_bytes()].concat()
            )),
            Err(Error::NotEnoughTextBytes {
                len: 5,
                bytes_read: 4
            })
        );
        assert_eq!(
            deserialize_text(&mut Cursor::new(
                [&5i16.to_le_bytes(), "abcde".as_bytes()].concat()
            )),
            Ok("abcde".to_string())
        );
        assert_eq!(
            deserialize_text(&mut Cursor::new(
                [&5i16.to_le_bytes(), "abcdefghi".as_bytes()].concat()
            )),
            Ok("abcde".to_string())
        );
    }

    #[test]
    fn test_deserialize_uuid() {
        // Needs 16 bytes
        assert!(is_io_error(
            deserialize_uuid(&mut Cursor::new(&[])).unwrap_err()
        ));
        assert!(is_io_error(
            deserialize_uuid(&mut Cursor::new(&[
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
            ]))
            .unwrap_err()
        ));

        // Correctly parses uuid
        let uuid1 = Uuid::new_v4();
        assert_eq!(
            deserialize_uuid(&mut Cursor::new(uuid1.to_bytes_le())),
            Ok(uuid1)
        );
        let uuid2 = Uuid::new_v4();
        assert_eq!(
            deserialize_uuid(&mut Cursor::new(
                [uuid2.to_bytes_le().as_slice(), &[1, 2, 3]].concat()
            )),
            Ok(uuid2)
        );
    }
}
