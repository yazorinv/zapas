//! Реализация авторизации и аутентификации пользователя.
//!
//! # Мотивация
//!
//! Есть стандартная идея в токенах - `access` и `refresh` токены.
//! Вопрос в том, как реализовать продление/обновление токена `refresh`.
//!
//! Логично запретить использование старых токенов (например после успешного `prolong`).
//! Проблема в том, что когда именно это делать?
//!
//! Если сразу после создания нового токена, то этот самый новый токен мог не дойти
//! до пользователя. Пользователь не может использовать старый токен - нужно заново
//! вводить логин и пароль.
//!
//! # Решение
//!
//! Вместо этого нужно подтверждение доставки. Для этого можно следить за
//! цепочкой `refresh` токенов.
//! В цепочке предком считается токен, из которого был получен исходный.
//! Потомком - токены, полученные через `prolong(&self)`.
//! Понятно, что непосредственных предков <= 1, а потомков может быть много.
//!
//! Началом такой цепочки будет токен при создании сессии.
//!
//! Почему цепочки?
//!
//! Потому что пусть мы использовали токен $A$ для создания токенов
//! $B, C, D$. Мы все еще можем использовать $A$ (так мы и создали 3 токена).
//!
//! Допустим, мы решим использовать токен $C$. Система это замечает и говорит, что
//! теперь текущий токен - это $C$. Значит, что $A$ больше невалиден. А вместе
//! с ним и $B, D$. Их использовать больше нельзя.
//!
//! Как это достигается?
//!
//! В токене хранится его `ver` и `parent_ver`.
//! А в сессии хранится `ver` текущего токена.
//! Тогда
//! - `token.ver == session.ver` - текущий токен
//! - `token.parent_ver == session.ver` - потомок. Нужно изменить `session.ver = token.ver`
//! - Иначе это неизвестный токен.
//!
//! Получается цепочка, из которой есть лишь небольшие ответления.
use crate::err::{AppErr, ErrorLog};
use chrono::{DateTime, Duration, Utc};
use ipnetwork::IpNetwork;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

/// Время жизни refresh токена.
const REFRESH_EXP: Duration = Duration::days(90);

/// Время жизни access токена.
const ACCESS_EXP: Duration = Duration::seconds(60);

/// Шаг для обновления `last_active_at`.
const UPD_DELTA: Duration = Duration::hours(1);

/// Структура с закодированным токеном.
#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct TokenData {
    pub token: String,
}

/// Необходимая информация для http хендлеров о пользователе.
#[derive(Clone, Copy)]
pub struct UserSession {
    pub user: Uuid,
    pub session: Uuid,
}

/// Stateless токен. Быстрый, короткоживыщий, мало функций.
#[derive(Clone, Serialize, Deserialize)]
pub struct AccessToken {
    user: Uuid,
    session: Uuid,

    access_ver: i64,

    ip: IpAddr,

    exp: i64,
}

/// Statefull токен. Медленный, долгоживущий, много функций.
#[derive(Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    user: Uuid,
    session: Uuid,

    refresh_ver: i64,
    parent_ver: i64,

    ip: IpAddr,

    exp: i64,
}

/// Объединение всех auth токенов.
#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AuthToken {
    Access(AccessToken),
    Refresh(RefreshToken),
}

/// Бизнес сущность для проверок refresh токенов.
pub struct Session {
    pub id: Uuid,
    pub user: Uuid,

    pub ver: i64,

    pub ip: IpAddr,
    pub last_active_at: DateTime<Utc>,

    pub created_at: DateTime<Utc>,

    need_delete: bool,
    need_upd: bool,
}

/// Сервис для кодирования/декодирования `jwt`.
#[derive(Clone)]
pub struct JwtService {
    en: EncodingKey,
    de: DecodingKey,
}

impl Session {
    /// Загрузка сессии из значений ее полей.
    pub fn restore(
        id: Uuid,
        user: Uuid,
        ver: i64,
        ip: IpNetwork,
        last_active_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user,
            ver,
            ip: ip.ip(),
            last_active_at,
            created_at,
            need_delete: false,
            need_upd: false,
        }
    }

    /// Создание новой сессии.
    ///
    /// Происходит, как правило, при `login`.
    /// Поэтому возвращает вместе с сессией и `RefreshToken`.
    pub fn new(user: Uuid, ip: IpAddr) -> (Self, RefreshToken) {
        let now = Utc::now();

        let this = Self {
            id: Uuid::now_v7(),
            user,
            ver: rand::random(),
            ip,
            last_active_at: now,
            created_at: now,

            need_delete: false,
            need_upd: false,
        };

        let token = RefreshToken::from_session(&this);

        (this, token)
    }

    /// Обновление данных сессии.
    ///
    /// А именно `ip` и `last_active_at`.
    fn upd(&mut self, ip: IpAddr) {
        let now = Utc::now();
        if self.ip == ip && (now - self.last_active_at).abs() < UPD_DELTA {
            return;
        }

        self.last_active_at = Utc::now();
        self.ip = ip;
        self.need_upd = true;
    }

    /// Внутренняя проверка `refresh` токена.
    ///
    /// Проверяются:
    /// - Совпадение `user` и `session` id. В случае ошибки возвращается `InvalidToken`.
    ///
    /// - Совпадение `ip` (если `check_ip == true`). В случае ошибки возвращается `NeedProlong` - требуется обновить токен.
    ///
    /// - Совпадение `ver` (`current_ver` если это актуальный токен или `parent_ver` если это новый токен). В случае ошибки сессия блокируется - возможна попытка взлома.
    ///
    /// Функция автоматически обновляет цепочку `refresh` токенов, а также поля
    /// `ip` и `last_active_at`.
    fn check(&mut self, refresh: &RefreshToken, ip: IpAddr, check_ip: bool) -> Result<(), AppErr> {
        if refresh.user != self.user || refresh.session != self.id {
            Err(AppErr::InvalidToken)
        } else if check_ip && refresh.ip != ip {
            Err(AppErr::NeedProlong)
        } else {
            if self.ver != refresh.refresh_ver && refresh.parent_ver == self.ver {
                self.ver = refresh.refresh_ver;
                self.need_upd = true;
            }

            if self.ver != refresh.refresh_ver {
                self.need_delete = true;
                Err(AppErr::DeleteToken)
            } else {
                self.upd(ip);
                Ok(())
            }
        }
    }

    /// Проверка `refresh` токена и использование его для обычных `http api`.
    pub fn validate_refresh(
        &mut self,
        refresh: RefreshToken,
        ip: IpAddr,
    ) -> Result<UserSession, AppErr> {
        self.check(&refresh, ip, true)?;
        Ok(refresh.make_user_session())
    }

    /// Проверка `access` токена (stateless) и использование его для обычных `http api`.
    pub fn validate_access(access: AccessToken, ip: IpAddr) -> Result<UserSession, AppErr> {
        if access.ip != ip {
            Err(AppErr::NeedProlong)
        } else {
            Ok(access.make_user_session())
        }
    }

    /// Создание из `refresh` токена `access` токен.
    pub fn make_access(
        &mut self,
        refresh: RefreshToken,
        ip: IpAddr,
    ) -> Result<AccessToken, AppErr> {
        self.check(&refresh, ip, true)?;
        Ok(AccessToken::from_refresh(&refresh))
    }

    /// Получение нового `refresh` токена из старого.
    ///
    /// Операция не сохраняет новосозданный токен в
    /// сессии - это происходит при первом его использовании.
    pub fn prolong(&mut self, refresh: RefreshToken, ip: IpAddr) -> Result<RefreshToken, AppErr> {
        self.check(&refresh, ip, false)?;
        Ok(refresh.from_parent(ip))
    }

    /// Нужно ли вместо сохранения сессии ее удалять.
    pub fn need_delete(&self) -> bool {
        self.need_delete
    }

    /// Нужно ли сохранять изменения в сессии.
    pub fn need_upd(&self) -> bool {
        self.need_upd
    }
}

impl AccessToken {
    /// Новое время жизни для токена.
    fn new_exp() -> i64 {
        (Utc::now() + ACCESS_EXP).timestamp()
    }

    /// Создание токена из `refresh`.
    fn from_refresh(refresh: &RefreshToken) -> Self {
        Self {
            user: refresh.user,
            session: refresh.session,
            access_ver: refresh.refresh_ver,
            ip: refresh.ip,
            exp: Self::new_exp(),
        }
    }

    /// Выдача информации о пользователе из токена.
    fn make_user_session(&self) -> UserSession {
        UserSession {
            user: self.user,
            session: self.session,
        }
    }
}

impl RefreshToken {
    /// id сессии.
    pub fn session(&self) -> Uuid {
        self.session
    }

    /// Новое время жизни для токена.
    fn new_exp() -> i64 {
        (Utc::now() + REFRESH_EXP).timestamp()
    }

    /// Создание начального элемента цепочки - самого первого токена из сессии.
    fn from_session(session: &Session) -> Self {
        Self {
            user: session.user,
            session: session.id,
            refresh_ver: session.ver,
            parent_ver: session.ver,
            ip: session.ip,
            exp: Self::new_exp(),
        }
    }

    /// Создание нового элемента в цепочке `refresh`.
    fn from_parent(mut self, ip: IpAddr) -> Self {
        self.ip = ip;
        self.parent_ver = self.refresh_ver;
        self.refresh_ver = rand::random();
        self
    }

    /// Выдача информации о пользователе из токена.
    fn make_user_session(&self) -> UserSession {
        UserSession {
            user: self.user,
            session: self.session,
        }
    }
}

impl JwtService {
    const ALGO: jsonwebtoken::Algorithm = jsonwebtoken::Algorithm::HS512;

    const BEARER: &'static str = "Bearer ";

    /// Создание сервиса по секрету.
    pub fn new(secret: &str) -> Self {
        Self {
            en: EncodingKey::from_secret(secret.as_bytes()),
            de: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    /// Кодирование токена.
    pub fn encode(&self, token: impl Into<AuthToken>) -> Result<TokenData, AppErr> {
        let token: AuthToken = token.into();
        let token = encode(&Self::header(), &token, &self.en).log_err()?;
        Ok(TokenData { token })
    }

    /// Декоидрование токена.
    pub fn decode(&self, token: &str) -> Result<AuthToken, AppErr> {
        let token = Self::remove_bearer(token);

        let res = decode(token.as_bytes(), &self.de, &Self::validation())
            .map_err(|_| AppErr::Unauthorized)?
            .claims;
        Ok(res)
    }

    /// Удаление стандартного префикса Bearer.
    fn remove_bearer(s: &str) -> &str {
        let s = s.trim();

        if let Some(s) = s.strip_prefix(Self::BEARER) {
            s.trim()
        } else {
            s
        }
    }

    fn header() -> Header {
        Header {
            alg: Self::ALGO,
            ..Default::default()
        }
    }

    fn validation() -> Validation {
        Validation::new(Self::ALGO)
    }
}

impl From<RefreshToken> for AuthToken {
    fn from(value: RefreshToken) -> Self {
        Self::Refresh(value)
    }
}

impl From<AccessToken> for AuthToken {
    fn from(value: AccessToken) -> Self {
        Self::Access(value)
    }
}
