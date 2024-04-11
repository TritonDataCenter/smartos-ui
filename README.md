<!--
    This Source Code Form is subject to the terms of the Mozilla Public
    License, v. 2.0. If a copy of the MPL was not distributed with this
    file, You can obtain one at http://mozilla.org/MPL/2.0/.
-->

<!--
    Copyright 2024 MNX Cloud, Inc.
-->

# SmartOS UI

This repository is part of the Triton project. See the [contribution
guidelines](https://github.com/TritonDataCenter/triton/blob/master/CONTRIBUTING.md)
and general documentation at the main
[Triton project](https://github.com/TritonDataCenter/triton) page.

## Overview

This project aims to provide a user-friendly web interface for a single SmartOS
installation.

### Installation

The UI can be selected when installing SmartOS or by using the `uiadm` utility.

### Accessing the interface

By default, the web interface will listen on port 4443 of the admin IP address.

## Development

The project consists of three workspaces:

- ui: Serves the web interface.
- executor: Executes programs such as [imgadm][imgadm] and [vmadm][vmadm] on
  behalf of the web interface.
- shared: Contains structs and functions used by both the ui and executor.

### Running in environments other than the Global Zone

Some mock data and scripts currently exists in the `test` directory, running
`make devrun` will launch the executor, ui, and mock vminfod services.

Login at: [https://localhost:4443](http://localhost:4443) using user "root" and
password "root"

[imgadm]: https://smartos.org/man/8/imgadm
[vmadm]: https://smartos.org/man/8/vmadm

