import { execSync } from "child_process";
import fs from "fs";
import path from "path";
import readlineSync from "readline-sync";
import {
  ENV_STORE,
  ListInput,
  RegisterInput,
  RmInput,
  SetInput,
} from "./constants";

/**
 * If the environment name already exists it will show diff between both files
 * and ask the user whether to replace it for the new one or not */
export const register = ({ env, project, target }: RegisterInput) => {
  if (env.includes("/")) {
    return console.error("Environment name can not contain /");
  }
  const envPath = path.join(ENV_STORE, project, env);
  const fileName = path.basename(target);

  if (!fs.existsSync(target) || !fs.statSync(target).isFile()) {
    return console.error("File doesn't exist");
  }

  fs.mkdirSync(envPath, { recursive: true });
  const dir = fs.readdirSync(envPath);
  if (dir.length) {
    return console.warn("Env already exists. Override not implemented yet");
  }

  const storedEnvPath = path.join(envPath, fileName);
  fs.copyFileSync(target, storedEnvPath);
  console.info("New env stored");
};

/**
 * Sets the environment file for the project specified in the current working directory
 */
export const set = ({ env, project, override: _override }: SetInput) => {
  const envFilePath = path.join(ENV_STORE, project, env);
  if (
    // checks conditions for the env file before reading them
    !fs.existsSync(envFilePath) ||
    !fs.statSync(envFilePath).isDirectory() ||
    !fs.readdirSync(envFilePath).length
  ) {
    return console.warn("Env does not exist");
  }
  const storedEnvFile = path.join(envFilePath, fs.readdirSync(envFilePath)[0]);
  const envFileToSet = path.join(process.cwd(), path.basename(storedEnvFile));

  // TODO: implement override
  if (fs.existsSync(envFileToSet)) {
    return console.warn("Env is already set. Override not implemented yet.");
  }
  fs.copyFileSync(storedEnvFile, envFileToSet);
  console.info(`${env} set`);
};

/**
 * Remove the env from the store
 */
export const rmEnv = ({ env, project }: RmInput) => {
  const envFilePath = path.join(ENV_STORE, project, env);
  if (!fs.existsSync(envFilePath) || !fs.readdirSync(envFilePath).length) {
    return console.warn("Env does not exist");
  }
  const [file] = fs.readdirSync(envFilePath);

  const content = fs.readFileSync(path.join(envFilePath, file), {
    encoding: "utf8",
  });
  const prelimChoice = readlineSync.keyInYNStrict(
    `Are you sure you want to remove this?: \n${content}`,
  );
  if (prelimChoice) {
    const trueChoice = readlineSync.keyInYNStrict(
      "But... Are you really sure?",
    );
    trueChoice && fs.rmSync(envFilePath, { recursive: true, force: true });
    !fs.readdirSync(path.join(ENV_STORE, project)).length &&
      fs.rmdirSync(path.join(ENV_STORE, project));
  }
};

/**
 * List all envs available
 */
export const list = ({ project }: ListInput) => {
  const dirToShow = project ? path.join(ENV_STORE, project) : ENV_STORE;
  console.log(execSync("tree -a", { cwd: dirToShow, encoding: "utf8" }));
};
