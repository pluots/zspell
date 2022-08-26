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
macro_rules! unwrap_or_ret_err {
    ($ex:expr, $ret:expr) => {
        match $ex {
            Some(v) => v,
            None => return Err($ret),
        }
    };
}

// #[macro_export]
// macro_rules! ok_or_ret {
//     ($ex:expr, e.$ret:tt) => {
//         match $ex {
//             Ok(v) => v,
//             Err(e) => return e.$ret,
//         }
//     };
//     // ($ex:expr, $ret1:expr(e)$ret2:expr) => {
//     //     match $ex.ok() {
//     //         Ok(v) => v,
//     //         Err(e) => return $ret1:expr(e)$ret2:expr,
//     //     }
//     // },
//     ($ex:expr, $ret:tt) => {
//         match $ex.ok() {
//             Ok(v) => v,
//             Err(e) => return $ret,
//         }
//     };
// }
