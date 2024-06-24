use std::collections::{BTreeMap, HashMap};

use essential_types::{intent::Intent, ContentAddress, Key, Value, Word};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct StateBuilder<KV = ()> {
    pub state: HashMap<ContentAddress, BTreeMap<Key, Value>>,
    pub intents: HashMap<String, ContentAddress>,
    pub kv: KV,
}

#[derive(Debug, Clone)]
pub struct Contract<K = ()> {
    pub address: ContentAddress,
    pub key: K,
}

pub type ContractKey = Contract<Key>;

#[derive(Debug, Clone)]
pub struct ContractIndex {
    pub address: ContentAddress,
    pub prefix: Key,
    pub index: Word,
}

impl StateBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<KV> StateBuilder<KV> {
    pub fn with_contract_address(
        mut self,
        name: impl Into<String>,
        address: ContentAddress,
    ) -> Self {
        if self.intents.insert(name.into(), address.clone()).is_none() {
            self.state.insert(address, BTreeMap::new());
        }
        self
    }

    pub fn with_contract(self, name: impl Into<String>, contract: &[Intent]) -> Self {
        let address = essential_hash::intent_set_addr::from_intents(contract);
        self.with_contract_address(name, address)
    }

    pub fn build(self) -> HashMap<ContentAddress, BTreeMap<Key, Value>> {
        self.state
    }
}

impl StateBuilder {
    pub fn push_scope(self, contract: ContentAddress) -> StateBuilder<Contract> {
        StateBuilder {
            state: self.state,
            intents: self.intents,
            kv: Contract {
                address: contract,
                key: (),
            },
        }
    }

    pub fn push(self, name: impl Into<String>) -> StateBuilder<Contract> {
        let address = self
            .intents
            .get(&name.into())
            .expect("Contract not found")
            .clone();
        self.push_scope(address)
    }
}

impl StateBuilder<Contract> {
    pub fn with_key(self, key: Key) -> StateBuilder<ContractKey> {
        StateBuilder {
            state: self.state,
            intents: self.intents,
            kv: ContractKey {
                address: self.kv.address,
                key,
            },
        }
    }

    pub fn with_index(self, index: Word) -> StateBuilder<ContractIndex> {
        StateBuilder {
            state: self.state,
            intents: self.intents,
            kv: ContractIndex {
                address: self.kv.address,
                prefix: vec![],
                index,
            },
        }
    }

    pub fn pop(self) -> StateBuilder {
        StateBuilder {
            state: self.state,
            intents: self.intents,
            kv: (),
        }
    }
}

impl StateBuilder<ContractIndex> {
    pub fn with_key(self, key: Key) -> StateBuilder<ContractKey> {
        let mut k = self.kv.prefix;
        k.push(self.kv.index);
        k.extend(key);
        StateBuilder {
            state: self.state,
            intents: self.intents,
            kv: ContractKey {
                address: self.kv.address,
                key: k,
            },
        }
    }

    pub fn with_int(self, value: Word) -> StateBuilder<Contract> {
        self.with_key(vec![]).with_int(value)
    }

    pub fn with_bool(self, value: bool) -> StateBuilder<Contract> {
        self.with_key(vec![]).with_bool(value)
    }

    pub fn with_b256(self, value: [Word; 4]) -> StateBuilder<Contract> {
        self.with_key(vec![]).with_b256(value)
    }

    pub fn with_map_int(self, key: Word) -> StateBuilder<ContractKey> {
        self.with_key(vec![key])
    }

    pub fn with_map_bool(self, key: bool) -> StateBuilder<ContractKey> {
        self.with_key(vec![key as Word])
    }

    pub fn with_map_b256(self, key: [Word; 4]) -> StateBuilder<ContractKey> {
        self.with_key(key.to_vec())
    }

    pub fn with_tuple(self, index: Word) -> StateBuilder<ContractIndex> {
        let mut prefix = self.kv.prefix;
        prefix.push(self.kv.index);
        StateBuilder {
            state: self.state,
            intents: self.intents,
            kv: ContractIndex {
                address: self.kv.address,
                prefix,
                index,
            },
        }
    }
}

impl StateBuilder<ContractKey> {
    pub fn with_value(mut self, value: Value) -> StateBuilder<Contract> {
        self.state
            .get_mut(&self.kv.address)
            .expect("Contract not found")
            .insert(self.kv.key, value);
        StateBuilder {
            state: self.state,
            intents: self.intents,
            kv: Contract {
                address: self.kv.address,
                key: (),
            },
        }
    }

    pub fn with_int(self, key: Word) -> StateBuilder<Contract> {
        self.with_value(vec![key])
    }

    pub fn with_bool(self, key: bool) -> StateBuilder<Contract> {
        self.with_value(vec![key as Word])
    }

    pub fn with_b256(self, key: [Word; 4]) -> StateBuilder<Contract> {
        self.with_value(key.to_vec())
    }

    pub fn with_map_int(mut self, key: Word) -> StateBuilder<ContractKey> {
        self.kv.key.push(key);
        self
    }

    pub fn with_map_bool(mut self, key: bool) -> StateBuilder<ContractKey> {
        self.kv.key.push(key as Word);
        self
    }

    pub fn with_map_b256(mut self, key: [Word; 4]) -> StateBuilder<ContractKey> {
        self.kv.key.extend(key);
        self
    }

    pub fn with_tuple(self, index: Word) -> StateBuilder<ContractIndex> {
        StateBuilder {
            state: self.state,
            intents: self.intents,
            kv: ContractIndex {
                address: self.kv.address,
                prefix: self.kv.key,
                index,
            },
        }
    }
}

impl Default for StateBuilder {
    fn default() -> Self {
        Self {
            state: Default::default(),
            intents: Default::default(),
            kv: (),
        }
    }
}
