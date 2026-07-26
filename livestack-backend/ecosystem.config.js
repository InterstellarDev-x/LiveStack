// pm2 process definitions for the production host.
//
// This file is the single source of truth for what should be running: the
// deploy runs `pm2 startOrReload` against it, which starts anything missing
// instead of only reloading what happens to already be there. The previous
// deploy reloaded each app by name and swallowed failures, so a process that
// had never been started (email-worker) stayed silently absent — and with it,
// every email alert.
//
// Each Rust binary reads its configuration from livestack-backend/.env via
// dotenvy, relative to `cwd`; the gateway reads channel-gateway/.env. Neither
// .env is in version control, so both live only on the host.

const ROOT = "/home/azureuser/LiveStack";
const BACKEND = `${ROOT}/livestack-backend`;

/** A Rust service built into livestack-backend/target/release. */
function backendService(name) {
  return {
    name,
    script: `./target/release/${name}`,
    cwd: BACKEND,
    env: { RUST_LOG: "info" },
    // Back off between restarts instead of hammering. A service that exits
    // immediately is almost always missing configuration (see .env.example),
    // and pm2 gives up and marks it "errored" after enough attempts — which
    // is the signal we want, rather than a hot restart loop.
    exp_backoff_restart_delay: 5000,
  };
}

module.exports = {
  apps: [
    backendService("api"),
    backendService("producer"),
    backendService("consumer"),
    backendService("webhook-worker"),
    backendService("email-worker"),
    {
      name: "channel-gateway",
      script: "dist/index.js",
      cwd: `${ROOT}/channel-gateway`,
      env: { NODE_ENV: "production" },
    },
  ],
};
