import fs from "fs";
import path from "path";
import { ENV_STORE, RegisterInput, RmInput, SetInput } from "./constants";

/**
 * If the environment name already exists it will show diff between both files
 * and ask the user whether to replace it for the new one or not
 */
export const register = ({ env, project, target }: RegisterInput) => {
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

/**
 * Sets the environment file for the project specified in the current working directory
 */
export const set = ({ env, project, override: _override }: SetInput) => {
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

/**
 * Remove the env from the store
 */
export const rmEnv = ({ env, project }: RmInput) => {
  console.log(env, project);
};
