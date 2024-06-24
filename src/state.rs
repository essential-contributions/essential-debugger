use std::{
    collections::{BTreeMap, HashMap},
    future::{self, Ready},
};

use essential_constraint_vm::{
    mut_keys_set, transient_data, Access, BytecodeMapped, SolutionAccess, StateSlots,
};
use essential_state_read_vm::{GasLimit, StateRead};
use essential_types::{
    intent::Intent,
    solution::{Solution, SolutionDataIndex},
    ContentAddress, Key, Value, Word,
};

pub struct Slots {
    pub pre: Vec<Value>,
    pub post: Vec<Value>,
}

struct State(HashMap<ContentAddress, BTreeMap<Key, Value>>);

pub async fn read_state(
    solution: &Solution,
    index: SolutionDataIndex,
    intent: &Intent,
    state: HashMap<ContentAddress, BTreeMap<Key, Value>>,
) -> anyhow::Result<Slots> {
    let pre_state = State(state.clone());
    // Apply mutations
    let mut post_state = State(state);
    post_state.apply_mutations(solution);

    let mut pre_slots: Vec<Vec<Word>> = Vec::new();
    let mut post_slots: Vec<Vec<Word>> = Vec::new();
    let mutable_keys = mut_keys_set(solution, index);
    let transient_data = transient_data(solution);
    for sr in &intent.state_read {
        let access = Access {
            solution: SolutionAccess::new(solution, index, &mutable_keys, &transient_data),
            state_slots: StateSlots {
                pre: &pre_slots,
                post: &post_slots,
            },
        };
        let mut vm = essential_state_read_vm::Vm::default();
        let bc: BytecodeMapped<essential_state_asm::Op, Vec<u8>> =
            BytecodeMapped::try_from_bytes(sr.clone())?;
        vm.exec_bytecode(
            &bc,
            access,
            &pre_state,
            &|_: &essential_state_asm::Op| 1,
            GasLimit::UNLIMITED,
        )
        .await?;

        pre_slots.extend(vm.into_state_slots());

        let access = Access {
            solution: SolutionAccess::new(solution, index, &mutable_keys, &transient_data),
            state_slots: StateSlots {
                pre: &pre_slots,
                post: &post_slots,
            },
        };
        let mut vm = essential_state_read_vm::Vm::default();
        let bc: BytecodeMapped<essential_state_asm::Op, Vec<u8>> =
            BytecodeMapped::try_from_bytes(sr.clone())?;
        vm.exec_bytecode(
            &bc,
            access,
            &post_state,
            &|_: &essential_state_asm::Op| 1,
            GasLimit::UNLIMITED,
        )
        .await?;

        post_slots.extend(vm.into_state_slots());
    }

    Ok(Slots {
        pre: pre_slots,
        post: post_slots,
    })
}

impl StateRead for State {
    type Error = anyhow::Error;
    type Future = Ready<Result<Vec<Vec<Word>>, Self::Error>>;
    fn key_range(&self, set_addr: ContentAddress, key: Key, num_words: usize) -> Self::Future {
        future::ready(self.key_range(set_addr, key, num_words))
    }
}

impl State {
    fn key_range(
        &self,
        set_addr: ContentAddress,
        mut key: Key,
        num_words: usize,
    ) -> anyhow::Result<Vec<Value>> {
        // Get the key that follows this one.
        fn next_key(mut key: Key) -> Option<Key> {
            for w in key.iter_mut().rev() {
                match *w {
                    Word::MAX => *w = Word::MIN,
                    _ => {
                        *w += 1;
                        return Some(key);
                    }
                }
            }
            None
        }

        let Some(set) = self.0.get(&set_addr) else {
            return Ok(vec![]);
        };

        // Collect the words.
        let mut words = vec![];
        for _ in 0..num_words {
            let opt = set.get(&key).cloned().unwrap_or_default();
            words.push(opt);
            key = next_key(key).ok_or(anyhow::anyhow!("Key error"))?;
        }
        Ok(words)
    }

    fn set(&mut self, set_addr: ContentAddress, key: &Key, value: Vec<Word>) {
        let set = self.0.entry(set_addr).or_default();
        if value.is_empty() {
            set.remove(key);
        } else {
            set.insert(key.clone(), value);
        }
    }

    fn apply_mutations(&mut self, solution: &Solution) {
        for data in &solution.data {
            for mutation in data.state_mutations.iter() {
                self.set(
                    data.intent_to_solve.set.clone(),
                    &mutation.key,
                    mutation.value.clone(),
                );
            }
        }
    }
}
