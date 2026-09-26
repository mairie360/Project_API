// k6 load test, run by ./performance_test.sh (docker-compose-performance.yml).
//
// Built on the shared OpenAPI coverage module (mairie360/CICD `tests/k6/coverage.js`, MAIR-194):
// every operation of the spec served by the API needs exactly one handler ("METHOD /path", path
// as in openapi.json), k6 aborts at init otherwise. When you add an endpoint, add its handler to
// `readHandlers` (GET) or `writeHandlers` (any other method) and send its request through
// `request()` (raw `http.*` calls are not counted).
//
// Two scenarios share the spec, split by HTTP method:
// - `reads`: the GET operations under the historical profile (ramp up to 20 VUs), against the
//   fixtures created once in setup() and removed in teardown();
// - `writes`: every other operation with 2 VUs. Each handler is self-contained: it creates what it
//   needs through `fixture()`, sends its request, then deletes what it created, so the handlers
//   do not depend on their order and the database ends as it started.
import http from 'k6/http';
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

// Plain `User` accounts seeded by init-test.sql.
const MEMBER_ID = 2;
const OTHER_MEMBER_ID = 3;

// p(95) latency budget of each family of operations, in ms (reference machine).
const READ_BUDGET_MS = 200;
const WRITE_BUDGET_MS = 500;

const READ_METHODS = ['get', 'head', 'options'];

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
  'GET /api/v1/projects/': ({ request }) =>
    check(request(), { 'list projects 200': (r) => r.status === 200 }),
  'GET /api/v1/projects/{project_id}/': ({ request, data }) =>
    check(request({ path: { project_id: data.projectId } }), {
      'get project 200': (r) => r.status === 200,
    }),
  'GET /api/v1/projects/{project_id}/tasks/': ({ request, data }) =>
    check(request({ path: { project_id: data.projectId } }), {
      'list tasks 200': (r) => r.status === 200,
    }),
  'GET /api/v1/projects/{project_id}/tasks/{task_id}/collaboration': ({ request, data }) =>
    check(request({ path: { project_id: data.projectId, task_id: data.taskId } }), {
      'task collaboration 200': (r) => r.status === 200,
    }),
  'GET /api/v1/projects/{project_id}/users/': ({ request, data }) =>
    check(request({ path: { project_id: data.projectId } }), {
      'list members 200': (r) => r.status === 200,
    }),
};

const writeHandlers = {
  'POST /': ({ request }) => check(request(), { 'hello 200': (r) => r.status === 200 }),

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

  // Tasks of the write sandbox project: create → patch → comment → history → delete.
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
  'POST /api/v1/projects/{project_id}/tasks/{task_id}/history': ({ request, data }) => {
    const taskId = createTask(data.writeProjectId, 'k6 history task');
    check(
      request({
        path: { project_id: data.writeProjectId, task_id: taskId },
        body: {
          action: 'task_updated',
          label: 'Budget révisé après consultation',
          changes: { budget: { from: 12000, to: 15500 } },
        },
      }),
      { 'task history 201': (r) => r.status === 201 },
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
        { duration: '30s', target: 20 }, // Ramp up to 20 virtual users
        { duration: '1m', target: 20 }, // Hold
        { duration: '10s', target: 0 }, // Ramp down
      ],
    },
    writes: {
      executor: 'constant-vus',
      exec: 'writeScenario',
      vus: 2,
      duration: '1m40s',
    },
  },
  thresholds: {
    ...reads.thresholds, // every operation exercised, no handler error (shared counters)
    ...latencyThresholds(reads, READ_BUDGET_MS),
    ...latencyThresholds(writes, WRITE_BUDGET_MS),
    http_req_failed: ['rate<0.01'], // Less than 1% errors
  },
};

/** Read fixtures: a project with a member, a task, a comment and a history entry. */
export function setup() {
  const projectId = createProject('k6 read fixture');
  addMember(projectId, MEMBER_ID);
  const taskId = createTask(projectId, 'k6 read task');
  fixture('POST', `/api/v1/projects/${projectId}/tasks/${taskId}/comments`, { message: 'k6 fixture' });
  fixture('POST', `/api/v1/projects/${projectId}/tasks/${taskId}/history`, {
    action: 'task_created',
    label: 'k6 fixture',
  });
  const writeProjectId = createProject('k6 write sandbox');
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
