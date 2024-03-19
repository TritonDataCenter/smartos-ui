# SmartOS UI

This repository is part of the Triton project. See the [contribution
guidelines](https://github.com/TritonDataCenter/triton/blob/master/CONTRIBUTING.md)
and general documentation at the main
[Triton project](https://github.com/TritonDataCenter/triton) page.

## Development

### Running in environments other than the Global Zone

Some mock data and scripts currently exists in the `test` directory, running
`make devrun` will launch the executor, ui, and mock vminfod services.

Login at: [https://localhost:4443](http://localhost:4443) using user "root" and
password "root"

### Running in the Global Zone

Copy `smartos_executor` and `smartos_ui` into the GZ and run:

    ./smartos_executor &
    UI_BIND_HTTP_ADDRESS=0.0.0.0:8080 UI_BIND_HTTPS_ADDRESS=192.16.1.5:4443 ./smartos_ui
