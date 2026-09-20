use crate::db::Result;

#[derive(Debug, Clone, PartialEq)]
pub struct NodePointer<K> {
    pub key: K,
    pub pointer: usize,
}

impl<K> NodePointer<K> {
    pub fn new(key: K, pointer: usize) -> Self {
        Self { key, pointer }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry<K, V> {
    pub key: K,
    pub value: V,
}

impl<K, V> Entry<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InnerNode<K> {
    pub pointers: Vec<NodePointer<K>>,
}

impl<K> InnerNode<K> {
    pub fn new(pointers: Vec<NodePointer<K>>) -> Self {
        Self { pointers }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeafNode<K, V> {
    pub entries: Vec<Entry<K, V>>,
    pub next: Option<usize>,
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
    fn get_node(&self, pointer: usize) -> Result<Node<K, V>>;
    fn create_node(&mut self, node: Node<K, V>) -> Result<usize>;
    fn update_node(&mut self, pointer: usize, node: &Node<K, V>) -> Result<()>;
    fn delete_node(&mut self, pointer: usize) -> Result<bool>;
}
