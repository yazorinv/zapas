//! Модуль с описанием всех ошибок программы.
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;
use tracing::{error, warn};
use utoipa::ToSchema;

/// Глобальная ошибка всей программы. Происходит при конфигах и тд.
pub type GlobalErr = Box<dyn std::error::Error>;

/// Общий enum ошибок программы.
///
/// Активно используется в API.
#[derive(Debug, Error)]
pub enum AppErr {
    #[error("Not implemented")]
    NotImpl,

    #[error("Internal server error")]
    Internal,

    #[error("Validation error: {0}")]
    Validate(#[from] validator::ValidationErrors),

    #[error("Validation error: {0}")]
    CustomValidate(&'static str),

    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Unathorized")]
    Unauthorized,

    #[error("Need prolongation")]
    NeedProlong,

    #[error("Token deleted")]
    DeleteToken,

    #[error("Not found")]
    NotFound,

    #[error("Wrong username or password")]
    BadUsername,

    #[error("Wrong username or password")]
    BadPasswd,

    #[error("Already exists: {0}")]
    AlreadyExists(&'static str),

    #[error("Permission denied")]
    PermDenied,
}

/// Возвращаемый тип ошибки для пользователя.
#[derive(Serialize, ToSchema)]
pub struct AppErrJson {
    code: u16,
    msg: String,
}

/// Трейт логгирования.
///
/// Необходим для того, чтобы удобно логгировать
/// все ошибки, которые не нужно обрабатывать. Пользователю
/// возвращается `INTERNAL_SERVER_ERROR`, в логи пишется
/// истинная причина.
pub trait ErrorLog<T> {
    fn log_warn(self) -> Result<T, AppErr>;

    fn log_err(self) -> Result<T, AppErr>;
}

impl<T, E: std::fmt::Display> ErrorLog<T> for Result<T, E> {
    #[track_caller]
    fn log_warn(self) -> Result<T, AppErr> {
        self.map_err(|err| {
            warn!("Internal error: {}", err);
            AppErr::Internal
        })
    }

    #[track_caller]
    fn log_err(self) -> Result<T, AppErr> {
        self.map_err(|err| {
            error!("Internal error: {}", err);
            AppErr::Internal
        })
    }
}

impl AppErr {
    /// HTTP код ошибки.
    fn code(&self) -> StatusCode {
        match self {
            AppErr::NotImpl => StatusCode::NOT_IMPLEMENTED,
            AppErr::Internal => StatusCode::INTERNAL_SERVER_ERROR,

            AppErr::Validate(_) | AppErr::CustomValidate(_) => StatusCode::BAD_REQUEST,

            AppErr::InvalidToken => StatusCode::BAD_REQUEST,

            AppErr::Unauthorized
            | AppErr::NeedProlong
            | AppErr::DeleteToken
            | AppErr::BadUsername
            | AppErr::BadPasswd => StatusCode::UNAUTHORIZED,

            AppErr::NotFound => StatusCode::NOT_FOUND,

            AppErr::AlreadyExists(_) => StatusCode::CONFLICT,

            AppErr::PermDenied => StatusCode::FORBIDDEN,
        }
    }

    /// Логирование для AppErr.
    fn trace(&self) {
        warn!("AppErr: {} --- {:?}", self, self);
    }
}

impl IntoResponse for AppErr {
    fn into_response(self) -> axum::response::Response {
        self.trace();

        let code = self.code();
        let json = AppErrJson {
            code: code.as_u16(),
            msg: format!("{}", self),
        };

        (code, Json(json)).into_response()
    }
}

impl From<sqlx::Error> for AppErr {
    #[track_caller]
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound,

            other => {
                error!("Internal error: {}", other);
                Self::Internal
            }
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AppErr {
    fn from(x: jsonwebtoken::errors::Error) -> Self {
        warn!("JWT error: {}", x);
        Self::InvalidToken
    }
}
