use essential_constraint_vm as constraint_vm;
use essential_sign::secp256k1::{PublicKey, Secp256k1, SecretKey};
use essential_state_read_vm as state_read_vm;
use essential_types::{
    intent::{Directive, Intent},
    solution::{Mutation, Solution, SolutionData},
    ContentAddress, IntentAddress,
};

#[tokio::test]
async fn test_debugger() {
    let intent = Intent {
        // State read program to read state slot 0.
        state_read: vec![state_read_vm::asm::to_bytes([
            state_read_vm::asm::Stack::Push(1).into(),
            state_read_vm::asm::StateSlots::AllocSlots.into(),
            state_read_vm::asm::Stack::Push(0).into(),
            state_read_vm::asm::Stack::Push(0).into(),
            state_read_vm::asm::Stack::Push(0).into(),
            state_read_vm::asm::Stack::Push(0).into(),
            state_read_vm::asm::Stack::Push(4).into(),
            state_read_vm::asm::Stack::Push(1).into(),
            state_read_vm::asm::Stack::Push(0).into(),
            state_read_vm::asm::StateRead::KeyRange,
            state_read_vm::asm::ControlFlow::Halt.into(),
        ])
        .collect()],
        // Program to check pre-mutation value is None and
        // post-mutation value is 42 at slot 0.
        constraints: vec![constraint_vm::asm::to_bytes([
            constraint_vm::asm::Stack::Push(0).into(),
            constraint_vm::asm::Stack::Push(1).into(),
            constraint_vm::asm::Stack::Push(2).into(),
            constraint_vm::asm::Stack::Push(3).into(),
            constraint_vm::asm::Stack::Pop.into(),
            constraint_vm::asm::Stack::Pop.into(),
            constraint_vm::asm::Stack::Pop.into(),
            constraint_vm::asm::Stack::Pop.into(),
            constraint_vm::asm::Stack::Push(0).into(), // slot
            constraint_vm::asm::Stack::Push(0).into(), // pre
            constraint_vm::asm::Access::StateLen.into(),
            constraint_vm::asm::Stack::Push(0).into(),
            constraint_vm::asm::Pred::Eq.into(),
            constraint_vm::asm::Stack::Push(0).into(), // slot
            constraint_vm::asm::Stack::Push(1).into(), // post
            constraint_vm::asm::Access::State.into(),
            constraint_vm::asm::Stack::Push(42).into(),
            constraint_vm::asm::Pred::Eq.into(),
            constraint_vm::asm::Pred::And.into(),
        ])
        .collect()],
        directive: Directive::Satisfy,
    };

    let (sk, _pk) = random_keypair([0; 32]);
    let intents = essential_sign::intent_set::sign(vec![intent.clone()], &sk);
    let intent_addr = IntentAddress {
        set: essential_hash::intent_set_addr::from_intents(&intents.set),
        intent: ContentAddress(essential_hash::hash(&intents.set[0])),
    };

    // Construct the solution decision variables.
    // The first is an inline variable 42.
    let decision_variables = vec![vec![42]];

    // Create the solution.
    let solution = Solution {
        data: vec![SolutionData {
            intent_to_solve: intent_addr,
            decision_variables,
            state_mutations: vec![Mutation {
                key: vec![0, 0, 0, 0],
                value: vec![42],
            }],
            transient_data: vec![],
        }],
    };

    essential_debugger::run(solution, 0, intent, 0, Default::default())
        .await
        .unwrap();
}

pub fn random_keypair(seed: [u8; 32]) -> (SecretKey, PublicKey) {
    use rand::SeedableRng;
    let mut rng = rand::rngs::SmallRng::from_seed(seed);
    let secp = Secp256k1::new();
    secp.generate_keypair(&mut rng)
}
