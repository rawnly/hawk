# GH Workflows watcher

Dead simple rust CLI to ease workflows management inside monorepos.

## Example

> Check out the [example](./example) folder.

Below an example monorepo situation:

```
example
├── hawk-config.yaml
├── .github
│   └── workflows
│       ├── my-second-app--deploy.yml
│       └── smartfish--deploy.yml
└── packages
    ├── my-app
    │   ├── .DS_Store
    │   ├── package.json // reads workspace name from package.json
    │   └── workflows
    │       └── deploy.yml
    └── my-second-app
        └── workflows
            └── deploy.yml
```

```bash
    $ cd example
    $ hawk -w
    ... let the magic happen
```

## Why

Github actions won't let you store workflows files inside subfolders, neither in your `.github/workflows/` folder or project custom folders.
To solve that I made `hawk`. It lets you copy workflows from custom paths and paste them with a prefix, handling most of the pain.
With 10 lines config you have a working monorepo setup.

## Installation

Until release installation is only available from source code. Make sure to have your rust environment ready, then:

- Clone the repo
- Run `cargo build -r` or `make build`
- Copy `target/release/hawk` to your path or use `sudo make install` (it will copy the bin into `/usr/local/bin`)
- Enjoy

## Features

- [x] File watching
- [x] Cleanup `workflows` folder from generated files.
- [x] Custom configuration
- [ ] Generate config from `pnpm-workspace.yaml` and yarns `package.json/workspaces`
- [ ] Create an action to automate this process. (so the user can update a workflow, push and get the generated one updated automatically)
