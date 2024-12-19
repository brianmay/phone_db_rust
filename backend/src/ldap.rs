use std::collections::HashSet;

use itertools::Itertools;
use simple_ldap::filter::{EqFilter, Filter};
use simple_ldap::ldap3::{Mod, ResultEntry, Scope, SearchEntry};
use simple_ldap::LdapClient;
use tap::Pipe;
use thiserror::Error;

use crate::Ldap;
use common::{Action, ContactDetails};

#[derive(Debug, Error)]
pub enum Error {
    #[error("LDAP error: {0}")]
    Ldap(#[from] simple_ldap::Error),

    #[error("LDAP3 error: {0}")]
    Ldap3(#[from] simple_ldap::ldap3::LdapError),

    #[error("Too many LDAP results")]
    LdapTooManyResults,
}

#[derive(Debug, Clone)]
pub struct Dn(String);

#[derive(Debug)]
#[allow(dead_code)]
pub struct LdapContact {
    dn: Dn,
    sn: Option<String>,
    cn: Option<String>,
    telephone_number: Option<String>,
}

impl LdapContact {
    pub fn dn(&self) -> &Dn {
        &self.dn
    }
}

pub struct AddLdapContact {
    pub sn: Option<String>,
    pub cn: Option<String>,
    pub telephone_number: String,
}

pub struct UpdateLdapContact {
    pub dn: Dn,
    pub sn: Option<String>,
    pub cn: Option<String>,
    pub telephone_number: Option<String>,
}

impl From<ResultEntry> for LdapContact {
    fn from(entry: ResultEntry) -> Self {
        let entry = SearchEntry::construct(entry);

        LdapContact {
            dn: Dn(entry.dn.clone()),
            sn: entry.attrs.get("sn").and_then(|v| v.first()).cloned(),
            cn: entry.attrs.get("cn").and_then(|v| v.first()).cloned(),
            telephone_number: entry
                .attrs
                .get("telephoneNumber")
                .and_then(|v| v.first())
                .cloned(),
        }
    }
}

pub fn include_contact_in_ldap(contact: &ContactDetails) -> bool {
    let tests = [
        contact.name.is_some(),
        contact.phone_number != "anonymous",
        contact.action == Action::Allow,
    ];

    tests.into_iter().all(|f| f)
}

pub async fn get_ldap_contact(
    phone_number: &str,
    ldap: &Ldap,
) -> Result<Option<LdapContact>, Error> {
    // get connection will deadlock if called twice from same thread.
    let mut conn = ldap.pool.get_connection().await?;
    internal_get_contact(phone_number, ldap, &mut conn).await
}

async fn internal_get_contact(
    phone_number: &str,
    ldap: &Ldap,
    conn: &mut LdapClient,
) -> Result<Option<LdapContact>, Error> {
    let filter = EqFilter::from("telephoneNumber".to_string(), phone_number.to_string());
    let mut inner = conn.get_inner();

    let (results, _) = inner
        .search(
            &ldap.base_dn,
            Scope::Subtree,
            &filter.filter(),
            &vec!["sn", "cn", "telephoneNumber"],
        )
        .await?
        .success()?;

    if results.is_empty() {
        return Ok(None);
    }

    results
        .into_iter()
        .exactly_one()
        .map(|v| v.pipe(LdapContact::from).pipe(Some))
        .map_err(|_err| Error::LdapTooManyResults)
}

// pub async fn update_contact(ldap: &Ldap) -> Result<(), Error> {
//     // get connection will deadlock if called twice from same thread.
//     let mut conn = ldap.pool.get_connection().await?;
//     let mut inner = conn.get_inner();

//     match internal_get_contact(contact, ldap, &mut conn).await? {
//         Some(ldap_contact) => {
//             let dn = &ldap_contact.dn;
//             if include_contact_in_ldap(contact) {
//                 if let Some(name) = &contact.name {
//                     let mods = vec![
//                         Mod::Replace("cn".to_string(), HashSet::from([name.to_string()])),
//                         Mod::Replace("sn".to_string(), HashSet::from([name.to_string()])),
//                     ];
//                     inner.modify(dn, mods).await?.success()?;
//                 } else {
//                     inner.delete(dn).await?.success()?;
//                 }
//             } else {
//                 inner.delete(dn).await?.success()?;
//             }
//         }

//         None => {
//             let dn = format!("telephoneNumber={},{}", contact.phone_number, ldap.base_dn);
//             println!("Creating new contact: {}", dn);
//             if include_contact_in_ldap(contact) {
//                 if let Some(name) = &contact.name {
//                     let attrs = vec![
//                         ("cn".to_string(), HashSet::from([name.to_string()])),
//                         ("sn".to_string(), HashSet::from([name.to_string()])),
//                         (
//                             "telephoneNumber".to_string(),
//                             HashSet::from([contact.phone_number.to_string()]),
//                         ),
//                         (
//                             "objectClass".to_string(),
//                             HashSet::from(["person".to_string()]),
//                         ),
//                     ];
//                     inner.add(&dn, attrs).await?.success()?;
//                 }
//             }
//         }
//     }

//     Ok(())
// }

pub async fn add_ldap_contact(request: AddLdapContact, ldap: &Ldap) -> Result<(), Error> {
    // get connection will deadlock if called twice from same thread.
    let conn = ldap.pool.get_connection().await?;
    let mut inner = conn.get_inner();

    let dn = format!(
        "telephoneNumber={},{}",
        request.telephone_number, ldap.base_dn
    );
    let attrs = vec![
        ("cn".to_string(), HashSet::from_iter(request.cn.into_iter())),
        ("sn".to_string(), HashSet::from_iter(request.sn.into_iter())),
        (
            "telephoneNumber".to_string(),
            HashSet::from([request.telephone_number]),
        ),
        (
            "objectClass".to_string(),
            HashSet::from(["person".to_string()]),
        ),
    ];
    inner.add(&dn, attrs).await?.success()?;

    Ok(())
}

pub async fn update_ldap_contact(request: UpdateLdapContact, ldap: &Ldap) -> Result<(), Error> {
    // get connection will deadlock if called twice from same thread.
    let conn = ldap.pool.get_connection().await?;
    let mut inner = conn.get_inner();

    let dn = &request.dn.0;
    let mods = vec![
        Mod::Replace("cn".to_string(), HashSet::from_iter(request.cn.into_iter())),
        Mod::Replace("sn".to_string(), HashSet::from_iter(request.sn.into_iter())),
        Mod::Replace(
            "telephoneNumber".to_string(),
            HashSet::from_iter(request.telephone_number.into_iter()),
        ),
    ];
    inner.modify(dn, mods).await?.success()?;

    Ok(())
}

pub async fn delete_ldap_contact(dn: &Dn, ldap: &Ldap) -> Result<(), Error> {
    // get connection will deadlock if called twice from same thread.
    let conn = ldap.pool.get_connection().await?;
    let mut inner = conn.get_inner();

    inner.delete(&dn.0).await?.success()?;

    Ok(())
}

pub async fn update_ldap_contact_from_contact(
    phone_number: &str,
    contact: &ContactDetails,
    ldap: &Ldap,
) -> Result<(), Error> {
    match get_ldap_contact(phone_number, ldap).await {
        Ok(Some(ldap_contact)) => {
            if include_contact_in_ldap(contact) {
                let request = UpdateLdapContact {
                    dn: ldap_contact.dn().clone(),
                    cn: contact.name.clone(),
                    sn: contact.name.clone(),
                    telephone_number: Some(contact.phone_number.clone()),
                };
                update_ldap_contact(request, ldap).await?;
            } else {
                delete_ldap_contact(ldap_contact.dn(), ldap).await?;
            }
        }
        Ok(None) => {
            if include_contact_in_ldap(contact) {
                let request = AddLdapContact {
                    cn: contact.name.clone(),
                    sn: contact.name.clone(),
                    telephone_number: contact.phone_number.clone(),
                };
                add_ldap_contact(request, ldap).await?;
            }
        }
        Err(_) => {}
    }

    Ok(())
}

pub async fn delete_ldap_contact_from_phone_number(
    phone_number: &str,
    ldap: &Ldap,
) -> Result<(), Error> {
    match get_ldap_contact(phone_number, ldap).await {
        Ok(Some(ldap_contact)) => {
            delete_ldap_contact(ldap_contact.dn(), ldap).await?;
        }
        Ok(None) => {}
        Err(_) => {}
    }

    Ok(())
}
