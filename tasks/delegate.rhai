// # Matching command line arguments
// matches the arguments passed to the script and returns a map of the arguments
let cli_args = match_args(["signer", "network", "funds", "validator"]);

// with this setup, we can run commands like:
// $ beaker task run delegate -- --signer test1 --network localnet --funds 1000cony --validator linkvaloper1twsfmuj28ndph54k4nw8crwu8h9c8mh33lyrp8

let portal_contract = wasm::execute(
    #{
        signer_account: cli_args.signer,
        network: cli_args.network,
        funds: cli_args.funds,
        contract_name: "portal",
        msg: #{
            delegate_and_tokenize: #{
                validator: cli_args.validator
            }
        }
    }
);
