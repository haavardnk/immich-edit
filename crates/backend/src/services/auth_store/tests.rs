use super::*;
use crate::services::edits_store::EditsStore;

async fn store() -> AuthStore {
    let edits = EditsStore::migrated_memory().await.unwrap();
    let dir = tempfile::tempdir().unwrap().keep();
    let crypto =
        Arc::new(InstanceCrypto::load_or_create(&dir.join("instance.key"), false).unwrap());
    AuthStore::new(edits.pool(), crypto)
}

fn immich_user(admin: bool) -> ImmichUser {
    ImmichUser {
        id: Uuid::new_v4(),
        email: "a@b.test".into(),
        name: "A".into(),
        is_admin: admin,
    }
}

#[tokio::test]
async fn session_roundtrip_returns_context() {
    let s = store().await;
    sqlx::query("UPDATE instance_config SET server_epoch = 1 WHERE id = 1")
        .execute(&s.pool)
        .await
        .unwrap();
    let user = s.upsert_user(&immich_user(true)).await.unwrap();
    let token = s
        .create_session(user.id, AuthKind::Password, b"bearer-xyz", 1, None, None)
        .await
        .unwrap();
    let ctx = s.authenticate(&token).await.unwrap().unwrap();
    assert_eq!(ctx.user.id, user.id);
    assert!(ctx.user.is_admin);
    assert_eq!(ctx.immich_cred.as_slice(), b"bearer-xyz");
}

#[tokio::test]
async fn auth_kind_survives_the_session_row() {
    let s = store().await;
    sqlx::query("UPDATE instance_config SET server_epoch = 1 WHERE id = 1")
        .execute(&s.pool)
        .await
        .unwrap();
    for kind in [AuthKind::Password, AuthKind::ApiKey, AuthKind::OAuth] {
        let user = s.upsert_user(&immich_user(true)).await.unwrap();
        let token = s
            .create_session(user.id, kind, b"cred", 1, None, None)
            .await
            .unwrap();
        let ctx = s.authenticate(&token).await.unwrap().unwrap();
        assert_eq!(ctx.auth_kind, kind, "{}", kind.as_str());
    }
}

#[test]
fn oauth_credentials_are_bearer_tokens() {
    assert!(matches!(
        AuthKind::OAuth.immich_auth("t".into()),
        ImmichAuth::Bearer(_)
    ));
    assert!(matches!(
        AuthKind::ApiKey.immich_auth("t".into()),
        ImmichAuth::ApiKey(_)
    ));
    assert!(AuthKind::OAuth.revokes_upstream());
    assert!(!AuthKind::ApiKey.revokes_upstream());
}

#[tokio::test]
async fn unknown_token_rejected() {
    let s = store().await;
    assert!(s.authenticate("nope").await.unwrap().is_none());
}

#[tokio::test]
async fn disabled_user_denied() {
    let s = store().await;
    sqlx::query("UPDATE instance_config SET server_epoch = 1 WHERE id = 1")
        .execute(&s.pool)
        .await
        .unwrap();
    let user = s.upsert_user(&immich_user(false)).await.unwrap();
    let token = s
        .create_session(user.id, AuthKind::ApiKey, b"key", 1, None, None)
        .await
        .unwrap();
    s.set_access(user.id, false).await.unwrap();
    assert!(s.authenticate(&token).await.unwrap().is_none());
}

#[tokio::test]
async fn revoke_ends_session() {
    let s = store().await;
    sqlx::query("UPDATE instance_config SET server_epoch = 1 WHERE id = 1")
        .execute(&s.pool)
        .await
        .unwrap();
    let user = s.upsert_user(&immich_user(false)).await.unwrap();
    let token = s
        .create_session(user.id, AuthKind::Password, b"c", 1, None, None)
        .await
        .unwrap();
    let sessions = s.list_sessions(user.id).await.unwrap();
    assert_eq!(sessions.len(), 1);
    s.revoke_session(sessions[0].id).await.unwrap();
    assert!(s.authenticate(&token).await.unwrap().is_none());
}

#[tokio::test]
async fn upsert_preserves_access_flag() {
    let s = store().await;
    let iu = immich_user(false);
    let user = s.upsert_user(&iu).await.unwrap();
    s.set_access(user.id, false).await.unwrap();
    let again = s.upsert_user(&iu).await.unwrap();
    assert!(!again.access_enabled);
}
