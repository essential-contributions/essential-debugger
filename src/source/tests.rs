use super::*;

#[test]
fn test_show_code() {
    let other = r#"
const ::auth::TRANSFER: {contract: b256, addr: b256} = {contract: 0x9B7830F3C039B7A598371617827A5EF5854976F06CF8924586D9F1ED18411D75, addr: 0x8BD1FFCF63EBDBDC33B1D063FC1F95A256FF077750E03961BC60BFA0324D9340};                                                                   
const ::auth::signed::Cancel::ADDRESS: b256 = 0x8870814B1269BDFEECBE275E8B4DF39132A241869B64DA719CC5DE0778444CFE;                        
const ::auth::signed::Transfer::ADDRESS: b256 = 0x8BD1FFCF63EBDBDC33B1D063FC1F95A256FF077750E03961BC60BFA0324D9340;                      
const ::config::MINT_KEY: b256 = 0x1ECBB0067FC057261A6B199A0F121B5CD653B93DE9195CA5F836CE0D4D2A6E21;                                     
const ::auth::signed::Burn::ADDRESS: b256 = 0x106A3566D7F2C08F16AD8D8CF2CBC8523225713FF406643A3B726602BB05FB04;                          
const ::config::NAME: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;                                         
const ::auth::signed::ADDRESS: b256 = 0x9B7830F3C039B7A598371617827A5EF5854976F06CF8924586D9F1ED18411D75;                                
const ::config::SYMBOL: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;                                       
const ::auth::TRANSFER_WITH: {contract: b256, addr: b256} = {contract: 0x9B7830F3C039B7A598371617827A5EF5854976F06CF8924586D9F1ED18411D75, addr: 0x3750D1EE658C1A69072EC71B7C586C29779B4570DB1B19C054A58A9AD5803653};                                                              
const ::auth::BURN: {contract: b256, addr: b256} = {contract: 0x9B7830F3C039B7A598371617827A5EF5854976F06CF8924586D9F1ED18411D75, addr: 0x106A3566D7F2C08F16AD8D8CF2CBC8523225713FF406643A3B726602BB05FB04};                                                                       
const ::auth::MINT: {contract: b256, addr: b256} = {contract: 0x9B7830F3C039B7A598371617827A5EF5854976F06CF8924586D9F1ED18411D75, addr: 0x447A43D22870598BFFAFCC3C49B475BB29B631CA5253D804C37D1762648E1994};                                                                       
const ::auth::signed::Mint::ADDRESS: b256 = 0x447A43D22870598BFFAFCC3C49B475BB29B631CA5253D804C37D1762648E1994;                          
const ::auth::CANCEL: {contract: b256, addr: b256} = {contract: 0x9B7830F3C039B7A598371617827A5EF5854976F06CF8924586D9F1ED18411D75, addr: 0x8870814B1269BDFEECBE275E8B4DF39132A241869B64DA719CC5DE0778444CFE};                                                                     
const ::auth::signed::TransferWith::ADDRESS: b256 = 0x3750D1EE658C1A69072EC71B7C586C29779B4570DB1B19C054A58A9AD5803653; 
    "#;
    let predicate = r#"
predicate ::Transfer {
    storage {
        balances: ( b256 => int ),
        nonce: ( b256 => int ),
        token_name: b256,
        token_symbol: b256,
        decimals: int,
    }
    interface ::Auth {
        predicate Predicate {
            pub var addr: {contract: b256, addr: b256};
        }
    }
    interface ::AuthI = ::Auth(::auth_addr.contract)
    predicate ::A = ::AuthI::Predicate(::auth_addr.addr)
    pub var ::key: b256;
    pub var ::to: b256;
    pub var ::amount: int;
    var ::auth_addr.contract: b256;
    var ::auth_addr.addr: b256;
    var __::A_pathway: int;
    state ::sender_balance: int = storage::balances[::key];
    state ::receiver_balance: int = storage::balances[::to];
    state ::nonce: int = storage::nonce[::key];
    type ::std::lib::PredicateAddress = {contract: b256, addr: b256};
    type ::std::lib::Secp256k1Signature = {b256, b256, int};
    type ::std::lib::Secp256k1PublicKey = {b256, int};
    constraint (::amount > 0);
    constraint (::sender_balance' >= 0);
    constraint ((::sender_balance' - ::sender_balance) == (0 - ::amount));
    constraint (((__state_len(::receiver_balance) == 0) && (::receiver_balance' == ::amount)) || ((::receiver_balance' - ::receiver_balance) == ::amount));
    constraint (((__state_len(::nonce) == 0) && (::nonce' == 1)) || ((::nonce' - ::nonce) == 1));
    constraint (__mut_keys_len() == 3);
    constraint ((__sha256({::auth_addr.contract, ::auth_addr.addr}) == ::key) || (((0x9B7830F3C039B7A598371617827A5EF5854976F06CF8924586D9F1ED18411D75 == ::auth_addr.contract) && (0x8BD1FFCF63EBDBDC33B1D063FC1F95A256FF077750E03961BC60BFA0324D9340 == ::auth_addr.addr)) || ((0x9B7830F3C039B7A598371617827A5EF5854976F06CF8924586D9F1ED18411D75 == ::auth_addr.contract) && (0x3750D1EE658C1A69072EC71B7C586C29779B4570DB1B19C054A58A9AD5803653 == ::auth_addr.addr))));
    constraint ((::A::addr.contract == __this_set_address()) && (::A::addr.addr == __this_address()));
}
    "#;

    let source = Source::default()
        .with_other_code(other.to_string())
        .with_predicate_find_line(predicate.to_string(), 4);

    let out = show_code(&Some(source), ShowOutput::ConstraintOnly);
    assert_eq!(out, "constraint (((__state_len(::nonce) == 0) && (::nonce' == 1)) || ((::nonce' - ::nonce) == 1));")
}
