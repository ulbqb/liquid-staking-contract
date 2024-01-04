let cli_args = match_args(["network"]);

wasm::query(
    #{
        network: cli_args.network,
        contract_name: "portal",
        msg: #{ env: #{} }
    }
);
