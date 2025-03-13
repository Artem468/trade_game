#[macro_export]
macro_rules! unwrap_or_http_err {
    ($expr:expr) => {
        match $expr {
            Some(val) => val,
            None => {
                return HttpResponse::BadRequest().json(
                    CommonResponse::<()> {
                        status: ResponseStatus::Error,
                        data: (),
                        error: Some("No item info".into()),
                    }
                )
            }
        }
    };
}


#[macro_export]
macro_rules! unwrap_or_http_err_with_opt_msg {
    ($expr:expr, $str:expr) => {
        match $expr {
            Some(val) => val,
            None => {
                return HttpResponse::BadRequest().json(
                    CommonResponse::<()> {
                        status: ResponseStatus::Error,
                        data: (),
                        error: Some($str.into()),
                    }
                )
            }
        }
    };
}