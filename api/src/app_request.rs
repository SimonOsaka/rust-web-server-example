use std::borrow::Cow;

use crate::{
    app_error::{JWTError, ValidateError},
    app_response::AppError,
};
use axum::{
    async_trait,
    body::HttpBody,
    extract::{FromRequest, Path, Query, RequestParts},
    BoxError, Json,
};
use http_body::Body;
use serde::de::DeserializeOwned;
use util::i18n::i18n;
use util::jwt::{decode_token, Claims};
use validator::{Validate, ValidationErrors};

pub struct JwtAuth(pub Claims);

#[async_trait]
impl<B> FromRequest<B> for JwtAuth
where
    B: Send,
{
    type Rejection = AppError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        if let Some(headers) = req.headers() {
            match headers.get("Authorization") {
                Some(k) => match k.to_str().ok().and_then(|x| decode_token(x).ok()) {
                    Some(k) => Ok(Self(k)),
                    None => Err(AppError::from(JWTError::Invalid)),
                },
                // for no login user
                None =>
                //Ok(Self(role_view())),
                {
                    Err(AppError::from(JWTError::Missing))
                }
            }
        } else {
            // Ok(Self(role_view()))
            Err(AppError::from(JWTError::Missing))
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

#[async_trait]
impl<T, B> FromRequest<B> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    B: Body + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = AppError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Query(value) = Query::<T>::from_request(req)
            .await
            .map_err(|e| AppError::from(ValidateError::AxumQueryRejection(e)))?;
        value.validate().map_err(|e| {
            let ves = to_new_validation_errors(e);
            AppError::from(ValidateError::InvalidParam(ves))
        })?;
        Ok(ValidatedQuery(value))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, B> FromRequest<B> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    B: HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = AppError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req)
            .await
            .map_err(|e| AppError::from(ValidateError::AxumJsonRejection(e)))?;
        value.validate().map_err(|e| {
            let ves = to_new_validation_errors(e);
            AppError::from(ValidateError::InvalidParam(ves))
        })?;
        Ok(ValidatedJson(value))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedPath<T>(pub T);

#[async_trait]
impl<T, B> FromRequest<B> for ValidatedPath<T>
where
    T: DeserializeOwned + Validate + Send,
    B: Send,
{
    type Rejection = AppError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Path(value) = Path::<T>::from_request(req)
            .await
            .map_err(|e| AppError::from(ValidateError::AxumPathRejection(e)))?;
        value.validate().map_err(|e| {
            let ves = to_new_validation_errors(e);
            AppError::from(ValidateError::InvalidParam(ves))
        })?;
        Ok(ValidatedPath(value))
    }
}

fn to_new_validation_errors(e: ValidationErrors) -> ValidationErrors {
    tracing::debug!("e.field_errors(): {:?}", e.field_errors());
    let mut new_validation_errors = ValidationErrors::new();
    for (field, vec_validation_error) in e.field_errors() {
        for validation_err in vec_validation_error {
            tracing::debug!("validation_err.code: {}", validation_err.code);
            let mut new_validation_error = validation_err.clone();
            new_validation_error.message = Some(Cow::from(i18n(
                new_validation_error.code.clone().into_owned().as_str(),
            )));
            new_validation_errors.add(field, new_validation_error);
        }
    }
    tracing::debug!(
        "ves.field_errors(): {:?}",
        new_validation_errors.field_errors()
    );

    new_validation_errors
}