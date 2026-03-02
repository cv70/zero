export type TaskStatus = 'Pending' | 'Running' | 'Completed' | 'Failed' | string

export interface TaskItem {
  id: string
  title: string
  description: string
  status: TaskStatus
  dependencies: string[]
  metadata: Record<string, string>
}

export interface StepSpec {
  task_id: string
  step_id: string
  op: string
  idempotency_key: string
}

export interface ExecutionPlan {
  task_id: string
  steps: StepSpec[]
}

export interface DispatchEvent {
  task_id: string
  step_id: string
  event_type: string
  payload: Record<string, unknown>
  timestamp: string
}

export interface DispatchResponse {
  task_id: string
  step_id: string
  output: string
  from_cache: boolean
  event: DispatchEvent
}

export interface RuntimeMetrics {
  tasks_per_min: number
  task_success_rate: number
  token_per_task: number
  p50_latency_ms: number
  p95_latency_ms: number
  p99_latency_ms: number
  total_tasks: number
  planned_tasks: number
  state_counts: Record<string, number>
}

export interface TaskStateResponse {
  task_id: string
  state: string
}

export interface RecoveryDecisionResponse {
  task_id: string
  decision: string
}

export interface ApiErrorShape {
  error: string
  code: string
  details: unknown
}

export interface CreateTaskPayload {
  title: string
  description: string
  dependencies: string[]
}

export interface PatchTaskPayload {
  title?: string
  description?: string
  metadata?: Record<string, string>
}

export interface RecoverPayload {
  task_id: string
  failure_class: 'provider_timeout' | 'tool_invalid_args' | 'planning_mismatch'
  attempt: number
}

export interface VerifyPayload {
  task_id: string
  outcome: 'passed' | 'needs_repair' | 'hard_fail'
}

export interface SessionItem {
  id: string
  title: string
  task_ids: string[]
  created_at: number
  updated_at: number
}

export interface CreateSessionPayload {
  title?: string
}
