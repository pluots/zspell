use core::fmt::Display;

#[macro_export]
macro_rules! unwrap_or_ret {
    ($ex:expr, $ret:expr) => {
        match $ex.ok() {
            Some(v) => v,
            None => return $ret,
        }
    };
}

#[inline]
pub fn convertu32<T: TryInto<u32> + Display + Copy>(value: T) -> u32 {
    value
        .try_into()
        .unwrap_or_else(|_| panic!("value {value} overflows u32 max of {}", u32::MAX))
}
