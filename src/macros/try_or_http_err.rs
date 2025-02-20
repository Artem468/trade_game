#[macro_export]
macro_rules! try_or_http_err {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                return HttpResponse::InternalServerError().json(
                    CommonResponse::<()> {
                        status: ResponseStatus::Error,
                        data: (),
                        error: Some(err.to_string()),
                    }
                )
            }
        }
    };
}