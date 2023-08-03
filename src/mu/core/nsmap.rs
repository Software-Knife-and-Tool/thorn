//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu namespace symbols
use {
    crate::{
        core::{
            exception::{self, Condition, Exception},
            mu::Mu,
            types::Tag,
        },
        types::{
            namespace::{Namespace, Scope},
            symbol::Symbol,
            vector::{Core as _, Vector},
        },
    },
    std::{collections::HashMap, sync::RwLock},
};

pub trait NSMaps {
    type NSCache;
    type NSMap;

    fn add_ns(_: &Mu, _: Tag) -> exception::Result<Tag>;
    fn intern(_: &Mu, _: Tag, _: Scope, _: Tag);
    fn map(_: &Mu, _: Tag, _: Scope, _: &str) -> Option<Tag>;
    fn map_ns(_: &Mu, _: &str) -> Option<Tag>;
}

impl NSMaps for Mu {
    type NSCache = (RwLock<HashMap<String, Tag>>, RwLock<HashMap<String, Tag>>);
    type NSMap = HashMap<u64, (Tag, Self::NSCache)>;

    fn add_ns(mu: &Mu, ns: Tag) -> exception::Result<Tag> {
        let mut ns_ref = mu.ns_map.write().unwrap();

        if ns_ref.contains_key(&ns.as_u64()) {
            return Err(Exception::new(Condition::Type, "make-ns", ns));
        }

        ns_ref.insert(
            ns.as_u64(),
            (
                ns,
                (
                    RwLock::new(HashMap::<String, Tag>::new()),
                    RwLock::new(HashMap::<String, Tag>::new()),
                ),
            ),
        );

        Ok(ns)
    }

    fn map_ns(mu: &Mu, name: &str) -> Option<Tag> {
        let ns_ref = mu.ns_map.read().unwrap();

        for (_, ns_cache) in ns_ref.iter() {
            let (ns, _) = ns_cache;
            let map_name = Vector::as_string(mu, Namespace::name(mu, *ns));

            if name == map_name {
                return Some(*ns);
            }
        }

        None
    }

    fn map(mu: &Mu, ns: Tag, scope: Scope, name: &str) -> Option<Tag> {
        let ns_ref = mu.ns_map.read().unwrap();
        let (_, (externs, interns)) = &ns_ref[&ns.as_u64()];
        let hash = match scope {
            Scope::Intern => externs.read().unwrap(),
            Scope::Extern => interns.read().unwrap(),
        };

        if hash.contains_key(name) {
            Some(hash[name])
        } else {
            None
        }
    }

    fn intern(mu: &Mu, ns: Tag, scope: Scope, symbol: Tag) {
        let ns_ref = mu.ns_map.read().unwrap();
        let (_, (externs, interns)) = &ns_ref[&ns.as_u64()];
        let name = Vector::as_string(mu, Symbol::name(mu, symbol));

        let mut hash = match scope {
            Scope::Intern => externs.write().unwrap(),
            Scope::Extern => interns.write().unwrap(),
        };

        hash.insert(name, symbol);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn namespace() {
        assert_eq!(2 + 2, 4);
    }
}
