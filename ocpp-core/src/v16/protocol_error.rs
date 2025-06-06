#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum ProtocolError {
    InternalError,
    ProtocolError,
    SecurityError,
    FormationViolation,
    PropertyConstraintViolation,
    OccurrenceConstraintViolation,
    TypeConstraintViolation,
    GenericError,
    NotImplemented,
    NotSupported,
}

impl core::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            ProtocolError::InternalError => "InternalError",
            ProtocolError::ProtocolError => "ProtocolError",
            ProtocolError::SecurityError => "SecurityError",
            ProtocolError::FormationViolation => "FormationViolation",
            ProtocolError::PropertyConstraintViolation => "PropertyConstraintViolation",
            ProtocolError::OccurrenceConstraintViolation => "OccurrenceConstraintViolation",
            ProtocolError::TypeConstraintViolation => "TypeConstraintViolation",
            ProtocolError::GenericError => "GenericError",
            ProtocolError::NotImplemented => "NotImplemented",
            ProtocolError::NotSupported => "NotSupported",
        };
        write!(f, "{s}")
    }
}
