use crate::unwrap_or_http_err_with_opt_msg;

#[macro_export]
macro_rules! extract_db_response_or_http_err {
    ($expr:expr) => {
        unwrap_or_http_err!(try_or_http_err!($expr))
    }
}


#[macro_export]
macro_rules! extract_db_response_or_http_err_with_opt_msg {
    ($expr:expr, $str:expr) => {
        unwrap_or_http_err_with_opt_msg!(try_or_http_err!($expr), $str)
    }
}
