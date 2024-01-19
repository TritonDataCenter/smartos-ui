# SmartOS UI

This repository is part of the Triton project. See the [contribution
guidelines](https://github.com/TritonDataCenter/triton/blob/master/CONTRIBUTING.md)
and general documentation at the main
[Triton project](https://github.com/TritonDataCenter/triton) page.

## Development

### Running in environments other than the Global Zone

Some mock data and scripts currently exists in the tests directory:

Set `PATH` to include the mock scripts and run the executor:

    PATH=test/mock:$PATH LOG_FILE=/tmp/ui.log ./target/debug/smartos_executor

and then run the UI:

    LOG_FILE=/tmp/ui.log ./target/debug/smartos_ui

Login via browser at: [localhost:8080](http://localhost:8080) using
user: admin and password: admin

### Running in the Global Zone

Copy `smartos_executor` and `smartos_ui` into the GZ and run:

    ./smartos_executor &
    UI_BIND_ADDRESS=0.0.0.0:8080 ./smartos_ui