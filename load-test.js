// k6 load test, run by ./performance_test.sh (docker-compose-performance.yml).
//
// Built on the shared OpenAPI coverage module (mairie360/CICD `tests/k6/coverage.js`, MAIR-194):
// every operation of the spec served by the API needs exactly one handler ("METHOD /path", path
// as in openapi.json), k6 aborts at init otherwise. When you add an endpoint, add its handler to
// `readHandlers` (GET) or `writeHandlers` (any other method) and send its request through
// `request()` (raw `http.*` calls are not counted).
//
// High load on a volume seed (MAIR-474): the performance stack also runs init-perf.sql (5 000
// projects, 50 000 tasks, 2 100 accounts in 100 teams, a hot project 12 with 2 000 tasks (200 active, 1 800 archived) and a
// task 87 with 1 000 comments and 1 000 history entries). Three scenarios:
// - `reads`: the GET operations, ramping up to the read VUs of the profile (PROFILES). Each call picks a caller (the Admin, a
//   seeded Responsable or a seeded agent, tokens signed here with the stack's JWT_SECRET) and a
//   random page, so the visibility predicate and deep pages are measured, not only page 1 as Admin;
// - `writes`: every other operation with the write VUs of the profile. Each handler is self-contained: it creates what it
//   needs through `fixture()`, sends its request, then deletes what it created, so the handlers
//   do not depend on their order and the database ends as it started. The task writes share one
//   sandbox project, so they also queue on its `FOR UPDATE` lock (access.rs::begin_write);
// - `list_rush`: `GET /projects/` as non-admins at the fixed arrival rate of the profile, failing if k6 has to drop
//   iterations (the API no longer keeps up).
import http from 'k6/http';
import crypto from 'k6/crypto';
import encoding from 'k6/encoding';
import { check, fail, sleep } from 'k6';
import { createCoverage, loadSpec } from '/coverage.js';

const BASE_URL = (__ENV.BASE_URL || 'http://localhost:3001').replace(/\/+$/, '');

// Static HS256 JWT (sub=1, the Admin seeded by liquibase, role=admin, exp=2100, signed with the
// stack's JWT_SECRET=b"secret"), the same one ZAP injects. The Admin passes every access check
// of the API; public routes ignore the header.
const TOKEN =
  __ENV.JWT ||
  'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxIiwicm9sZSI6ImFkbWluIiwiZXhwIjo0MTAyNDQ0ODAwfQ.xCeBe_2QxRlXW8WXr3t6F69wbEHA93HbP_7l4OTJwjA';
const AUTH = { Authorization: `Bearer ${TOKEN}` };

// Secret of the stack (docker-compose-performance.yml passes the API's JWT_SECRET), used to sign
// the tokens of the seeded non-admin callers.
const JWT_SECRET = __ENV.JWT_SECRET || 'b"secret"';

// Plain `User` accounts seeded by init-test.sql.
const MEMBER_ID = 2;
const OTHER_MEMBER_ID = 3;

// p(95) latency budget of each family of operations, in ms (reference machine).
const READ_BUDGET_MS = 200;
const WRITE_BUDGET_MS = 500;

const READ_METHODS = ['get', 'head', 'options'];

// Rows of init-perf.sql.
const SEEDED_PROJECTS = 5000;
const AGENTS = { first: 100001, count: 2000 };
const MANAGERS = { first: 103001, count: 100 };
// Responsables whose team (agents 100001..100300) belongs to the hot project 12.
const HOT_MANAGERS = 15;
const HOT_PROJECT_ID = 12;
const HOT_TASK_ID = 87;
// Hot project 12 (init-perf.sql): 200 active tasks, 1 800 completed hence archived (MAIR-502).
const HOT_TASKS = 200;
const HOT_ARCHIVED_TASKS = 1800;
const HOT_MEMBERS = 300;
const HOT_FEED = 1000;
const PAGE = 100;

// Load profile (MAIR-474), K6_PROFILE:
// - `ci` (default): what the CI runner holds with the same strict thresholds. The runner
//   (ubuntu-latest, 4 vCPU) hosts the API, Postgres, Redis and k6 together;
// - `stress`: the high load, run by hand (`K6_PROFILE=stress ./performance_test.sh`) to find
//   the breaking point on a larger machine, not on every push.
const PROFILES = {
  ci: { readVus: 30, writeVus: 4, rushRate: 30 },
  stress: { readVus: 100, writeVus: 10, rushRate: 100 },
};
const PROFILE = PROFILES[__ENV.K6_PROFILE || 'ci'];
if (!PROFILE) throw new Error(`Unknown K6_PROFILE ${__ENV.K6_PROFILE}: ${Object.keys(PROFILES).join(', ')}`);

// Fixed-rate `GET /projects/` as non-admins.
const LIST_RUSH_RATE = PROFILE.rushRate; // requests per second
const LIST_RUSH_BUDGET_MS = 200;

function randomInt(max) {
  return Math.floor(Math.random() * max);
}

/** Random page offset of a list of `size` rows. */
function randomOffset(size) {
  return randomInt(Math.max(1, Math.ceil(size / PAGE))) * PAGE;
}

const tokens = {};

/** `Authorization` header of user `sub`, an HS256 token signed with the stack's secret. */
function bearer(sub) {
  if (!tokens[sub]) {
    const part = (value) => encoding.b64encode(JSON.stringify(value), 'rawurl');
    const unsigned = `${part({ alg: 'HS256', typ: 'JWT' })}.${part({ sub: String(sub), role: 'user', exp: 4102444800 })}`;
    tokens[sub] = `Bearer ${unsigned}.${crypto.hmac('sha256', JWT_SECRET, unsigned, 'base64rawurl')}`;
  }
  return { Authorization: tokens[sub] };
}

const randomAgent = () => AGENTS.first + randomInt(AGENTS.count);
const randomManager = () => MANAGERS.first + randomInt(MANAGERS.count);
const randomHotManager = () => MANAGERS.first + randomInt(HOT_MANAGERS);

/** The served spec restricted to the operations whose method passes `keep`. */
function specSubset(spec, keep) {
  const paths = {};
  for (const path of Object.keys(spec.paths)) {
    const kept = {};
    for (const method of Object.keys(spec.paths[path])) {
      if (keep(method)) kept[method] = spec.paths[path][method];
    }
    if (Object.keys(kept).length > 0) paths[path] = kept;
  }
  return Object.assign({}, spec, { paths });
}

/**
 * Raw call for the fixtures of setup(), teardown() and the write handlers, outside the coverage
 * count. Tagged `op: fixture` so it stays out of the per-operation latency thresholds, but it
 * still counts in `http_req_failed`. Aborts the handler (or setup) on a non-2xx answer.
 */
function fixture(method, path, body) {
  const res = http.request(
    method,
    `${BASE_URL}${path}`,
    body === undefined ? null : JSON.stringify(body),
    { headers: Object.assign({ 'Content-Type': 'application/json' }, AUTH), tags: { op: 'fixture' } },
  );
  if (res.status < 200 || res.status >= 300) {
    fail(`fixture ${method} ${path} answered ${res.status}: ${res.body}`);
  }
  return res;
}

function createProject(name) {
  return fixture('POST', '/api/v1/projects/', { name, description: 'k6 fixture' }).json('project_id');
}

function deleteProject(projectId) {
  fixture('DELETE', `/api/v1/projects/${projectId}/`);
}

function createTask(projectId, name) {
  return fixture('POST', `/api/v1/projects/${projectId}/tasks/`, {
    name,
    description: 'k6 fixture',
    assigned_to: MEMBER_ID,
    priority: 'Medium',
    status: 'Todo',
    fields: [],
  }).json('task_id');
}

function deleteTask(projectId, taskId) {
  fixture('DELETE', `/api/v1/projects/${projectId}/tasks/${taskId}/`);
}

function addMember(projectId, userId) {
  fixture('POST', `/api/v1/projects/${projectId}/users/`, { user_id: userId });
}

const spec = loadSpec();

const readHandlers = {
  'GET /health': ({ request }) => check(request(), { 'health 200': (r) => r.status === 200 }),
  'GET /ready': ({ request }) => check(request(), { 'ready 200': (r) => r.status === 200 }),
  // A third of the calls each as the Admin (any page of the 5 000 projects, or the projects page of the
  // BFF: high priority, active, a search), a Responsable (their team's projects) and an agent (their own).
  // Every answer carries the task aggregates and a summary over every matching project (MAIR-474).
  'GET /api/v1/projects/': ({ request }) => {
    const caller = randomInt(3);
    const filtered = caller === 0 && randomInt(2) === 0;
    const res =
      caller === 0
        ? request({
          query: filtered
            ? { limit: 10, priority: 'High', status: 'Active', search: `perf project ${randomInt(50)}` }
            : { limit: PAGE, offset: randomOffset(SEEDED_PROJECTS) },
        })
        : request({ headers: bearer(caller === 1 ? randomManager() : randomAgent()) });
    const sum = (counts) => Object.values(counts || {}).reduce((total, n) => total + n, 0);
    check(res, {
      'list projects 200': (r) => r.status === 200,
      // The Admin sees the whole seed (every seeded project has a high task), a seeded agent or
      // Responsable at least their own projects.
      'list projects reads the seed': (r) =>
        r.status === 200 && r.json('total') >= (caller === 0 && !filtered ? SEEDED_PROJECTS : 1) && r.json('projects').length > 0,
      'list projects summarizes every match': (r) =>
        r.status === 200 && sum(r.json('summary.by_status')) === r.json('total')
        && sum(r.json('summary.by_priority')) === r.json('total')
        && r.json('projects').every((project) => project.tasks_total >= project.tasks_completed),
    });
  },
  // The k6 fixture as Admin, or the hot project as a Responsable who sees it through their team.
  'GET /api/v1/projects/{project_id}/': ({ request, data }) => {
    const res =
      randomInt(2) === 0
        ? request({ path: { project_id: data.projectId } })
        : request({ path: { project_id: HOT_PROJECT_ID }, headers: bearer(randomHotManager()) });
    check(res, {
      'get project 200': (r) => r.status === 200,
      'get project reads its tasks and members': (r) => r.status === 200 && r.json('tasks_total') >= 1,
    });
  },
  'GET /api/v1/projects/{project_id}/archived-tasks/': ({ request }) =>
    check(
      request({
        path: { project_id: HOT_PROJECT_ID },
        query: { limit: PAGE, offset: randomOffset(HOT_ARCHIVED_TASKS) },
        headers: bearer(randomHotManager()),
      }),
      {
        'list archived tasks 200': (r) => r.status === 200,
        'list archived tasks reads the hot project': (r) =>
          r.status === 200 && r.json('total') >= HOT_ARCHIVED_TASKS && r.json('tasks').length > 0
          && r.json('tasks').every((task) => task.archived_at),
      },
    ),
  'GET /api/v1/projects/{project_id}/tasks/': ({ request }) =>
    check(
      request({
        path: { project_id: HOT_PROJECT_ID },
        query: { limit: PAGE, offset: randomOffset(HOT_TASKS) },
        headers: bearer(randomHotManager()),
      }),
      {
        'list tasks 200': (r) => r.status === 200,
        'list tasks reads the hot project': (r) =>
          r.status === 200 && r.json('total') >= HOT_TASKS && r.json('tasks').length > 0,
      },
    ),
  // One task of the hot project, as a Responsable who sees it through their team (MAIR-474).
  'GET /api/v1/projects/{project_id}/tasks/{task_id}/': ({ request }) =>
    check(request({ path: { project_id: HOT_PROJECT_ID, task_id: HOT_TASK_ID }, headers: bearer(randomHotManager()) }), {
      'get task 200': (r) => r.status === 200,
      'get task reads the hot task': (r) => r.status === 200 && r.json('id') === HOT_TASK_ID,
    }),
  'GET /api/v1/projects/{project_id}/tasks/{task_id}/collaboration': ({ request }) =>
    check(
      request({
        path: { project_id: HOT_PROJECT_ID, task_id: HOT_TASK_ID },
        query: { limit: PAGE, offset: randomOffset(HOT_FEED) },
        headers: bearer(randomHotManager()),
      }),
      {
        'task collaboration 200': (r) => r.status === 200,
        'task collaboration reads the hot feeds': (r) =>
          r.status === 200 && r.json('comments_total') >= HOT_FEED && r.json('history_total') >= HOT_FEED,
      },
    ),
  'GET /api/v1/projects/{project_id}/users/': ({ request }) =>
    check(
      request({
        path: { project_id: HOT_PROJECT_ID },
        query: { limit: PAGE, offset: randomOffset(HOT_MEMBERS) },
        headers: bearer(randomHotManager()),
      }),
      {
        'list members 200': (r) => r.status === 200,
        'list members reads the hot project': (r) => r.status === 200 && r.json('total') >= HOT_MEMBERS,
      },
    ),
};

const writeHandlers = {

  // Projects: create → patch → close → delete.
  'POST /api/v1/projects/': ({ request }) => {
    const res = request({ body: { name: 'k6 create project', description: 'Load test' } });
    check(res, { 'create project 200': (r) => r.status === 200 });
    if (res.status === 200) deleteProject(res.json('project_id'));
  },
  'PATCH /api/v1/projects/{project_id}/': ({ request }) => {
    const projectId = createProject('k6 patch project');
    check(request({ path: { project_id: projectId }, body: { name: 'k6 patched', status: 'Suspended' } }), {
      'patch project 204': (r) => r.status === 204,
    });
    deleteProject(projectId);
  },
  'PATCH /api/v1/projects/{project_id}/close': ({ request }) => {
    const projectId = createProject('k6 close project');
    check(request({ path: { project_id: projectId } }), {
      'close project 200': (r) => r.status === 200,
    });
    deleteProject(projectId);
  },
  'DELETE /api/v1/projects/{project_id}/': ({ request }) => {
    const projectId = createProject('k6 delete project');
    check(request({ path: { project_id: projectId } }), {
      'delete project 204': (r) => r.status === 204,
    });
  },

  // Tasks of the write sandbox project: create → patch → comment → delete.
  'POST /api/v1/projects/{project_id}/tasks/': ({ request, data }) => {
    const res = request({
      path: { project_id: data.writeProjectId },
      body: { name: 'k6 create task', description: 'Load test', assigned_to: MEMBER_ID, fields: [] },
    });
    check(res, { 'create task 200': (r) => r.status === 200 });
    if (res.status === 200) deleteTask(data.writeProjectId, res.json('task_id'));
  },
  'PATCH /api/v1/projects/{project_id}/tasks/{task_id}/': ({ request, data }) => {
    const taskId = createTask(data.writeProjectId, 'k6 patch task');
    check(
      request({
        path: { project_id: data.writeProjectId, task_id: taskId },
        body: { status: 'InProgress', priority: 'High' },
      }),
      { 'patch task 204': (r) => r.status === 204 },
    );
    deleteTask(data.writeProjectId, taskId);
  },
  'POST /api/v1/projects/{project_id}/tasks/{task_id}/comments': ({ request, data }) => {
    const taskId = createTask(data.writeProjectId, 'k6 comment task');
    check(
      request({
        path: { project_id: data.writeProjectId, task_id: taskId },
        body: { message: 'Réunion publique calée au 3 octobre.' },
      }),
      { 'comment task 201': (r) => r.status === 201 },
    );
    deleteTask(data.writeProjectId, taskId);
  },
  'DELETE /api/v1/projects/{project_id}/tasks/{task_id}/': ({ request, data }) => {
    const taskId = createTask(data.writeProjectId, 'k6 delete task');
    check(request({ path: { project_id: data.writeProjectId, task_id: taskId } }), {
      'delete task 204': (r) => r.status === 204,
    });
  },

  // Members, on a project of their own: two VUs adding the same user to a shared project would
  // answer 409.
  'POST /api/v1/projects/{project_id}/users/': ({ request }) => {
    const projectId = createProject('k6 add member');
    check(request({ path: { project_id: projectId }, body: { user_id: OTHER_MEMBER_ID } }), {
      'add member 200': (r) => r.status === 200,
    });
    deleteProject(projectId);
  },
  'DELETE /api/v1/projects/{project_id}/users/{user_id}/': ({ request }) => {
    const projectId = createProject('k6 remove member');
    addMember(projectId, OTHER_MEMBER_ID);
    check(request({ path: { project_id: projectId, user_id: OTHER_MEMBER_ID } }), {
      'remove member 204': (r) => r.status === 204,
    });
    deleteProject(projectId);
  },
};

const reads = createCoverage(readHandlers, {
  spec: specSubset(spec, (method) => READ_METHODS.includes(method)),
});
const writes = createCoverage(writeHandlers, {
  spec: specSubset(spec, (method) => !READ_METHODS.includes(method)),
});

/** One `p(95)` threshold per operation (`op` tag) of `coverage`. */
function latencyThresholds(coverage, budgetMs) {
  const thresholds = {};
  for (const operation of coverage.operations) {
    thresholds[`http_req_duration{op:${operation.op}}`] = [`p(95)<${budgetMs}`];
  }
  return thresholds;
}

export const options = {
  scenarios: {
    reads: {
      executor: 'ramping-vus',
      exec: 'readScenario',
      stages: [
        { duration: '30s', target: Math.ceil(PROFILE.readVus / 2) },
        { duration: '30s', target: PROFILE.readVus },
        { duration: '2m', target: PROFILE.readVus }, // Hold
        { duration: '20s', target: 0 },
      ],
    },
    writes: {
      executor: 'constant-vus',
      exec: 'writeScenario',
      vus: PROFILE.writeVus,
      duration: '3m20s',
    },
    list_rush: {
      executor: 'constant-arrival-rate',
      exec: 'listRushScenario',
      startTime: '1m', // once the reads are at full load
      rate: LIST_RUSH_RATE,
      timeUnit: '1s',
      duration: '1m',
      preAllocatedVUs: 50,
      maxVUs: 200,
    },
  },
  thresholds: {
    ...reads.thresholds, // every operation exercised, no handler error (shared counters)
    ...latencyThresholds(reads, READ_BUDGET_MS),
    ...latencyThresholds(writes, WRITE_BUDGET_MS),
    'http_req_duration{op:list_rush}': [`p(95)<${LIST_RUSH_BUDGET_MS}`],
    dropped_iterations: ['count==0'], // the list rush kept its rate
    // Strict (MAIR-474): one wrong status or one missing seeded row fails the run.
    checks: ['rate==1'],
    http_req_failed: ['rate==0'],
  },
};

/** Read fixtures: a project with a member, a task (its creation is logged) and a comment. */
export function setup() {
  const projectId = createProject('k6 read fixture');
  addMember(projectId, MEMBER_ID);
  const taskId = createTask(projectId, 'k6 read task');
  fixture('POST', `/api/v1/projects/${projectId}/tasks/${taskId}/comments`, { message: 'k6 fixture' });
  const writeProjectId = createProject('k6 write sandbox');
  // Tasks are assigned to MEMBER_ID, who must belong to the project (MAIR-393).
  addMember(writeProjectId, MEMBER_ID);
  return { projectId, taskId, writeProjectId };
}

export function teardown(data) {
  deleteProject(data.projectId);
  deleteProject(data.writeProjectId);
}

export function readScenario(data) {
  reads.run({ headers: AUTH, data });
  sleep(1);
}

export function writeScenario(data) {
  writes.run({ headers: AUTH, data });
  sleep(1);
}

export function listRushScenario() {
  const caller = randomInt(2) === 0 ? randomManager() : randomAgent();
  const res = http.get(`${BASE_URL}/api/v1/projects/`, { headers: bearer(caller), tags: { op: 'list_rush' } });
  check(res, {
    'list rush 200': (r) => r.status === 200,
    'list rush reads the seed': (r) => r.status === 200 && r.json('total') >= 1,
  });
}
