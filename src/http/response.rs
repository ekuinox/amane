use serde::Serialize;

#[derive(Serialize)]
pub struct Response<T: Serialize> {
    pub ok: bool,
    pub  data: T,
}

#[derive(Serialize)]
pub struct Error {
    pub message: String,
}

impl From<Error> for Response<Error> {
    fn from(error: Error) -> Self {
        Response {
            ok: false,
            data: error,
        }
    }
}
