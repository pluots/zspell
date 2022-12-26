use core::fmt::Display;

#[inline]
pub fn convertu32<T: TryInto<u32> + Display + Copy>(value: T) -> u32 {
    value
        .try_into()
        .unwrap_or_else(|_| panic!("value {value} overflows u32 max of {}", u32::MAX))
}
