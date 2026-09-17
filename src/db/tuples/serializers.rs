use std::io::Write;

use uuid::Uuid;

use crate::db::Error;

pub fn serialize_null(buf: &mut impl Write, is_null: bool) -> Result<(), Error> {
    let byte = if is_null { 0 } else { 1 };
    buf.write_all(&[byte]).map_err(|e| Error::IO(e.to_string()))
}

pub fn serialize_int(buf: &mut impl Write, n: i64) -> Result<(), Error> {
    buf.write_all(&n.to_le_bytes())
        .map_err(|e| Error::IO(e.to_string()))
}

pub fn serialize_float(buf: &mut impl Write, n: f64) -> Result<(), Error> {
    buf.write_all(&n.to_le_bytes())
        .map_err(|e| Error::IO(e.to_string()))
}

pub fn serialize_text(buf: &mut impl Write, text: &str) -> Result<(), Error> {
    if text.len() > u16::MAX as usize {
        return Err(Error::TextOverflow(text.len()));
    }
    buf.write_all(&(text.len() as u16).to_le_bytes())
        .map_err(|e| Error::IO(e.to_string()))?;
    buf.write_all(text.as_bytes())
        .map_err(|e| Error::IO(e.to_string()))
}

pub fn serialize_uuid(buf: &mut impl Write, id: Uuid) -> Result<(), Error> {
    buf.write_all(&id.to_bytes_le())
        .map_err(|e| Error::IO(e.to_string()))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_serialize_null() {
        let mut buf = Cursor::new(vec![]);
        serialize_null(&mut buf, false).unwrap();
        assert_eq!(buf.into_inner(), vec![1]);

        let mut buf = Cursor::new(vec![]);
        serialize_null(&mut buf, true).unwrap();
        assert_eq!(buf.into_inner(), vec![0]);

        let res = serialize_null(&mut Cursor::new([]), true);
        assert!(matches!(res, Err(Error::IO(_))));
    }

    #[test]
    fn test_serialize_int() {
        let mut buf = Cursor::new(vec![]);
        serialize_int(&mut buf, 0).unwrap();
        assert_eq!(buf.into_inner(), vec![0, 0, 0, 0, 0, 0, 0, 0]);

        for n in [1, -1, i64::MIN, i64::MAX] {
            let mut buf = Cursor::new(vec![]);
            serialize_int(&mut buf, n).unwrap();
            assert_eq!(buf.into_inner(), n.to_le_bytes().to_vec());
        }

        let mut buf = Cursor::new([]);
        let res = serialize_int(&mut buf, 123);
        assert!(matches!(res, Err(Error::IO(_))));
    }

    #[test]
    fn test_serialize_float() {
        for n in [0., 1., -1., f64::MIN, f64::MAX] {
            let mut buf = Cursor::new(vec![]);
            serialize_float(&mut buf, n).unwrap();
            assert_eq!(buf.into_inner(), n.to_le_bytes().to_vec());
        }

        let mut buf = Cursor::new([]);
        let res = serialize_float(&mut buf, 123.456);
        assert!(matches!(res, Err(Error::IO(_))));
    }

    #[test]
    fn test_serialize_text() {
        let mut buf = Cursor::new(vec![]);
        serialize_text(&mut buf, "").unwrap();
        assert_eq!(buf.into_inner(), vec![0, 0]);

        let mut buf = Cursor::new(vec![]);
        serialize_text(&mut buf, "hello").unwrap();
        assert_eq!(
            buf.into_inner(),
            [&5u16.to_le_bytes(), "hello".as_bytes()].concat()
        );

        let mut buf = Cursor::new(vec![]);
        let text = vec![b'a'; u16::MAX as usize + 1];
        let res = serialize_text(&mut buf, String::from_utf8(text).unwrap().as_str());
        assert_eq!(res, Err(Error::TextOverflow(u16::MAX as usize + 1)));
    }

    #[test]
    fn test_serialize_uuid() {
        let mut buf = Cursor::new(vec![]);
        serialize_uuid(&mut buf, Uuid::from_u128(0)).unwrap();
        assert_eq!(buf.into_inner(), vec![0; 16]);

        let mut buf = Cursor::new(vec![]);
        let id = Uuid::new_v4();
        serialize_uuid(&mut buf, id).unwrap();
        assert_eq!(buf.into_inner(), id.to_bytes_le().to_vec());
    }
}
