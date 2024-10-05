# Intro

A CLI tool for building V1 apps.

# Features

## Dialogue

The CLI tool has a dialogue interface when no subcommands are provided, i.e. when the user runs `create-v1-app`.

## Subcommands

The CLI tool has two subcommands:

- `new`
- `add`

### `new`

The `new` subcommand creates a new V1 app.

#### Input

- `project_name` (required): The name of the new project.
- `--services, -s` (optional): A list of services to add to the project.
- `--package-manager, -pm` (optional): The package manager to use for the project, defaults to `npm`.

#### Example

```bash
create-v1-app new my-project --services=cal,dub
```

The above command creates a new V1 app with the name `my-project` and adds the `cal` and `dub` services to it.

### `add`

The `add` subcommand adds a service or provider to an existing V1 app.

#### Input

- `service` (required): The name of the service or provider to add.

#### Example

```bash
# inside a v1 project root directory
create-v1-app add service cal
```

The above command adds the `Cal.com` service to an existing V1 app.

## Help message

The CLI tool comes with help messages by using `create-v1-app --help`.

## Package managers

The CLI tool supports the following package managers:

- npm
- yarn
- pnpm
- bun
