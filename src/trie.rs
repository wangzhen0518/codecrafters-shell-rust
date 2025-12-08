use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Eq, Clone)]
struct TrieNode {
    children: HashMap<char, TrieNode>,
}

impl TrieNode {
    pub fn new() -> TrieNode {
        TrieNode {
            children: HashMap::new(),
        }
    }

    fn add(&mut self, s: &str) {
        let mut current_node = self;
        for c in s.chars() {
            current_node = current_node.children.entry(c).or_insert_with(TrieNode::new);
        }
    }

    fn delete(&mut self, s: &str) {
        fn _delete(node: &mut TrieNode, mut chars: std::str::Chars) -> bool {
            if let Some(c) = chars.next() {
                if let Some(child) = node.children.get_mut(&c) {
                    if _delete(child, chars) && child.children.is_empty() {
                        node.children.remove(&c);
                        return true;
                    }
                }

                false
            } else {
                node.children.is_empty()
            }
        }
        _delete(self, s.chars());
    }

    fn find(&self, s: &str) -> Option<&TrieNode> {
        let mut current_node = self;
        let mut is_matched = true;
        for c in s.chars() {
            match current_node.children.get(&c) {
                Some(next_node) => current_node = next_node,
                None => {
                    is_matched = false;
                    break;
                }
            }
        }

        if is_matched {
            Some(current_node)
        } else {
            None
        }
    }

    fn get_all_chars(&self) -> HashSet<Vec<char>> {
        fn _get_all_chars(
            node: &TrieNode,
            current_path: &mut Vec<char>,
            all_chars: &mut HashSet<Vec<char>>,
        ) {
            if node.children.is_empty() {
                all_chars.insert(current_path.clone());
            } else {
                for (c, child) in node.children.iter() {
                    current_path.push(*c);
                    _get_all_chars(child, current_path, all_chars);
                    current_path.pop();
                }
            }
        }
        let mut current_path = vec![];
        let mut all_chars = HashSet::new();
        _get_all_chars(self, &mut current_path, &mut all_chars);
        all_chars
    }

    fn get_all_strings(&self) -> HashSet<String> {
        fn _get_all_strings(
            node: &TrieNode,
            current_path: &mut Vec<char>,
            all_strings: &mut HashSet<String>,
        ) {
            if node.children.is_empty() {
                all_strings.insert(current_path.iter().collect());
            } else {
                for (c, child) in node.children.iter() {
                    current_path.push(*c);
                    _get_all_strings(child, current_path, all_strings);
                    current_path.pop();
                }
            }
        }
        let mut current_path = vec![];
        let mut all_strings = HashSet::new();
        _get_all_strings(self, &mut current_path, &mut all_strings);
        all_strings
    }

    fn extract(&self, s: &str) -> Option<HashSet<String>> {
        if s.is_empty() && self.children.is_empty() {
            return None;
        }

        self.find(s).map(|node| {
            node.get_all_chars()
                .into_iter()
                .map(|sub_chars| s.chars().chain(sub_chars).collect())
                .collect()
        })
    }
}

impl From<&str> for TrieNode {
    fn from(value: &str) -> Self {
        let mut node = TrieNode::new();
        node.add(value);
        node
    }
}

impl<const N: usize> From<[(char, TrieNode); N]> for TrieNode {
    fn from(value: [(char, TrieNode); N]) -> Self {
        TrieNode {
            children: HashMap::from(value),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Trie {
    root: TrieNode,
}

impl Trie {
    pub fn new() -> Trie {
        Trie {
            root: TrieNode::new(),
        }
    }

    pub fn add(&mut self, s: &str) {
        self.root.add(s);
    }

    pub fn delete(&mut self, s: &str) {
        self.root.delete(s);
    }

    pub fn extract(&self, s: &str) -> Option<HashSet<String>> {
        if s.is_empty() && self.root.children.is_empty() {
            None
        } else {
            self.root.extract(s)
        }
    }
}

impl From<&str> for Trie {
    fn from(value: &str) -> Self {
        Trie {
            root: TrieNode::from(value),
        }
    }
}

impl From<TrieNode> for Trie {
    fn from(value: TrieNode) -> Self {
        Trie { root: value }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::vec_str_to_vec_string;

    use super::*;

    fn create_abc_tree() -> Trie {
        Trie::from(TrieNode::from([(
            'a',
            TrieNode::from([('b', TrieNode::from([('c', TrieNode::new())]))]),
        )]))
    }

    fn create_complex_tree() -> Trie {
        Trie::from(TrieNode::from([
            (
                'a',
                TrieNode::from([
                    (
                        'b',
                        TrieNode::from([('c', TrieNode::new()), ('d', TrieNode::new())]),
                    ),
                    ('e', TrieNode::from([('f', TrieNode::new())])),
                ]),
            ),
            (
                'h',
                TrieNode::from([(
                    'j',
                    TrieNode::from([('k', TrieNode::new()), ('l', TrieNode::new())]),
                )]),
            ),
        ]))
    }

    #[test]
    fn test_trie_from() {
        assert_eq!(Trie::from("abc"), create_abc_tree());
    }

    #[test]
    fn test_trie_add() {
        let mut tree = Trie::from("abc");
        tree.add("abd");
        tree.add("aef");
        tree.add("hjk");
        tree.add("hjl");
        assert_eq!(tree, create_complex_tree());
    }

    #[test]
    fn test_trie_extract() {
        let tree = create_complex_tree();
        assert_eq!(
            tree.extract(""),
            Some(vec_str_to_vec_string(&["aef", "abd", "abc", "hjk", "hjl"]))
        );
        assert_eq!(
            tree.extract("ab"),
            Some(vec_str_to_vec_string(&["abd", "abc"]))
        );
        assert_eq!(tree.extract("abc"), Some(vec_str_to_vec_string(&["abc"])));
        assert_eq!(tree.extract("abcd"), None);
        assert_eq!(tree.extract("abe"), None);
        assert_eq!(tree.extract("pqr"), None);
    }

    #[test]
    fn test_trie_delete() {
        let mut tree = create_complex_tree();
        tree.delete("abcd");
        assert_eq!(
            tree,
            Trie::from(TrieNode::from([
                (
                    'a',
                    TrieNode::from([
                        (
                            'b',
                            TrieNode::from([('c', TrieNode::new()), ('d', TrieNode::new())]),
                        ),
                        ('e', TrieNode::from([('f', TrieNode::new())])),
                    ]),
                ),
                (
                    'h',
                    TrieNode::from([(
                        'j',
                        TrieNode::from([('k', TrieNode::new()), ('l', TrieNode::new())]),
                    )]),
                ),
            ]))
        );
        tree.delete("abd");
        assert_eq!(
            tree,
            Trie::from(TrieNode::from([
                (
                    'a',
                    TrieNode::from([
                        ('b', TrieNode::from([('c', TrieNode::new())])),
                        ('e', TrieNode::from([('f', TrieNode::new())])),
                    ]),
                ),
                (
                    'h',
                    TrieNode::from([(
                        'j',
                        TrieNode::from([('k', TrieNode::new()), ('l', TrieNode::new())]),
                    )]),
                ),
            ]))
        );
        tree.delete("abc");
        assert_eq!(
            tree,
            Trie::from(TrieNode::from([
                (
                    'a',
                    TrieNode::from([('e', TrieNode::from([('f', TrieNode::new())])),]),
                ),
                (
                    'h',
                    TrieNode::from([(
                        'j',
                        TrieNode::from([('k', TrieNode::new()), ('l', TrieNode::new())]),
                    )]),
                ),
            ]))
        );
        tree.delete("aef");
        assert_eq!(
            tree,
            Trie::from(TrieNode::from([(
                'h',
                TrieNode::from([(
                    'j',
                    TrieNode::from([('k', TrieNode::new()), ('l', TrieNode::new())]),
                )]),
            ),]))
        );
    }
}
