use crate::utils::{is_tag_name, is_tag_version, package_by_name, tag_prefix};
use cargo_metadata::Metadata;
use git_repository as git;
use git_repository::prelude::ReferenceAccessExt;
use std::path::PathBuf;

/// A head reference will all commits that are 'governed' by it, that is are in its exclusive ancestry.
#[derive(PartialEq, Eq, Debug, Hash, Ord, PartialOrd, Clone)]
pub struct Segment {
    head: git::refs::Reference,
    commits: Vec<git::hash::ObjectId>,
}

/// Return the head reference followed by all tags affecting `crate_name` as per our tag name rules, ordered by ancestry.
pub fn crate_references_descending(
    crate_name: &str,
    meta: &Metadata,
    repo: &git::Easy,
) -> anyhow::Result<Vec<Segment>> {
    let package = package_by_name(meta, crate_name)?;
    let tag_prefix = tag_prefix(package, repo);
    let _tags: Vec<_> = {
        let refs = repo.references()?;
        match tag_prefix {
            Some(prefix) => refs
                .prefixed(PathBuf::from(format!("refs/tags/{}", prefix)))?
                .filter_map(Result::ok)
                .filter(|r| is_tag_name(prefix, r.name().as_bstr()))
                .filter_map(|mut r| r.peel_to_id_in_place().ok().map(|_| r))
                .map(|r| r.detach())
                .collect(),
            None => refs
                .prefixed("refs/tags")?
                .filter_map(Result::ok)
                .filter(|r| is_tag_version(r.name().as_bstr()))
                .filter_map(|mut r| r.peel_to_id_in_place().ok().map(|_| r))
                .map(|r| r.detach())
                .collect(),
        }
    };
    Ok(vec![])
}
