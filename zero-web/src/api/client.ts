import type {
  ApiErrorShape,
  CreateTaskPayload,
  DispatchResponse,
  ExecutionPlan,
  PatchTaskPayload,
  RecoverPayload,
  RecoveryDecisionResponse,
  RuntimeMetrics,
  SessionItem,
  TaskItem,
  TaskStateResponse,
  VerifyPayload,
} from '../types'

export class ApiError extends Error {
  status: number
  code: string
  details: unknown

  constructor(message: string, status: number, code = 'network_error', details: unknown = null) {
    super(message)
    this.status = status
    this.code = code
    this.details = details
  }
}

async function parse<T>(res: Response): Promise<T> {
  if (res.ok) {
    if (res.status === 204) return undefined as T
    return (await res.json()) as T
  }

  let body: ApiErrorShape | null = null
  try {
    body = (await res.json()) as ApiErrorShape
  } catch {
    // noop
  }

  throw new ApiError(body?.error ?? `Request failed: ${res.status}`, res.status, body?.code ?? 'http_error', body?.details ?? null)
}

async function request<T>(baseUrl: string, path: string, init?: RequestInit): Promise<T> {
  let res: Response
  try {
    res = await fetch(`${baseUrl}${path}`, {
      ...init,
      headers: {
        'content-type': 'application/json',
        ...(init?.headers ?? {}),
      },
    })
  } catch {
    throw new ApiError('Unable to connect to zero-api', 0, 'network_error', null)
  }
  return parse<T>(res)
}

export function createApiClient(baseUrl: string) {
  return {
    healthz: () => request<{ status: string }>(baseUrl, '/healthz'),
    metrics: () => request<RuntimeMetrics>(baseUrl, '/metrics/runtime'),
    createTask: (payload: CreateTaskPayload) =>
      request<{ id: string }>(baseUrl, '/tasks', { method: 'POST', body: JSON.stringify(payload) }),
    listTasks: (status?: string) =>
      request<TaskItem[]>(baseUrl, `/tasks${status ? `?status=${encodeURIComponent(status)}` : ''}`),
    getTask: (id: string) => request<TaskItem>(baseUrl, `/tasks/${id}`),
    patchTask: (id: string, payload: PatchTaskPayload) =>
      request<TaskItem>(baseUrl, `/tasks/${id}`, { method: 'PATCH', body: JSON.stringify(payload) }),
    deleteTask: (id: string) => request<void>(baseUrl, `/tasks/${id}`, { method: 'DELETE' }),
    submitPlan: (id: string, payload: ExecutionPlan) =>
      request<{ id: string }>(baseUrl, `/tasks/${id}/plan`, { method: 'POST', body: JSON.stringify(payload) }),
    getPlan: (id: string) => request<ExecutionPlan>(baseUrl, `/tasks/${id}/plan`),
    dispatch: (id: string) => request<DispatchResponse>(baseUrl, `/tasks/${id}/dispatch`, { method: 'POST' }),
    completeStep: (id: string, stepId: string) =>
      request<void>(baseUrl, `/tasks/${id}/steps/${stepId}/completed`, { method: 'POST' }),
    failStep: (id: string, stepId: string) =>
      request<void>(baseUrl, `/tasks/${id}/steps/${stepId}/failed`, { method: 'POST' }),
    timeoutStep: (id: string, stepId: string) =>
      request<void>(baseUrl, `/tasks/${id}/steps/${stepId}/timeout`, { method: 'POST' }),
    taskState: (id: string) => request<TaskStateResponse>(baseUrl, `/tasks/${id}/state`),
    recover: (id: string, payload: RecoverPayload) =>
      request<RecoveryDecisionResponse>(baseUrl, `/tasks/${id}/recover/decide`, {
        method: 'POST',
        body: JSON.stringify(payload),
      }),
    verify: (id: string, payload: VerifyPayload) =>
      request<RecoveryDecisionResponse>(baseUrl, `/tasks/${id}/verify/outcome`, {
        method: 'POST',
        body: JSON.stringify(payload),
      }),
    createSession: (title?: string) =>
      request<SessionItem>(baseUrl, '/sessions', { method: 'POST', body: JSON.stringify({ title }) }),
    listSessions: () => request<SessionItem[]>(baseUrl, '/sessions'),
    getSession: (id: string) => request<SessionItem>(baseUrl, `/sessions/${id}`),
    patchSession: (id: string, title: string) =>
      request<SessionItem>(baseUrl, `/sessions/${id}`, { method: 'PATCH', body: JSON.stringify({ title }) }),
    deleteSession: (id: string) => request<void>(baseUrl, `/sessions/${id}`, { method: 'DELETE' }),
    attachTaskToSession: (id: string, taskId: string) =>
      request<SessionItem>(baseUrl, `/sessions/${id}/tasks`, {
        method: 'POST',
        body: JSON.stringify({ task_id: taskId }),
      }),
    detachTaskFromSession: (id: string, taskId: string) =>
      request<SessionItem>(baseUrl, `/sessions/${id}/tasks/${taskId}`, { method: 'DELETE' }),
  }
}
