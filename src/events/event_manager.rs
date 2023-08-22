use std::{collections::HashMap, hash::Hash, mem::replace};

pub struct EventManager<K: Eq + PartialEq + Hash, V> {
  pub listeners: HashMap<K, Vec<Box<dyn Fn(&V)>>>
}

impl<K: Eq + PartialEq + Hash, V> EventManager<K, V> {
  pub fn new() -> Self {
    EventManager { listeners: HashMap::new() }
  }

  pub fn on(&mut self, key: K, callback: Box<dyn Fn(&V)>) {
    let callbacks = self.listeners.get_mut(&key);
    if let Some(c) = callbacks {
      c.push(callback);
    }
    else {
      self.listeners.insert(key, vec![callback]);
    }
  }

  pub fn emit(&self, key: K, value: V) {
    let callbacks = self.listeners.get(&key);
    if let Some(c) = callbacks {
      for callback in c.iter() {
        (callback)(&value);
      }
    }
  }
}

pub struct EventManagerBathtub<K: Eq + PartialEq + Hash, V, R> {
  pub listeners: HashMap<K, Vec<Box<dyn Fn(&V) -> R>>>,
  pub bath: Vec<R>
}

impl<K: Eq + PartialEq + Hash, V, R> EventManagerBathtub<K, V, R> {
  pub fn new() -> Self {
    EventManagerBathtub { listeners: HashMap::new(), bath: Vec::new() }
  }

  pub fn on(&mut self, key: K, callback: Box<dyn Fn(&V) -> R>) {
    let callbacks = self.listeners.get_mut(&key);
    if let Some(c) = callbacks {
      c.push(callback);
    }
    else {
      self.listeners.insert(key, vec![callback]);
    }
  }

  pub fn emit(&mut self, key: K, value: V) {
    let callbacks = self.listeners.get(&key);
    if let Some(c) = callbacks {
      for callback in c.iter() {
        self.bath.push((callback)(&value));
      }
    }
  }

  pub fn drain(&mut self) -> Option<R> {
    self.bath.pop()
  }

  pub fn drain_all(&mut self) -> Vec<R> {
    replace(&mut self.bath, Vec::new())
  }
}
