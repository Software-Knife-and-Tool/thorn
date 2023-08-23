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
            namespace::Namespace,
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
    fn intern(_: &Mu, _: Tag, _: Tag);
    fn map(_: &Mu, _: Tag, _: &str) -> Option<Tag>;
    fn map_ns(_: &Mu, _: &str) -> Option<Tag>;
}

impl NSMaps for Mu {
    type NSCache = RwLock<HashMap<String, Tag>>;
    type NSMap = HashMap<u64, (Tag, Self::NSCache)>;

    fn add_ns(mu: &Mu, ns: Tag) -> exception::Result<Tag> {
        let mut ns_ref = mu.ns_map.write().unwrap();

        if ns_ref.contains_key(&ns.as_u64()) {
            return Err(Exception::new(Condition::Type, "make-ns", ns));
        }

        ns_ref.insert(
            ns.as_u64(),
            (ns, RwLock::new(HashMap::<String, Tag>::new())),
        );

        Ok(ns)
    }

    fn map_ns(mu: &Mu, name: &str) -> Option<Tag> {
        let ns_ref = mu.ns_map.read().unwrap();

        for (_, ns) in ns_ref.iter() {
            let (ns_name, _) = ns;
            let map_name = Vector::as_string(mu, Namespace::name(mu, *ns_name));

            if name == map_name {
                return Some(*ns_name);
            }
        }

        None
    }

    fn map(mu: &Mu, ns: Tag, name: &str) -> Option<Tag> {
        let ns_ref = mu.ns_map.read().unwrap();
        let (_, ns_cache) = &ns_ref[&ns.as_u64()];
        let hash = ns_cache.read().unwrap();

        if hash.contains_key(name) {
            Some(hash[name])
        } else {
            None
        }
    }

    fn intern(mu: &Mu, ns: Tag, symbol: Tag) {
        let ns_ref = mu.ns_map.read().unwrap();
        let (_, ns_cache) = &ns_ref[&ns.as_u64()];
        let name = Vector::as_string(mu, Symbol::name(mu, symbol));

        let mut hash = ns_cache.write().unwrap();
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
