// Copyright 2021 Remi Bernotavicius

use serde::{Deserialize, Serialize};
use std::collections::{hash_map, HashMap};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct PathTreeNode {
    path: PathBuf,
    num_descendants: usize,
    num_bytes: u64,
    children: HashMap<String, Box<PathTreeNode>>,
}

impl PathTreeNode {
    fn new(path: &Path, num_bytes: u64) -> Self {
        Self {
            path: path.to_owned(),
            num_descendants: 0,
            num_bytes,
            children: HashMap::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.num_descendants + 1
    }

    pub fn num_bytes(&self) -> u64 {
        self.num_bytes
    }

    pub fn children<'a>(&'a self) -> ChildrenForNode<'a> {
        ChildrenForNode {
            node: self,
            keys: self.children.keys(),
        }
    }

    fn add_path(&mut self, path: &Path, num_bytes: u64) -> bool {
        if let Ok(sub_path) = path.strip_prefix(&self.path) {
            let mut iter = sub_path.iter();
            if let Some(next) = iter.next() {
                if iter.next().is_none() {
                    self.children.insert(
                        next.to_str().unwrap().into(),
                        Box::new(PathTreeNode::new(path, num_bytes)),
                    );
                    self.num_descendants += 1;
                    self.num_bytes = self.num_bytes + num_bytes;
                    return true;
                }
                if let Some(c) = self.children.get_mut(next.to_str().unwrap()) {
                    if c.add_path(path, num_bytes) {
                        self.num_descendants += 1;
                        self.num_bytes = self.num_bytes + num_bytes;
                        return true;
                    }
                }
            }
        }

        false
    }
}

#[derive(Serialize, Deserialize)]
pub struct PathTree {
    root: Option<Box<PathTreeNode>>,
}

impl fmt::Display for PathTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_node(f: &mut fmt::Formatter<'_>, n: &PathTreeNode, depth: usize) -> fmt::Result {
            writeln!(
                f,
                "{}{} {} ({})",
                " ".repeat(depth * 4),
                n.path.display(),
                n.num_bytes,
                n.num_descendants
            )?;
            for c in n.children.values() {
                fmt_node(f, c, depth + 1)?;
            }
            Ok(())
        }

        if let Some(root) = &self.root {
            fmt_node(f, root, 0)
        } else {
            writeln!(f, "(empty)")
        }
    }
}

pub struct ChildrenForNode<'a> {
    node: &'a PathTreeNode,
    keys: hash_map::Keys<'a, String, Box<PathTreeNode>>,
}

impl<'a> Iterator for ChildrenForNode<'a> {
    type Item = (&'a str, &'a PathTreeNode);

    fn next(&mut self) -> Option<Self::Item> {
        self.keys
            .next()
            .map(|k| (&k[..], &**self.node.children.get(k).unwrap()))
    }
}

pub struct Children<'a>(Option<ChildrenForNode<'a>>);

impl<'a> Iterator for Children<'a> {
    type Item = <ChildrenForNode<'a> as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(children_for_node) = self.0.as_mut() {
            children_for_node.next()
        } else {
            None
        }
    }
}

impl PathTree {
    pub fn empty() -> Self {
        Self { root: None }
    }

    pub fn size(&self) -> usize {
        if let Some(root) = &self.root {
            root.size()
        } else {
            0
        }
    }

    pub fn num_bytes(&self) -> u64 {
        if let Some(root) = &self.root {
            root.num_bytes()
        } else {
            0
        }
    }

    pub fn add_path(&mut self, path: &Path, num_bytes: u64) {
        if let Some(root) = &mut self.root {
            assert!(root.add_path(path, num_bytes));
        } else {
            self.root = Some(Box::new(PathTreeNode::new(path, num_bytes)));
        }
    }

    pub fn children<'a>(&'a self) -> Children<'a> {
        Children(self.root.as_ref().map(|n| n.children()))
    }
}
