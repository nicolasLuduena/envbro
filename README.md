# **EnvBro: Your New Best Friend for Environment Files**

Tired of juggling multiple environment files? EnvBro is here to help! This CLI (Command-Line Interface) tool helps you manage environments with ease, making it perfect for developers who work on projects with different configuration sets.
**Warning:** It's in beta. Some features are a work in progress yet.

**Features**
------------

* **Register**: Add a new environment to a project using `envbro register`. Supports local files or remote P2P tickets.
* **Set**: Set the environment for a project by running `envbro set`.
* **Rm**: Remove an environment from a project with `envbro rm`.
* **List**: List all environments available for a project or globally using `envbro list`.
* **Show**: View the contents of a stored environment file.
* **Share**: Share an environment peer-to-peer securely.

**Installation**
---------------

To get started, you can install `envbro` using Cargo:

```bash
cargo install envbro
```

**Usage**
---------

### Register New Environment

Add a new environment to a project. You can register a local file or download one from a remote peer using a ticket.

**From local file:**
```bash
envbro register <project_name> <environment> --path=<file_path>
```

**From remote ticket:**
```bash
envbro register <project_name> <environment> --ticket=<iroh_ticket> [--filename=<target_filename>]
```

Replace `<project_name>`, `<environment>`, `<file_path>`, and `<iroh_ticket>` with your actual values.

### Set Environment

Set the environment for a project by running:

```bash
envbro set <project_name> <environment>
```

This decrypts the stored file and writes it into the current directory under the filename it was registered with. If a different file with that name is already there, it shows a diff and asks for confirmation first (skip with `--force`).

### Remove Environment

Remove an environment from a project by running:

```bash
envbro rm <project_name> <environment>
```

### List Environments

List all environments available for a project or globally using:

```bash
envbro list [--project=<project_name>]
```

### Show Environment

View the contents of an environment file:

```bash
envbro show <project_name> <environment>
```

### Share Environment

Share an environment with another peer:

```bash
envbro share <project_name> <environment>
```
This will generate a ticket that you can share with another user to let them `register` this environment securely.

The blob is transferred still encrypted, so the recipient needs the passphrase the environment was registered with to `set` or `show` it.

### Passphrase

Each environment is encrypted with the passphrase given when it was registered and nothing checks that passphrases match across environments, so use one passphrase per vault.

### Environment Variables

* `ENVBRO_PASSPHRASE`: use this passphrase instead of prompting for one.
* `ENVBRO_ROOT`: keep the vault in this directory instead of `~/.envbro`.

**Acknowledgments**
-----------------

EnvBro is powered by [Iroh](https://iroh.computer), which provides the robust peer-to-peer networking capabilities that make secure environment sharing possible.

**Troubleshooting**
-----------------

If you encounter any issues, please refer to the [Issues](https://github.com/nicolasLuduena/envbro/issues) page on GitHub.

That's it! With EnvBro, managing your environment files has never been easier. Give it a try and see how much time you can save!
