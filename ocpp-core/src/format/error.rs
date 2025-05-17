#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum GenericError {
    TimeOut,
    Offline,
    General,
    ParsingError,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum OcppError<T> {
    Protocol(T),
    Other(GenericError),
}
