use bb8::ManageConnection;
use ldap3::{LdapConnAsync, Scope};

pub struct LdapManager {
    pub url: String,
    pub bind_dn: String,
    pub bind_pw: String,
}

impl ManageConnection for LdapManager {
    type Connection = ldap3::Ldap;
    type Error = ldap3::LdapError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let (conn, mut ldap) = LdapConnAsync::new(&self.url).await?;
        ldap3::drive!(conn);

        ldap.simple_bind(&self.bind_dn, &self.bind_pw)
            .await?
            .success()?;

        Ok(ldap)
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        // A cheap "ping" query
        conn.search("", Scope::Base, "(objectClass=*)", vec!["dn"])
            .await?
            .success()?;

        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}
