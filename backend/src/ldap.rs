use std::collections::HashSet;

use itertools::Itertools;
use simple_ldap::filter::{EqFilter, Filter};
use simple_ldap::ldap3::{Mod, ResultEntry, Scope, SearchEntry};
use simple_ldap::LdapClient;
use tap::Pipe;

use crate::contacts::Contact;
use crate::errors::Error;
use crate::Ldap;

use common::Action;

#[derive(Debug)]
#[allow(dead_code)]
pub struct LdapContact {
    dn: String,
    sn: Option<String>,
    cn: Option<String>,
    telephone_number: Option<String>,
}

impl From<ResultEntry> for LdapContact {
    fn from(entry: ResultEntry) -> Self {
        let entry = SearchEntry::construct(entry);

        LdapContact {
            dn: entry.dn.clone(),
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

pub fn include_contact_in_ldap(contact: &Contact) -> bool {
    let tests = [
        contact.name.is_some(),
        contact.phone_number != "anonymous",
        contact.action == Action::Allow,
    ];

    tests.into_iter().all(|f| f)
}

pub async fn get_contact(contact: &Contact, ldap: &Ldap) -> Result<Option<LdapContact>, Error> {
    // get connection will deadlock if called twice from same thread.
    let mut conn = ldap.pool.get_connection().await?;
    internal_get_contact(contact, ldap, &mut conn).await
}

async fn internal_get_contact(
    contact: &Contact,
    ldap: &Ldap,
    conn: &mut LdapClient,
) -> Result<Option<LdapContact>, Error> {
    let filter = EqFilter::from("telephoneNumber".to_string(), contact.phone_number.clone());
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

pub async fn update_contact(contact: &Contact, ldap: &Ldap) -> Result<(), Error> {
    // get connection will deadlock if called twice from same thread.
    let mut conn = ldap.pool.get_connection().await?;
    let mut inner = conn.get_inner();

    match internal_get_contact(contact, ldap, &mut conn).await? {
        Some(ldap_contact) => {
            let dn = &ldap_contact.dn;
            if include_contact_in_ldap(contact) {
                if let Some(name) = &contact.name {
                    let mods = vec![
                        Mod::Replace("cn".to_string(), HashSet::from([name.to_string()])),
                        Mod::Replace("sn".to_string(), HashSet::from([name.to_string()])),
                    ];
                    inner.modify(dn, mods).await?.success()?;
                } else {
                    inner.delete(dn).await?.success()?;
                }
            } else {
                inner.delete(dn).await?.success()?;
            }
        }

        None => {
            let dn = format!("telephoneNumber={},{}", contact.phone_number, ldap.base_dn);
            if include_contact_in_ldap(contact) {
                if let Some(name) = &contact.name {
                    let attrs = vec![
                        ("cn".to_string(), HashSet::from([name.to_string()])),
                        ("sn".to_string(), HashSet::from([name.to_string()])),
                        (
                            "telephoneNumber".to_string(),
                            HashSet::from([contact.phone_number.to_string()]),
                        ),
                        (
                            "objectClass".to_string(),
                            HashSet::from(["person".to_string()]),
                        ),
                    ];
                    inner.add(&dn, attrs).await?.success()?;
                }
            }
        }
    }

    Ok(())
}
