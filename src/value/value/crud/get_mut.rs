use super::Value;
use super::ValueCollection;
use crate::path::BorrowedSegment;

pub fn get_mut<'a>(
    mut value: &mut Value,
    mut path_iter: impl Iterator<Item = BorrowedSegment<'a>>,
) -> Option<&mut Value> {
    loop {
        match (path_iter.next(), value) {
            (None, value) => return Some(value),
            (Some(BorrowedSegment::Field(key)), Value::Object(map)) => {
                let nested_value = map.get_mut_value(key.as_ref())?;
                value = nested_value;
            }
            (Some(BorrowedSegment::Index(index)), Value::Array(array)) => {
                let nested_value = array.get_mut_value(&index)?;
                value = nested_value;
            }
            _ => return None,
        }
    }
}
