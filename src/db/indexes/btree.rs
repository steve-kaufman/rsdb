use crate::db::Result;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
pub struct NodePointer<K> {
    key: K,
    index: usize,
}

impl<K> NodePointer<K> {
    pub fn new(key: K, index: usize) -> Self {
        Self { key, index }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry<K, V> {
    key: K,
    value: V,
}

impl<K, V> Entry<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InnerNode<K> {
    pointers: Vec<NodePointer<K>>,
}

impl<K> InnerNode<K> {
    pub fn new(pointers: Vec<NodePointer<K>>) -> Self {
        Self { pointers }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeafNode<K, V> {
    entries: Vec<Entry<K, V>>,
    next: Option<usize>,
}

impl<K, V> LeafNode<K, V> {
    pub fn new(entries: Vec<Entry<K, V>>, next: Option<usize>) -> Self {
        Self { entries, next }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node<K, V> {
    Inner(InnerNode<K>),
    Leaf(LeafNode<K, V>),
}

pub trait NodeSource<K, V> {
    fn get_node(&self, index: usize) -> Result<Node<K, V>>;
    fn create_node(&mut self, node: Node<K, V>) -> Result<usize>;
    fn update_node(&mut self, index: usize, node: &Node<K, V>) -> Result<()>;
    fn delete_node(&mut self, index: usize) -> Result<bool>;
}

pub struct BTree<K, V> {
    order: usize,
    nodes: Box<dyn NodeSource<K, V>>,
}

impl<K, V> BTree<K, V>
where
    K: Clone + Ord + Debug,
    V: Clone + Debug,
{
    pub fn new(order: usize, nodes: Box<dyn NodeSource<K, V>>) -> Self {
        Self { order, nodes }
    }

    pub fn get(&self, key: &K) -> Result<Option<V>> {
        self.get_from_node(key, &self.nodes.get_node(0)?)
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<()> {
        let entry = Entry::new(key, value);
        let mut root = self.nodes.get_node(0)?;
        let new_right = self.insert_into_node(&mut root, entry)?;
        if let Some(new_right) = new_right {
            self.split_root(&mut root, new_right)?;
        }
        self.nodes.update_node(0, &root)
    }

    pub fn update(&mut self, key: &K, value: V) -> Result<bool> {
        self.update_in_node(0, key, value)
    }

    fn get_from_node(&self, key: &K, node: &Node<K, V>) -> Result<Option<V>> {
        match node {
            Node::Leaf(leaf) => Ok(self.get_from_leaf(key, leaf)),
            Node::Inner(inner) => self.get_from_inner(key, inner),
        }
    }

    fn get_from_leaf(&self, key: &K, leaf: &LeafNode<K, V>) -> Option<V> {
        for entry in leaf.entries.iter() {
            if &entry.key > key {
                return None;
            }
            if &entry.key == key {
                return Some(entry.value.clone());
            }
        }
        None
    }

    fn get_from_inner(&self, key: &K, inner: &InnerNode<K>) -> Result<Option<V>> {
        for i in 0..(inner.pointers.len() - 1) {
            if &inner.pointers[i + 1].key > key {
                return self.get_from_node(key, &self.nodes.get_node(inner.pointers[i].index)?);
            }
        }
        self.get_from_node(
            key,
            &self.nodes.get_node(inner.pointers.last().unwrap().index)?,
        )
    }

    fn insert_into_node(
        &mut self,
        node: &mut Node<K, V>,
        entry: Entry<K, V>,
    ) -> Result<Option<NodePointer<K>>> {
        match node {
            Node::Leaf(leaf) => self.insert_into_leaf(entry, leaf),
            Node::Inner(inner) => self.insert_into_inner(entry, inner),
        }
    }

    fn insert_into_leaf(
        &mut self,
        entry: Entry<K, V>,
        leaf: &mut LeafNode<K, V>,
    ) -> Result<Option<NodePointer<K>>> {
        let insertion_index = self.find_insertion_index(&entry.key, leaf);
        leaf.entries.insert(insertion_index, entry);
        if leaf.entries.len() > self.order {
            Ok(Some(self.split_leaf(leaf)?))
        } else {
            Ok(None)
        }
    }

    fn insert_into_inner(
        &mut self,
        entry: Entry<K, V>,
        inner: &mut InnerNode<K>,
    ) -> Result<Option<NodePointer<K>>> {
        let child_index = self.find_child_index(&entry.key, inner);
        let child_pointer = inner.pointers[child_index].index;
        let mut child = self.nodes.get_node(child_pointer)?;
        let new_right = self.insert_into_node(&mut child, entry)?;
        self.nodes.update_node(child_pointer, &child)?;
        if let Some(new_right) = new_right {
            inner.pointers.insert(child_index + 1, new_right);
            if inner.pointers.len() > self.order {
                return Ok(Some(self.split_inner(inner)?));
            }
        }
        Ok(None)
    }

    fn find_insertion_index(&self, key: &K, leaf: &LeafNode<K, V>) -> usize {
        for (i, entry) in leaf.entries.iter().enumerate() {
            if &entry.key >= key {
                return i;
            }
        }
        leaf.entries.len()
    }

    fn find_child_index(&self, key: &K, inner: &InnerNode<K>) -> usize {
        for i in 0..(inner.pointers.len() - 1) {
            if &inner.pointers[i + 1].key >= key {
                return i;
            }
        }
        inner.pointers.len() - 1
    }

    fn split_leaf(&mut self, leaf: &mut LeafNode<K, V>) -> Result<NodePointer<K>> {
        let split_index = self.order.div_ceil(2);

        // Build new right leaf
        let mut right = LeafNode::new(vec![], None);
        for i in split_index..leaf.entries.len() {
            right.entries.push(leaf.entries[i].clone());
        }
        right.next = leaf.next;

        // Allocate new right leaf
        let right_key = right.entries[0].key.clone();
        let right_index = self.nodes.create_node(Node::Leaf(right))?;

        // Build new left leaf
        leaf.entries.truncate(split_index);
        leaf.next = Some(right_index);

        Ok(NodePointer::new(right_key, right_index))
    }

    fn split_inner(&mut self, inner: &mut InnerNode<K>) -> Result<NodePointer<K>> {
        let split_index = self.order.div_ceil(2);

        let mut right = InnerNode::new(vec![]);
        for i in split_index..inner.pointers.len() {
            right.pointers.push(inner.pointers[i].clone());
        }

        let right_key = right.pointers[0].key.clone();
        let right_index = self.nodes.create_node(Node::Inner(right))?;

        inner.pointers.truncate(split_index);

        Ok(NodePointer::new(right_key, right_index))
    }

    fn split_root(&mut self, root: &mut Node<K, V>, new_right: NodePointer<K>) -> Result<()> {
        let new_left_pointer = self.nodes.create_node(root.clone())?;
        let new_left_key = match root {
            Node::Leaf(leaf) => leaf.entries.first().unwrap().key.clone(),
            Node::Inner(inner) => inner.pointers.first().unwrap().key.clone(),
        };
        *root = Node::Inner(InnerNode::new(vec![
            NodePointer::new(new_left_key, new_left_pointer),
            new_right,
        ]));
        Ok(())
    }

    fn update_in_node(&mut self, pointer: usize, k: &K, v: V) -> Result<bool> {
        let mut node = self.nodes.get_node(pointer)?;
        match &mut node {
            Node::Leaf(leaf) => {
                let was_updated = self.update_in_leaf(leaf, k, v);
                if was_updated {
                    self.nodes.update_node(pointer, &node)?;
                }
                Ok(was_updated)
            }
            Node::Inner(inner) => self.update_in_inner(inner, k, v),
        }
    }

    fn update_in_leaf(&mut self, leaf: &mut LeafNode<K, V>, k: &K, v: V) -> bool {
        for entry in leaf.entries.iter_mut() {
            if &entry.key == k {
                entry.value = v;
                return true;
            } else {
                println!("{:?} != {:?}", &entry.key, k);
            }
        }
        false
    }

    fn update_in_inner(&mut self, inner: &mut InnerNode<K>, k: &K, v: V) -> Result<bool> {
        let child_index = self.find_update_child_index(inner, k);
        self.update_in_node(inner.pointers[child_index].index, k, v)
    }

    fn find_update_child_index(&self, inner: &InnerNode<K>, k: &K) -> usize {
        for i in 0..(inner.pointers.len() - 1) {
            if &inner.pointers[i + 1].key > k {
                return i;
            }
        }
        inner.pointers.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    type NodeVec<K, V> = Arc<Mutex<Vec<Option<Node<K, V>>>>>;

    pub struct VecNodeSource<K, V> {
        nodes: NodeVec<K, V>,
    }

    impl<K, V> VecNodeSource<K, V>
    where
        K: Clone,
        V: Clone,
    {
        pub fn new(nodes: NodeVec<K, V>) -> Self {
            Self { nodes }
        }
    }

    impl<K, V> NodeSource<K, V> for VecNodeSource<K, V>
    where
        K: Clone,
        V: Clone,
    {
        fn get_node(&self, index: usize) -> Result<Node<K, V>> {
            let nodes = self.nodes.lock().unwrap();
            Ok(nodes[index].as_ref().unwrap().clone())
        }

        fn create_node(&mut self, node: Node<K, V>) -> Result<usize> {
            let mut nodes = self.nodes.lock().unwrap();
            nodes.push(Some(node));
            Ok(nodes.len() - 1)
        }

        fn update_node(&mut self, index: usize, node: &Node<K, V>) -> Result<()> {
            let mut nodes = self.nodes.lock().unwrap();
            *nodes.get_mut(index).unwrap() = Some(node.clone());
            Ok(())
        }

        fn delete_node(&mut self, index: usize) -> Result<bool> {
            let mut nodes = self.nodes.lock().unwrap();
            if let Some(node) = nodes.get_mut(index)
                && !node.is_none()
            {
                *node = None;
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    #[test]
    fn test_b_tree_get() {
        // Empty
        let nodes = Arc::new(Mutex::new(vec![Some(Node::Leaf(LeafNode::new(
            vec![],
            None,
        )))]));
        let btree = BTree::<usize, String>::new(3, Box::new(VecNodeSource::new(nodes)));

        assert_eq!(btree.get(&1), Ok(None));

        // Three nodes in root
        let nodes = Arc::new(Mutex::new(vec![Some(Node::Leaf(LeafNode::new(
            vec![
                Entry::new(1, "foo".to_string()),
                Entry::new(2, "bar".to_string()),
                Entry::new(3, "baz".to_string()),
            ],
            None,
        )))]));
        let node_source = VecNodeSource::<usize, String>::new(nodes);

        let btree = BTree::new(3, Box::new(node_source));
        assert_eq!(btree.get(&1), Ok(Some("foo".to_string())));
        assert_eq!(btree.get(&2), Ok(Some("bar".to_string())));
        assert_eq!(btree.get(&3), Ok(Some("baz".to_string())));

        // Nested
        let nodes = Arc::new(Mutex::new(vec![
            Some(Node::Inner(InnerNode {
                pointers: vec![
                    NodePointer::new(1, 1),
                    NodePointer::new(3, 3),
                    NodePointer::new(10, 2),
                ],
            })),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(1, "foo".to_string()),
                    Entry::new(2, "bar".to_string()),
                ],
                Some(3),
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(10, "foo10".to_string()),
                    Entry::new(11, "bar11".to_string()),
                ],
                None,
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(3, "foo3".to_string()),
                    Entry::new(5, "bar5".to_string()),
                ],
                Some(2),
            ))),
        ]));
        let node_source = VecNodeSource::new(nodes);
        let btree = BTree::new(3, Box::new(node_source));

        assert_eq!(btree.get(&1), Ok(Some("foo".to_string())));
        assert_eq!(btree.get(&2), Ok(Some("bar".to_string())));
        assert_eq!(btree.get(&3), Ok(Some("foo3".to_string())));
        assert_eq!(btree.get(&4), Ok(None));
        assert_eq!(btree.get(&5), Ok(Some("bar5".to_string())));
        assert_eq!(btree.get(&10), Ok(Some("foo10".to_string())));
        assert_eq!(btree.get(&11), Ok(Some("bar11".to_string())));
    }

    #[test]
    fn test_insert() {
        let nodes = Arc::new(Mutex::new(vec![Some(Node::Leaf(LeafNode::new(
            vec![],
            None,
        )))]));
        let node_source = Box::new(VecNodeSource::<usize, String>::new(nodes.clone()));
        let mut btree = BTree::new(3, node_source);

        btree.insert(5, "5".to_string()).unwrap();

        assert_eq!(
            nodes.lock().unwrap().as_ref(),
            vec![Some(Node::Leaf(LeafNode::new(
                vec![Entry::new(5, "5".to_string())],
                None
            )))]
        );

        btree.insert(7, "7".to_string()).unwrap();

        assert_eq!(
            nodes.lock().unwrap().as_ref(),
            vec![Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(5, "5".to_string()),
                    Entry::new(7, "7".to_string()),
                ],
                None
            )))]
        );

        btree.insert(3, "3".to_string()).unwrap();

        assert_eq!(
            nodes.lock().unwrap().as_ref(),
            vec![Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(3, "3".to_string()),
                    Entry::new(5, "5".to_string()),
                    Entry::new(7, "7".to_string()),
                ],
                None
            )))]
        );

        btree.insert(1, "1".to_string()).unwrap();

        assert_eq!(
            nodes.lock().unwrap().as_ref(),
            vec![
                Some(Node::Inner(InnerNode::new(vec![
                    NodePointer::new(1, 2),
                    NodePointer::new(5, 1)
                ]))),
                Some(Node::Leaf(LeafNode::new(
                    vec![
                        Entry::new(5, "5".to_string()),
                        Entry::new(7, "7".to_string()),
                    ],
                    None,
                ))),
                Some(Node::Leaf(LeafNode::new(
                    vec![
                        Entry::new(1, "1".to_string()),
                        Entry::new(3, "3".to_string()),
                    ],
                    Some(1),
                ))),
            ]
        );

        btree.insert(6, "6".to_string()).unwrap();

        assert_eq!(
            nodes.lock().unwrap().as_ref(),
            vec![
                Some(Node::Inner(InnerNode::new(vec![
                    NodePointer::new(1, 2),
                    NodePointer::new(5, 1)
                ]))),
                Some(Node::Leaf(LeafNode::new(
                    vec![
                        Entry::new(5, "5".to_string()),
                        Entry::new(6, "6".to_string()),
                        Entry::new(7, "7".to_string()),
                    ],
                    None,
                ))),
                Some(Node::Leaf(LeafNode::new(
                    vec![
                        Entry::new(1, "1".to_string()),
                        Entry::new(3, "3".to_string()),
                    ],
                    Some(1),
                ))),
            ]
        );

        btree.insert(8, "8".to_string()).unwrap();

        assert_eq!(
            nodes.lock().unwrap().as_ref(),
            vec![
                Some(Node::Inner(InnerNode::new(vec![
                    NodePointer::new(1, 2),
                    NodePointer::new(5, 1),
                    NodePointer::new(7, 3),
                ]))),
                Some(Node::Leaf(LeafNode::new(
                    vec![
                        Entry::new(5, "5".to_string()),
                        Entry::new(6, "6".to_string()),
                    ],
                    Some(3),
                ))),
                Some(Node::Leaf(LeafNode::new(
                    vec![
                        Entry::new(1, "1".to_string()),
                        Entry::new(3, "3".to_string()),
                    ],
                    Some(1),
                ))),
                Some(Node::Leaf(LeafNode::new(
                    vec![
                        Entry::new(7, "7".to_string()),
                        Entry::new(8, "8".to_string()),
                    ],
                    None,
                ))),
            ]
        );

        btree.insert(2, "2".to_string()).unwrap();

        assert_eq!(
            nodes.lock().unwrap().as_ref(),
            vec![
                Some(Node::Inner(InnerNode::new(vec![
                    NodePointer::new(1, 2),
                    NodePointer::new(5, 1),
                    NodePointer::new(7, 3),
                ]))),
                Some(Node::Leaf(LeafNode::new(
                    vec![
                        Entry::new(5, "5".to_string()),
                        Entry::new(6, "6".to_string()),
                    ],
                    Some(3),
                ))),
                Some(Node::Leaf(LeafNode::new(
                    vec![
                        Entry::new(1, "1".to_string()),
                        Entry::new(2, "2".to_string()),
                        Entry::new(3, "3".to_string()),
                    ],
                    Some(1),
                ))),
                Some(Node::Leaf(LeafNode::new(
                    vec![
                        Entry::new(7, "7".to_string()),
                        Entry::new(8, "8".to_string()),
                    ],
                    None,
                ))),
            ]
        );

        btree.insert(4, "4".to_string()).unwrap();

        let expected = vec![
            Some(Node::Inner(InnerNode::new(vec![
                NodePointer::new(1, 6),
                NodePointer::new(5, 5),
            ]))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(5, "5".to_string()),
                    Entry::new(6, "6".to_string()),
                ],
                Some(3),
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(1, "1".to_string()),
                    Entry::new(2, "2".to_string()),
                ],
                Some(4),
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(7, "7".to_string()),
                    Entry::new(8, "8".to_string()),
                ],
                None,
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(3, "3".to_string()),
                    Entry::new(4, "4".to_string()),
                ],
                Some(1),
            ))),
            Some(Node::Inner(InnerNode::new(vec![
                NodePointer::new(5, 1),
                NodePointer::new(7, 3),
            ]))),
            Some(Node::Inner(InnerNode::new(vec![
                NodePointer::new(1, 2),
                NodePointer::new(3, 4),
            ]))),
        ];

        let actual = nodes.lock().unwrap();

        println!("{:#?}", actual);
        println!("{:#?}", expected);

        assert_eq!(actual.as_ref(), expected);
    }

    #[test]
    fn test_update() {
        let nodes = vec![
            Some(Node::Inner(InnerNode::new(vec![
                NodePointer::new(1, 6),
                NodePointer::new(5, 5),
            ]))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(5, "5".to_string()),
                    Entry::new(6, "6".to_string()),
                ],
                Some(3),
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(1, "1".to_string()),
                    Entry::new(2, "2".to_string()),
                ],
                Some(4),
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(7, "7".to_string()),
                    Entry::new(8, "8".to_string()),
                ],
                None,
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(3, "3".to_string()),
                    Entry::new(4, "4".to_string()),
                ],
                Some(1),
            ))),
            Some(Node::Inner(InnerNode::new(vec![
                NodePointer::new(5, 1),
                NodePointer::new(7, 3),
            ]))),
            Some(Node::Inner(InnerNode::new(vec![
                NodePointer::new(1, 2),
                NodePointer::new(3, 4),
            ]))),
        ];

        let nodes = Arc::new(Mutex::new(nodes));

        let node_source = Box::new(VecNodeSource::new(nodes.clone()));

        let mut btree = BTree::new(3, node_source);

        let was_updated = btree.update(&3, "new value".to_string()).unwrap();

        assert!(was_updated);

        let expected = vec![
            Some(Node::Inner(InnerNode::new(vec![
                NodePointer::new(1, 6),
                NodePointer::new(5, 5),
            ]))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(5, "5".to_string()),
                    Entry::new(6, "6".to_string()),
                ],
                Some(3),
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(1, "1".to_string()),
                    Entry::new(2, "2".to_string()),
                ],
                Some(4),
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(7, "7".to_string()),
                    Entry::new(8, "8".to_string()),
                ],
                None,
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(3, "new value".to_string()),
                    Entry::new(4, "4".to_string()),
                ],
                Some(1),
            ))),
            Some(Node::Inner(InnerNode::new(vec![
                NodePointer::new(5, 1),
                NodePointer::new(7, 3),
            ]))),
            Some(Node::Inner(InnerNode::new(vec![
                NodePointer::new(1, 2),
                NodePointer::new(3, 4),
            ]))),
        ];

        assert_eq!(nodes.lock().unwrap().as_ref(), expected);

        let was_updated = btree.update(&12, "new value".to_string()).unwrap();

        assert!(!was_updated);

        let expected = vec![
            Some(Node::Inner(InnerNode::new(vec![
                NodePointer::new(1, 6),
                NodePointer::new(5, 5),
            ]))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(5, "5".to_string()),
                    Entry::new(6, "6".to_string()),
                ],
                Some(3),
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(1, "1".to_string()),
                    Entry::new(2, "2".to_string()),
                ],
                Some(4),
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(7, "7".to_string()),
                    Entry::new(8, "8".to_string()),
                ],
                None,
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(3, "new value".to_string()),
                    Entry::new(4, "4".to_string()),
                ],
                Some(1),
            ))),
            Some(Node::Inner(InnerNode::new(vec![
                NodePointer::new(5, 1),
                NodePointer::new(7, 3),
            ]))),
            Some(Node::Inner(InnerNode::new(vec![
                NodePointer::new(1, 2),
                NodePointer::new(3, 4),
            ]))),
        ];

        assert_eq!(nodes.lock().unwrap().as_ref(), expected);
    }
}
