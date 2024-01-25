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
    UI_BIND_ADDRESS=0.0.0.0:8080 SKIP_PRIVILEGE_DROP=1 ./smartos_ui

Login using user "root" and the password specified in `/etc/shadow`

If you omit setting the `SKIP_PRIVILEGE_DROP` environment variable on illumos,
the UI will drop any [privileges](https://illumos.org/man/7/privileges) that
are not needed, change to the user `nobody`, and chroot into `/opt/smartos_ui`
(or a path set in the `CHROOT` env var) where you will need a few extra things
setup:

#### resolv.conf

A `/etc/resolv.conf` is needed in the chroot to resolve imgapi server names when
importing images. This can be copied from the GZ.

#### TLS certs

The `SSL_CERT_DIR`environment variable needs to be set to a directory of TLS
root certs within the chroot. If pkgsrc is installed the certs can be obtained
from `/opt/tools/etc/openssl/certs/`

Note: This should not be necessary when built as a Pkgsrc package.

#### cache directory

The environment variable `CACHE_DIR` must be set to a writable directory within
the chroot.