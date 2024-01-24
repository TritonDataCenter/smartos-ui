# SmartOS UI

This repository is part of the Triton project. See the [contribution
guidelines](https://github.com/TritonDataCenter/triton/blob/master/CONTRIBUTING.md)
and general documentation at the main
[Triton project](https://github.com/TritonDataCenter/triton) page.

## Development

### Running in environments other than the Global Zone

Some mock data and scripts currently exists in the `test` directory, running
`make devrun` will launch the executor, ui, and mock vminfod services.

Login at: [localhost:8080](http://localhost:8080) using user "root" and password "root"

### Running in the Global Zone

Copy `smartos_executor` and `smartos_ui` into the GZ and run:

    ./smartos_executor &
    UI_BIND_ADDRESS=0.0.0.0:8080 ./smartos_ui

Login using user root and the password specified in `/etc/shadow`