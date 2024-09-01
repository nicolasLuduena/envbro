#!/usr/bin/env node
import { Command } from "commander";
import { register, rmEnv, set, list } from "./actions";

const program = new Command();

program.name("EnvBro").description("Drugs for your env files insecurities");

program
  .command("set")
  .description("Sets the environment for a project")
  .argument("<string>", "Project name for whichset the environment will be set")
  .argument("<string>", "Environment to set for the project")
  .action((project, env) => set({ env, project }));

program
  .command("register")
  .description("Adds a new environment to a project")
  .argument("<string>", "Project name")
  .argument("<string>", "Environment name")
  .requiredOption("--path <string>", "Path the environment file")
  .action((project, env, { path: target }) =>
    register({ env, project, target }),
  );

program
  .command("rm")
  .description("Removes an environment from a project")
  .argument("<string>", "Project name")
  .argument("<string>", "Environment name to be deleted")
  .action((project, env) => {
    rmEnv({ project, env });
  });

program
  .command("list")
  .description("Lists environments and filter by project")
  .option("-p, --project <string>", "Project to list envs for")
  .action((project) => list(project));

program.parse();
