// # Matching command line arguments
// matches the arguments passed to the script and returns a map of the arguments
let cli_args = match_args(["signer", "network"]);

// use the shared_args library to create a base set of arguments
let shared_args = #{
    signer_account: cli_args.signer,
    network: cli_args.network,
};

// with this setup, we can run commands like:
// $ beaker task run deploy -- --signer test1 --network localnet

let cw20_contract = wasm::store_code(merge(
    shared_args,
    #{
        contract_name: "cw20_base",
        msg: #{}
    }
));

let cw721_contract = wasm::store_code(merge(
    shared_args,
    #{
        contract_name: "cw721_base",
        msg: #{}
    }
));

let delegator_contract = wasm::store_code(merge(
    shared_args,
    #{
        contract_name: "delegator",
        msg: #{}
    }
));

let portal_contract = wasm::store_code(merge(
    shared_args,
    #{
        contract_name: "portal",
        msg: #{}
    }
));

let portal_contract = wasm::instantiate(merge(
    merge(shared_args, #{
        no_proposal_sync: true,
    }),
    #{
        contract_name: "portal",
        msg: #{
            cw20_code_id: cw20_contract.code_id,
            cw721_code_id: cw721_contract.code_id,
            delegator_code_id: delegator_contract.code_id,
        }
    }
));
