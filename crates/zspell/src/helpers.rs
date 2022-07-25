/// Create a vector of unicode graphemes
/// Each &str within this array is a single unicode character, which
/// is composed of one to four 8-bit integers ("chars")
#[macro_export]
macro_rules! graph_vec {
    ($ex:expr) => {
        $ex.graphemes(true)
            .collect::<Vec<&str>>()
            .iter()
            .map(|s| s.to_string())
            .collect()
    };
}

#[macro_export]
macro_rules! unwrap_or_ret {
    ($ex:expr, $ret:expr) => {
        match $ex.ok() {
            Some(v) => v,
            None => return $ret,
        }
    };
}

#[macro_export]
macro_rules! unwrap_or_ret_e {
    ($ex:expr, $ret:expr) => {
        match $ex {
            Some(v) => v,
            None => return Err($ret),
        }
    };
}
