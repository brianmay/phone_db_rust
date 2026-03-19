use bb8::Pool;
use std::env;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

use crate::server::ldap::manager::LdapManager;

#[derive(Clone)]
pub struct LdapPool {
    pool: Arc<bb8::Pool<LdapManager>>,
    base_dn: Arc<String>,
}

impl LdapPool {
    pub async fn get(
        &self,
    ) -> Result<bb8::PooledConnection<'_, LdapManager>, bb8::RunError<ldap3::LdapError>> {
        self.pool.get().await
    }

    pub fn base_dn(&self) -> &str {
        self.base_dn.deref()
    }
}

pub type LdapConnection = ldap3::Ldap;

pub async fn connect_ldap() -> LdapPool {
    let ldap_server = env::var("LDAP_SERVER").expect("LDAP_SERVER must be set");
    let ldap_port = env::var("LDAP_PORT").expect("LDAP_PORT must be set");
    let ldap_base_dn = env::var("LDAP_BASE_DN").expect("LDAP_BASE_DN must be set");
    let ldap_username = env::var("LDAP_USERNAME").expect("LDAP_USERNAME must be set");
    let ldap_password = env::var("LDAP_PASSWORD").expect("LDAP_PASSWORD must be set");
    let url = format!("ldap://{}:{}", ldap_server, ldap_port);
    let url: Url = url.parse().expect("Invalid LDAP URL");

    let manager = LdapManager {
        url: url.to_string(),
        bind_dn: ldap_username.clone(),
        bind_pw: ldap_password.clone(),
    };

    let pool = Pool::builder()
        .max_size(10)
        .connection_timeout(Duration::from_secs(5))
        .build(manager)
        .await
        .unwrap();

    LdapPool {
        pool: Arc::new(pool),
        base_dn: Arc::new(ldap_base_dn),
    }
}
