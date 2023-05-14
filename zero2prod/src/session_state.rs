use axum::{async_trait, extract::FromRequestParts};
use axum_session::{DatabasePool, Session};
use uuid::Uuid;

pub struct TypedSession<T>(Session<T>)
where
    T: DatabasePool + Clone + std::fmt::Debug + Sync + Send + 'static;
impl<T> TypedSession<T>
where
    T: DatabasePool + Clone + std::fmt::Debug + Sync + Send + 'static,
{
    const USER_ID_KEY: &'static str = "user_id";

    pub fn get_user_id(&self) -> Option<Uuid> {
        self.0.get(Self::USER_ID_KEY)
    }

    pub fn insert_user_id(&self, user_id: Uuid) {
        self.0.set(Self::USER_ID_KEY, user_id)
    }

    pub fn log_out(self) {
        self.0.destroy();
    }

    pub fn renew(&self) {
        self.0.renew();
    }
}

#[async_trait]
impl<T, S> FromRequestParts<S> for TypedSession<T>
where
    T: DatabasePool + Clone + std::fmt::Debug + Sync + Send + 'static,
    S: Send + Sync,
{
    type Rejection = <Session<T> as FromRequestParts<S>>::Rejection;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session: Session<T> = Session::<T>::from_request_parts(parts, state).await?;

        Ok(TypedSession(session))
    }
}
