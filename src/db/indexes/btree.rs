use crate::db::Result;
use crate::db::indexes::node::*;
use std::fmt::Debug;

pub struct BTree<K, V> {
    order: usize,
    nodes: Box<dyn NodeSource<K, V>>,
}

pub trait BTreeKey: Clone + Ord + Debug + 'static {}
impl<T> BTreeKey for T where T: Clone + Ord + Debug + 'static {}

pub trait BTreeValue: Clone + Debug + 'static {}
impl<T> BTreeValue for T where T: Clone + Debug + 'static {}

impl<K, V> BTree<K, V>
where
    K: BTreeKey,
    V: BTreeValue,
{
    pub fn new(order: usize, nodes: Box<dyn NodeSource<K, V>>) -> Self {
        Self { order, nodes }
    }

    pub fn get(&self, key: &K) -> Result<Option<V>> {
        self.get_from_node(key, &self.nodes.get_node(0)?)
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
        let child_index = self.find_child_index(key, inner);
        self.get_from_node(key, &self.nodes.get_node(inner.pointers[child_index])?)
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

    fn insert_into_node(
        &mut self,
        node: &mut Node<K, V>,
        entry: Entry<K, V>,
    ) -> Result<Option<(K, usize)>> {
        match node {
            Node::Leaf(leaf) => self.insert_into_leaf(entry, leaf),
            Node::Inner(inner) => self.insert_into_inner(entry, inner),
        }
    }

    fn insert_into_leaf(
        &mut self,
        entry: Entry<K, V>,
        leaf: &mut LeafNode<K, V>,
    ) -> Result<Option<(K, usize)>> {
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
    ) -> Result<Option<(K, usize)>> {
        let child_index = self.find_child_index(&entry.key, inner);
        let child_pointer = inner.pointers[child_index];
        let mut child = self.nodes.get_node(child_pointer)?;
        let new_right = self.insert_into_node(&mut child, entry)?;
        self.nodes.update_node(child_pointer, &child)?;
        if let Some((new_right_key, new_right_index)) = new_right {
            inner.keys.insert(child_index, new_right_key);
            inner.pointers.insert(child_index + 1, new_right_index);
            if inner.pointers.len() > self.order {
                return Ok(Some(self.split_inner(inner)?));
            }
        }
        Ok(None)
    }

    pub fn update(&mut self, key: &K, value: V) -> Result<bool> {
        self.update_in_node(0, key, value)
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
        let child_index = self.find_child_index(k, inner);
        self.update_in_node(inner.pointers[child_index], k, v)
    }

    // pub fn delete(&mut self, key: &K) -> Result<bool> {
    //     self.delete_from_node(0, key).map(|res| res.is_some())
    // }
    //
    // fn delete_from_node(&mut self, pointer: usize, key: &K) -> Result<Option<Node<K, V>>> {
    //     let mut node = self.nodes.get_node(pointer)?;
    //     match &mut node {
    //         Node::Leaf(leaf) => {
    //             let was_deleted = self.delete_from_leaf(key, leaf);
    //             if was_deleted {
    //                 self.nodes.update_node(pointer, &node)?;
    //                 Ok(Some(node))
    //             } else {
    //                 Ok(None)
    //             }
    //         }
    //         Node::Inner(inner) => {
    //             let child_index = self.find_child_index(key, inner);
    //             let child = self.delete_from_node(inner.pointers[child_index], key)?;
    //             if let Some(child) = child {
    //                 if let Node::Leaf(child_leaf) = child
    //                     && child_leaf.entries[0].key != inner.pointers[child_index].key
    //                 {
    //                     inner.pointers[child_index].key = child_leaf.entries[0].key.clone();
    //                     self.nodes.update_node(pointer, &node)?;
    //                 }
    //                 Ok(Some(node))
    //             } else {
    //                 Ok(None)
    //             }
    //         }
    //     }
    // }
    //
    // fn delete_from_leaf(&mut self, key: &K, leaf: &mut LeafNode<K, V>) -> bool {
    //     let entry_index = self.find_entry_index(key, leaf);
    //     if let Some(entry_index) = entry_index {
    //         leaf.entries.remove(entry_index);
    //         true
    //     } else {
    //         false
    //     }
    // }

    fn split_leaf(&mut self, leaf: &mut LeafNode<K, V>) -> Result<(K, usize)> {
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

        Ok((right_key, right_index))
    }

    fn split_inner(&mut self, inner: &mut InnerNode<K>) -> Result<(K, usize)> {
        let split_index = self.order / 2;
        let split_key = inner.keys.remove(split_index);

        let mut right = InnerNode::new(vec![], vec![]);
        for i in split_index..inner.keys.len() {
            right.keys.push(inner.keys[i].clone());
            right.pointers.push(inner.pointers[i + 1]);
        }
        right.pointers.push(*inner.pointers.last().unwrap());

        let right_index = self.nodes.create_node(Node::Inner(right))?;

        inner.pointers.truncate(split_index + 1);
        inner.keys.truncate(split_index);

        Ok((split_key, right_index))
    }

    fn split_root(
        &mut self,
        root: &mut Node<K, V>,
        (split_key, new_right_pointer): (K, usize),
    ) -> Result<()> {
        let new_left_pointer = self.nodes.create_node(root.clone())?;
        *root = Node::Inner(InnerNode::new(
            vec![new_left_pointer, new_right_pointer],
            vec![split_key],
        ));
        Ok(())
    }

    fn find_child_index(&self, key: &K, inner: &InnerNode<K>) -> usize {
        for i in 0..inner.keys.len() {
            if &inner.keys[i] > key {
                return i;
            }
        }
        inner.pointers.len() - 1
    }

    fn find_insertion_index(&self, key: &K, leaf: &LeafNode<K, V>) -> usize {
        for (i, entry) in leaf.entries.iter().enumerate() {
            if &entry.key >= key {
                return i;
            }
        }
        leaf.entries.len()
    }
}
