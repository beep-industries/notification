use axum::{Form, Json, extract::{FromRequest, Request, rejection::FormRejection}};
use beep_server::ApiError;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use validator::{Validate, ValidationErrors};

pub mod error;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidateJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidateJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {

        // Extract the JSON payload body
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|err| ApiError::Unknown { message: format!("Unexpected payload: {err}") })?;

        // Validate the payload
        value.validate().map_err(map_validation_errors)?;

        Ok(ValidateJson(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldValidationError {
    pub message: String,
    pub field: String,
}

pub fn map_validation_errors(errors: ValidationErrors) -> ApiError {
    let mut validation_errors = Vec::new();

    for (field, error_msgs) in errors.field_errors() {
        for error in error_msgs {
            let message = error
                .message
                .as_ref()
                .map(|cow| cow.to_string())
                .unwrap_or_else(|| format!("Validation failed on {field}"));

            validation_errors.push(FieldValidationError {
                message,
                field: field.to_string(),
            });
        }
    }

    // TODO modify unknown error to a validation error type
    ApiError::Unknown { message: validation_errors
        .into_iter()
        .map(|e| format!("{}: {}", e.field, e.message))
        .collect::<Vec<String>>()
        .join(", ") }
}