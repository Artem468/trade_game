use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct CommonResponse<T> {
    pub status: ResponseStatus,
    pub data: T,
    pub error: Option<String>,
}


#[derive(Serialize, Debug)]
pub enum ResponseStatus {
    Ok,
    Error,
}
