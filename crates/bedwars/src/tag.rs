use protocol::Identifier;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct Tag {
    values: Vec<String>,
}

enum MaybeIdentifier {
    Identifier(Identifier),
    Reference(String),
}

fn collect_tags(
    base: &Path,
    namespace: &str,
) -> io::Result<HashMap<Identifier, Vec<MaybeIdentifier>>> {
    let mut map = HashMap::new();

    fn visit(
        dir: &Path,
        base: &Path,
        out: &mut HashMap<Identifier, Vec<MaybeIdentifier>>,
        namespace: &str,
    ) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let e = entry?;
            let p = e.path();
            if p.is_dir() {
                visit(&p, base, out, namespace)?;
                continue;
            }
            if p.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            let rel = p.strip_prefix(base).unwrap();
            let comps: Vec<_> = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            if comps.is_empty() {
                continue;
            }

            let file_stem = comps.last().unwrap().trim_end_matches(".json");
            // skip comps[0] (registry name), take middle dirs + file stem
            let mut entry_parts: Vec<&str> = comps[1..comps.len() - 1]
                .iter()
                .map(|s| s.as_str())
                .collect();
            entry_parts.push(file_stem);
            let entry_path = entry_parts.join("/");

            let id = Identifier::with_namespace(namespace.to_owned(), entry_path);

            let s = fs::read_to_string(&p)?;
            let values = match serde_json::from_str::<Tag>(&s) {
                Ok(t) => t
                    .values
                    .into_iter()
                    .map(|v| {
                        if v.starts_with('#') {
                            MaybeIdentifier::Reference(v[1..].to_string())
                        } else if v.contains(':') {
                            let (ns, name) = v.split_once(':').unwrap();
                            MaybeIdentifier::Identifier(Identifier::with_namespace(
                                ns.to_owned(),
                                name.to_owned(),
                            ))
                        } else {
                            MaybeIdentifier::Identifier(Identifier::with_namespace(
                                "minecraft",
                                v.to_owned(),
                            ))
                        }
                    })
                    .collect(),
                Err(_) => match serde_json::from_str::<Value>(&s)
                    .ok()
                    .and_then(|v| v.get("values").cloned())
                {
                    Some(Value::Array(arr)) => arr
                        .into_iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .map(|v| {
                            if v.starts_with('#') {
                                MaybeIdentifier::Reference(v[1..].to_string())
                            } else if v.contains(':') {
                                let (ns, name) = v.split_once(':').unwrap();
                                MaybeIdentifier::Identifier(Identifier::with_namespace(
                                    ns.to_owned(),
                                    name.to_owned(),
                                ))
                            } else {
                                MaybeIdentifier::Identifier(Identifier::with_namespace(
                                    "minecraft",
                                    v,
                                ))
                            }
                        })
                        .collect(),
                    _ => Vec::new(),
                },
            };

            out.insert(id, values);
        }
        Ok(())
    }

    visit(base, base, &mut map, namespace)?;
    Ok(map)
}

fn expand_all(
    raw: &HashMap<Identifier, Vec<MaybeIdentifier>>,
) -> HashMap<Identifier, Vec<Identifier>> {
    let mut cache: HashMap<Identifier, Vec<Identifier>> = HashMap::new();

    fn expand_id(
        id: &Identifier,
        raw: &HashMap<Identifier, Vec<MaybeIdentifier>>,
        cache: &mut HashMap<Identifier, Vec<Identifier>>,
        stack: &mut HashSet<Identifier>,
    ) -> Vec<Identifier> {
        if let Some(cached) = cache.get(id) {
            return cached.clone();
        }
        if !stack.insert(id.clone()) {
            return Vec::new();
        }

        let mut out = Vec::new();
        if let Some(values) = raw.get(id) {
            for v in values {
                match v {
                    MaybeIdentifier::Identifier(ident) => out.push(ident.clone()),
                    MaybeIdentifier::Reference(rest) => {
                        let expanded = expand_id(
                            &{
                                if rest.contains(':') {
                                    let (ns, name) = rest.split_once(':').unwrap();
                                    Identifier::with_namespace(ns.to_owned(), name.to_owned())
                                } else {
                                    Identifier::with_namespace("minecraft", rest.clone())
                                }
                            },
                            raw,
                            cache,
                            stack,
                        );
                        out.extend(expanded);
                    }
                }
            }
        }
        stack.remove(id);

        let mut seen = HashSet::new();
        let mut uniq = Vec::new();
        for x in out {
            if seen.insert(x.clone()) {
                uniq.push(x);
            }
        }
        cache.insert(id.clone(), uniq.clone());
        uniq
    }

    for id in raw.keys() {
        let mut stack = HashSet::new();
        let expanded = expand_id(id, raw, &mut cache, &mut stack);
        cache.insert(id.clone(), expanded);
    }
    cache
}

pub fn load_tags(base: &Path, namespace: &str) -> io::Result<HashMap<Identifier, Vec<Identifier>>> {
    let raw = collect_tags(&base, namespace)?;
    Ok(expand_all(&raw))
}
