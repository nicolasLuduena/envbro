import path from "path";
if (!process.env.HOME) {
  throw new Error("HOME not set");
}

export const ENV_STORE = path.join(process.env.HOME, ".envbro");

export type RegisterInput = {
  env: string;
  project: string;
  /** Path of the file to store */
  target: string;
};

export type SetInput = {
  env: string;
  project: string;
  override?: boolean;
};

export type RmInput = {
  env: string;
  project: string;
};

export type ListInput = {
  project?: string;
};
