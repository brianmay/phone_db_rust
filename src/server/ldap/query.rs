use std::collections::HashSet;

use itertools::Itertools;
use ldap3::{Mod, ResultEntry, Scope, SearchEntry};
use tap::Pipe;
use thiserror::Error;

use super::connect::LdapConnection;
use crate::{models::contacts::Contact, server::ldap::filters};

#[derive(Debug, Error)]
pub enum Error {
    #[error("LDAP3 error: {0}")]
    Ldap3(#[from] ldap3::LdapError),

    #[error("Too many LDAP results")]
    LdapTooManyResults,

    #[error("Pool error: {0}")]
    Pool(#[from] bb8::RunError<ldap3::LdapError>),
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

#[derive(Debug)]
pub struct AddLdapContact {
    pub sn: Option<String>,
    pub cn: Option<String>,
    pub telephone_number: String,
}

#[derive(Debug)]
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

pub fn include_contact_in_ldap(contact: &Contact) -> bool {
    let tests = [
        contact.name.is_some(),
        contact.phone_number != "anonymous",
        contact.action == "allow",
    ];

    tests.into_iter().all(|f| f)
}

async fn add_ldap_contact(
    request: AddLdapContact,
    base_dn: &str,
    conn: &mut LdapConnection,
) -> Result<(), Error> {
    // get connection will deadlock if called twice from same thread.
    let dn = format!(
        "telephoneNumber={},{}",
        filters::escape_dn_value(&request.telephone_number),
        base_dn
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
    conn.add(&dn, attrs).await?.success()?;

    Ok(())
}

async fn update_ldap_contact(
    request: UpdateLdapContact,
    base_dn: &str,
    conn: &mut LdapConnection,
) -> Result<(), Error> {
    let old_dn = &request.dn.0;

    // If the phone number changed we must rename the entry via ModifyDN before
    // touching any other attributes, because telephoneNumber forms the RDN and
    // cannot be changed with a plain Modify operation.
    let current_dn = if let Some(ref new_number) = request.telephone_number {
        let new_rdn = format!("telephoneNumber={}", filters::escape_dn_value(new_number));
        // Extract the existing RDN (everything before the first ',').
        let old_rdn = old_dn.split_once(',').map(|(rdn, _)| rdn).unwrap_or(old_dn);
        if old_rdn != new_rdn {
            conn.modifydn(old_dn, &new_rdn, true, None)
                .await?
                .success()?;
            format!("{},{}", new_rdn, base_dn)
        } else {
            old_dn.clone()
        }
    } else {
        old_dn.clone()
    };

    // Modify only cn/sn — telephoneNumber is handled by ModifyDN above and
    // must never appear in a Modify on an entry where it forms the RDN.
    let mods = vec![
        Mod::Replace("cn".to_string(), HashSet::from_iter(request.cn.into_iter())),
        Mod::Replace("sn".to_string(), HashSet::from_iter(request.sn.into_iter())),
    ];
    conn.modify(&current_dn, mods).await?.success()?;

    Ok(())
}

async fn delete_ldap_contact(dn: &Dn, conn: &mut LdapConnection) -> Result<(), Error> {
    // get connection will deadlock if called twice from same thread.
    conn.delete(&dn.0).await?.success()?;

    Ok(())
}

pub async fn get_contact(
    phone_number: &str,
    base_dn: &str,
    conn: &mut LdapConnection,
) -> Result<Option<LdapContact>, Error> {
    let filter = filters::eq("telephoneNumber", phone_number);

    let (results, _) = conn
        .search(
            base_dn,
            Scope::Subtree,
            &filter,
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

pub async fn update_ldap_contact_from_contact(
    phone_number: &str,
    contact: &Contact,
    base_dn: &str,
    conn: &mut LdapConnection,
) -> Result<(), Error> {
    match get_contact(phone_number, base_dn, conn).await {
        Ok(Some(ldap_contact)) => {
            if include_contact_in_ldap(contact) {
                let request = UpdateLdapContact {
                    dn: ldap_contact.dn().clone(),
                    cn: contact.name.clone(),
                    sn: contact.name.clone(),
                    telephone_number: Some(contact.phone_number.clone()),
                };
                update_ldap_contact(request, base_dn, conn).await?;
            } else {
                delete_ldap_contact(ldap_contact.dn(), conn).await?;
            }
        }
        Ok(None) => {
            if include_contact_in_ldap(contact) {
                let request = AddLdapContact {
                    cn: contact.name.clone(),
                    sn: contact.name.clone(),
                    telephone_number: contact.phone_number.clone(),
                };
                add_ldap_contact(request, base_dn, conn).await?;
            }
        }
        Err(err) => {
            tracing::error!(
                "Error getting LDAP contact for phone number {}: {}",
                phone_number,
                err
            );
        }
    }

    Ok(())
}

pub async fn delete_ldap_contact_from_phone_number(
    phone_number: &str,
    base_dn: &str,
    conn: &mut LdapConnection,
) -> Result<(), Error> {
    match get_contact(phone_number, base_dn, conn).await {
        Ok(Some(ldap_contact)) => {
            delete_ldap_contact(ldap_contact.dn(), conn).await?;
        }
        Ok(None) => {}
        Err(_) => {}
    }

    Ok(())
}
