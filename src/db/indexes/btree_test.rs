use std::sync::{Arc, Mutex};

use super::btree::*;
use super::node::*;

use crate::db::Result;

type NodeVec<K, V> = Vec<Option<Node<K, V>>>;

pub struct VecNodeSource<K, V> {
    nodes: Arc<Mutex<NodeVec<K, V>>>,
}

impl<K, V> VecNodeSource<K, V>
where
    K: BTreeKey,
    V: BTreeValue,
{
    pub fn new(nodes: Arc<Mutex<NodeVec<K, V>>>) -> Self {
        Self { nodes }
    }
}

impl<K, V> NodeSource<K, V> for VecNodeSource<K, V>
where
    K: BTreeKey,
    V: BTreeValue,
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

struct TestCase<K, V> {
    nodes: Arc<Mutex<NodeVec<K, V>>>,
    btree: BTree<K, V>,
}

impl<K, V> TestCase<K, V>
where
    K: BTreeKey,
    V: BTreeValue + PartialEq,
{
    pub fn new(order: usize, initial_nodes: NodeVec<K, V>) -> Self {
        let nodes = Arc::new(Mutex::new(initial_nodes));
        Self {
            nodes: nodes.clone(),
            btree: BTree::new(order, Box::new(VecNodeSource::new(nodes))),
        }
    }

    pub fn assert_nodes_eq(&self, expected: NodeVec<K, V>) {
        assert_eq!(self.nodes.lock().unwrap().as_ref(), expected);
    }
}

type StdTestCase = TestCase<usize, String>;

mod get {
    use super::*;

    #[test]
    fn empty() {
        // Empty
        let tc = StdTestCase::new(3, vec![Some(Node::Leaf(LeafNode::new(vec![], None)))]);
        assert_eq!(tc.btree.get(&1), Ok(None));
    }

    #[test]
    fn three_entries() {
        let tc = StdTestCase::new(
            3,
            vec![Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(1, "foo".to_string()),
                    Entry::new(2, "bar".to_string()),
                    Entry::new(3, "baz".to_string()),
                ],
                None,
            )))],
        );

        assert_eq!(tc.btree.get(&1), Ok(Some("foo".to_string())));
        assert_eq!(tc.btree.get(&2), Ok(Some("bar".to_string())));
        assert_eq!(tc.btree.get(&3), Ok(Some("baz".to_string())));
    }

    #[test]
    fn nested() {
        let tc = StdTestCase::new(
            3,
            vec![
                Some(Node::Inner(InnerNode::new(vec![1, 3, 2], vec![3, 10]))),
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
            ],
        );

        assert_eq!(tc.btree.get(&1), Ok(Some("foo".to_string())));
        assert_eq!(tc.btree.get(&2), Ok(Some("bar".to_string())));
        assert_eq!(tc.btree.get(&3), Ok(Some("foo3".to_string())));
        assert_eq!(tc.btree.get(&4), Ok(None));
        assert_eq!(tc.btree.get(&5), Ok(Some("bar5".to_string())));
        assert_eq!(tc.btree.get(&10), Ok(Some("foo10".to_string())));
        assert_eq!(tc.btree.get(&11), Ok(Some("bar11".to_string())));
    }
}

mod insert {
    use super::*;

    #[test]
    fn insert() {
        let mut tc = StdTestCase::new(3, vec![Some(Node::Leaf(LeafNode::new(vec![], None)))]);

        tc.btree.insert(5, "5".to_string()).unwrap();
        tc.assert_nodes_eq(vec![Some(Node::Leaf(LeafNode::new(
            vec![Entry::new(5, "5".to_string())],
            None,
        )))]);

        tc.btree.insert(7, "7".to_string()).unwrap();
        tc.assert_nodes_eq(vec![Some(Node::Leaf(LeafNode::new(
            vec![
                Entry::new(5, "5".to_string()),
                Entry::new(7, "7".to_string()),
            ],
            None,
        )))]);

        tc.btree.insert(3, "3".to_string()).unwrap();
        tc.assert_nodes_eq(vec![Some(Node::Leaf(LeafNode::new(
            vec![
                Entry::new(3, "3".to_string()),
                Entry::new(5, "5".to_string()),
                Entry::new(7, "7".to_string()),
            ],
            None,
        )))]);

        tc.btree.insert(1, "1".to_string()).unwrap();
        tc.assert_nodes_eq(vec![
            Some(Node::Inner(InnerNode::new(vec![2, 1], vec![5]))),
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
        ]);

        tc.btree.insert(6, "6".to_string()).unwrap();
        tc.assert_nodes_eq(vec![
            Some(Node::Inner(InnerNode::new(vec![2, 1], vec![5]))),
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
        ]);

        tc.btree.insert(8, "8".to_string()).unwrap();
        tc.assert_nodes_eq(vec![
            Some(Node::Inner(InnerNode::new(vec![2, 1, 3], vec![5, 7]))),
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
        ]);

        tc.btree.insert(2, "2".to_string()).unwrap();
        tc.assert_nodes_eq(vec![
            Some(Node::Inner(InnerNode::new(vec![2, 1, 3], vec![5, 7]))),
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
        ]);

        tc.btree.insert(4, "4".to_string()).unwrap();
        tc.assert_nodes_eq(vec![
            Some(Node::Inner(InnerNode::new(vec![6, 5], vec![5]))),
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
            Some(Node::Inner(InnerNode::new(vec![1, 3], vec![7]))),
            Some(Node::Inner(InnerNode::new(vec![2, 4], vec![3]))),
        ]);
    }
}

mod update {
    use super::*;

    #[test]
    fn empty() {
        // Empty
        let mut tc = StdTestCase::new(3, vec![Some(Node::Leaf(LeafNode::new(vec![], None)))]);
        assert_eq!(tc.btree.update(&1, "foo".to_string()), Ok(false));
    }

    #[test]
    fn three_entries() {
        let mut tc = StdTestCase::new(
            3,
            vec![Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(1, "foo".to_string()),
                    Entry::new(2, "bar".to_string()),
                    Entry::new(3, "baz".to_string()),
                ],
                None,
            )))],
        );

        assert_eq!(tc.btree.update(&1, "new foo".to_string()), Ok(true));
        assert_eq!(tc.btree.update(&2, "new bar".to_string()), Ok(true));
        assert_eq!(tc.btree.update(&3, "new baz".to_string()), Ok(true));
        assert_eq!(tc.btree.update(&4, "new baz".to_string()), Ok(false));

        tc.assert_nodes_eq(vec![Some(Node::Leaf(LeafNode::new(
            vec![
                Entry::new(1, "new foo".to_string()),
                Entry::new(2, "new bar".to_string()),
                Entry::new(3, "new baz".to_string()),
            ],
            None,
        )))]);
    }

    #[test]
    fn nested() {
        let mut tc = StdTestCase::new(
            3,
            vec![
                Some(Node::Inner(InnerNode::new(vec![1, 3, 2], vec![3, 10]))),
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
            ],
        );

        assert_eq!(tc.btree.update(&1, "new foo".to_string()), Ok(true));
        assert_eq!(tc.btree.update(&2, "new bar".to_string()), Ok(true));
        assert_eq!(tc.btree.update(&3, "new foo3".to_string()), Ok(true));
        assert_eq!(tc.btree.update(&4, "new 4".to_string()), Ok(false));
        assert_eq!(tc.btree.update(&5, "new bar5".to_string()), Ok(true));
        assert_eq!(tc.btree.update(&10, "new foo10".to_string()), Ok(true));
        assert_eq!(tc.btree.update(&11, "new bar11".to_string()), Ok(true));

        tc.assert_nodes_eq(vec![
            Some(Node::Inner(InnerNode::new(vec![1, 3, 2], vec![3, 10]))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(1, "new foo".to_string()),
                    Entry::new(2, "new bar".to_string()),
                ],
                Some(3),
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(10, "new foo10".to_string()),
                    Entry::new(11, "new bar11".to_string()),
                ],
                None,
            ))),
            Some(Node::Leaf(LeafNode::new(
                vec![
                    Entry::new(3, "new foo3".to_string()),
                    Entry::new(5, "new bar5".to_string()),
                ],
                Some(2),
            ))),
        ]);
    }
}

// mod delete {
//     use super::*;
//
//     #[test]
//     fn empty_tree() {
//         let mut tc = StdTestCase::new(3, vec![Some(Node::Leaf(LeafNode::new(vec![], None)))]);
//
//         let exists = tc.btree.delete(&3).unwrap();
//         assert!(!exists);
//     }
//
//     #[test]
//     fn single_item() {
//         let mut tc = StdTestCase::new(
//             3,
//             vec![Some(Node::Leaf(LeafNode::new(
//                 vec![Entry::new(3, "3".to_string())],
//                 None,
//             )))],
//         );
//
//         let exists = tc.btree.delete(&3).unwrap();
//         assert!(exists);
//         tc.assert_nodes_eq(vec![Some(Node::Leaf(LeafNode::new(vec![], None)))]);
//     }
//
//     #[test]
//     fn full_leaf_first() {
//         let mut tc = StdTestCase::new(
//             3,
//             vec![Some(Node::Leaf(LeafNode::new(
//                 vec![
//                     Entry::new(3, "3".to_string()),
//                     Entry::new(5, "5".to_string()),
//                     Entry::new(7, "7".to_string()),
//                 ],
//                 None,
//             )))],
//         );
//
//         let exists = tc.btree.delete(&3).unwrap();
//         assert!(exists);
//         tc.assert_nodes_eq(vec![Some(Node::Leaf(LeafNode::new(
//             vec![
//                 Entry::new(5, "5".to_string()),
//                 Entry::new(7, "7".to_string()),
//             ],
//             None,
//         )))]);
//     }
//
//     #[test]
//     fn full_leaf_last() {
//         let mut tc = StdTestCase::new(
//             3,
//             vec![Some(Node::Leaf(LeafNode::new(
//                 vec![
//                     Entry::new(3, "3".to_string()),
//                     Entry::new(5, "5".to_string()),
//                     Entry::new(7, "7".to_string()),
//                 ],
//                 None,
//             )))],
//         );
//
//         let exists = tc.btree.delete(&7).unwrap();
//         assert!(exists);
//         tc.assert_nodes_eq(vec![Some(Node::Leaf(LeafNode::new(
//             vec![
//                 Entry::new(3, "3".to_string()),
//                 Entry::new(5, "5".to_string()),
//             ],
//             None,
//         )))]);
//     }
//
//     #[test]
//     fn full_leaf_middle() {
//         let mut tc = StdTestCase::new(
//             3,
//             vec![Some(Node::Leaf(LeafNode::new(
//                 vec![
//                     Entry::new(3, "3".to_string()),
//                     Entry::new(5, "5".to_string()),
//                     Entry::new(7, "7".to_string()),
//                 ],
//                 None,
//             )))],
//         );
//
//         let exists = tc.btree.delete(&5).unwrap();
//         assert!(exists);
//         tc.assert_nodes_eq(vec![Some(Node::Leaf(LeafNode::new(
//             vec![
//                 Entry::new(3, "3".to_string()),
//                 Entry::new(7, "7".to_string()),
//             ],
//             None,
//         )))]);
//     }
//
//     #[test]
//     fn nested_full_leaf() {
//         let mut tc = StdTestCase::new(
//             3,
//             vec![
//                 Some(Node::Inner(InnerNode::new(vec![
//                     NodePointer::new(1, 1),
//                     NodePointer::new(4, 2),
//                 ]))),
//                 Some(Node::Leaf(LeafNode::new(
//                     vec![
//                         Entry::new(1, "1".to_string()),
//                         Entry::new(2, "2".to_string()),
//                         Entry::new(3, "3".to_string()),
//                     ],
//                     Some(2),
//                 ))),
//                 Some(Node::Leaf(LeafNode::new(
//                     vec![
//                         Entry::new(4, "4".to_string()),
//                         Entry::new(5, "5".to_string()),
//                         Entry::new(6, "6".to_string()),
//                     ],
//                     None,
//                 ))),
//             ],
//         );
//
//         let was_deleted = tc.btree.delete(&5).unwrap();
//         assert!(was_deleted);
//         tc.assert_nodes_eq(vec![
//             Some(Node::Inner(InnerNode::new(vec![
//                 NodePointer::new(1, 1),
//                 NodePointer::new(4, 2),
//             ]))),
//             Some(Node::Leaf(LeafNode::new(
//                 vec![
//                     Entry::new(1, "1".to_string()),
//                     Entry::new(2, "2".to_string()),
//                     Entry::new(3, "3".to_string()),
//                 ],
//                 Some(2),
//             ))),
//             Some(Node::Leaf(LeafNode::new(
//                 vec![
//                     Entry::new(4, "4".to_string()),
//                     Entry::new(6, "6".to_string()),
//                 ],
//                 None,
//             ))),
//         ]);
//     }
//
//     #[test]
//     fn nested_leaf_first() {
//         let mut tc = StdTestCase::new(
//             3,
//             vec![
//                 Some(Node::Inner(InnerNode::new(vec![
//                     NodePointer::new(1, 1),
//                     NodePointer::new(4, 2),
//                 ]))),
//                 Some(Node::Leaf(LeafNode::new(
//                     vec![
//                         Entry::new(1, "1".to_string()),
//                         Entry::new(2, "2".to_string()),
//                         Entry::new(3, "3".to_string()),
//                     ],
//                     Some(2),
//                 ))),
//                 Some(Node::Leaf(LeafNode::new(
//                     vec![
//                         Entry::new(4, "4".to_string()),
//                         Entry::new(5, "5".to_string()),
//                         Entry::new(6, "6".to_string()),
//                     ],
//                     None,
//                 ))),
//             ],
//         );
//
//         let was_deleted = tc.btree.delete(&4).unwrap();
//         assert!(was_deleted);
//         tc.assert_nodes_eq(vec![
//             Some(Node::Inner(InnerNode::new(vec![
//                 NodePointer::new(1, 1),
//                 NodePointer::new(5, 2),
//             ]))),
//             Some(Node::Leaf(LeafNode::new(
//                 vec![
//                     Entry::new(1, "1".to_string()),
//                     Entry::new(2, "2".to_string()),
//                     Entry::new(3, "3".to_string()),
//                 ],
//                 Some(2),
//             ))),
//             Some(Node::Leaf(LeafNode::new(
//                 vec![
//                     Entry::new(5, "5".to_string()),
//                     Entry::new(6, "6".to_string()),
//                 ],
//                 None,
//             ))),
//         ]);
//     }
// }
