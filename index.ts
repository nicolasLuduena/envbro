#!/usr/bin/env node
import { Command } from "commander";
import fs from "fs";
import path from "path";

const program = new Command();

if (!process.env.HOME) {
  throw new Error("HOME not set");
}

type RegisterInput = {
  env: string;
  project: string;
  /** Path of the file to store */
  target: string;
};

/**
 * If the environment name already exists it will show diff between both files
 * and ask the user whether to replace it for the new one or not
 */
const register = ({ env, project, target }: RegisterInput) => {
  if (env.includes("/")) {
    return console.error("Environment name can't contain /");
  }
  const envPath = path.join(ENV_STORE, project, env);
  const fileName = path.basename(target);

  if (!fs.existsSync(target) || !fs.statSync(target).isFile()) {
    return console.error("Make sure the file exists");
  }

  fs.mkdirSync(envPath, { recursive: true });
  const dir = fs.readdirSync(envPath);
  if (dir.length) {
    return console.warn(
      "This env already has contents. Feature to override in progress",
    );
  }

  const storedEnvPath = path.join(envPath, fileName);
  fs.copyFileSync(target, storedEnvPath);
  console.info("Stored your new env");
};

type SetInput = {
  env: string;
  project: string;
  override?: boolean;
};

/**
 * Sets the environment file for the project specified in the current working directory
 */
const set = ({ env, project, override: _override }: SetInput) => {
  const envFilePath = path.join(ENV_STORE, project, env);
  if (
    // checks conditions for the env file before reading them
    !fs.existsSync(envFilePath) ||
    !fs.statSync(envFilePath).isDirectory() ||
    !!fs.readdirSync(envFilePath).length
  ) {
    return console.warn("This environment doesn not exist for this project");
  }
  const storedEnvFile = path.join(envFilePath, fs.readdirSync(envFilePath)[0]);
  const envFileToSet = path.join(process.cwd(), storedEnvFile);

  // TODO: implement override
  if (fs.existsSync(envFileToSet)) {
    return console.warn(
      "Environment file is present already. Override not implemented",
    );
  }
  fs.copyFileSync(storedEnvFile, envFileToSet);
  console.info(`${env} env set my good friend`);
};

const ENV_STORE = path.join(process.env.HOME, ".envbro");

program
  .name("Env Friend")
  .description("Overcome your env files insecurities lil' bro");

program
  .command("set")
  .description("Sets the environment for a project")
  .argument("<string>", "Project name for whichset the environment will be set")
  .argument("<string>", "Environment to set for the project")
  .option(
    "-o, --override",
    "Override an environment file that's not registered",
  )
  .action((project, env, { override }) => {
    set({ env, project, override });
  });

program
  .command("register")
  .description("Adds a new environment to a project")
  .argument("<string>", "Project name")
  .argument("<string>", "Environment name")
  .requiredOption("--path <string>", "Path the environment file")
  .option("-o, --override", "Override the registered environment file")
  .action((project, env, { path: target }) =>
    register({ env, project, target }),
  );

program
  .command("rm")
  .description("Removes an environment from a project")
  .argument("<string>", "Project name")
  .argument("<string>", "Environment name to be deleted")
  .action(() => {});

program.parse();
