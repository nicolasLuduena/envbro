# **EnvBro: Your New Best Friend for Environment Files**
==============================================

Tired of juggling multiple environment files? EnvBro is here to help! This CLI (Command-Line Interface) tool helps you manage environments with ease, making it perjfect for developers who work on projects with different configuration sets.
**Warning:** It's in beta. Some features are a work in progress yet


**Features**
------------

* **Register**: Add a new environment to a project using `envbro register <project_name> <environment> --path=<file_path>`.
* **Set**: Set the environment for a project by running `envbro set <project_name> <environment>`.
* **Rm**: Remove an environment from a project with `envbro rm <project_name> <environment>`.
* **List**: List all environments available for a project or globally using `envbro list [--project=<project_name>]`.

**Installation**
---------------

To get started, run the following command in your terminal:

```bash
npm install -g envbro
```

**Usage**
---------

### Register New Environment

Add a new environment to a project using:

```bash
envbro register <project_name> <environment> --path=<file_path>
```

Replace `<project_name>`, `<environment>`, and `<file_path>` with your actual values.

### Set Environment

Set the environment for a project by running:

```bash
envbro set <project_name> <environment>
```

Replace `<project_name>` and `<environment>` with your actual project name and environment.


### Remove Environment

Remove an environment from a project by running:

```bash
envbro rm <project_name> <environment>
```

Replace `<project_name>` and `<environment>` with your actual project name and environment to be removed.

### List Environments

List all environments available for a project or globally using:

```bash
envbro list [--project=<project_name>]
```

**Troubleshooting**
-----------------

If you encounter any issues, please refer to the [Issues](https://github.com/nicolasLuduena/envbro/issues) page on GitHub.

That's it! With EnvBro, managing your environment files has never been easier. Give it a try and see how much time you can save!
