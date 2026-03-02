import { useCallback, useEffect, useMemo, useState } from 'react'
import type { FormEvent, KeyboardEvent } from 'react'
import './App.css'
import { ApiError, createApiClient } from './api/client'
import type { ExecutionPlan, RuntimeMetrics, SessionItem, TaskItem, TaskStateResponse } from './types'

const API_BASE = (import.meta.env.VITE_ZERO_API_BASE as string | undefined)?.trim() || 'http://127.0.0.1:3000'

function App() {
  const client = useMemo(() => createApiClient(API_BASE), [])

  const [tasks, setTasks] = useState<TaskItem[]>([])
  const [selectedTaskId, setSelectedTaskId] = useState('')
  const [selectedTask, setSelectedTask] = useState<TaskItem | null>(null)
  const [taskState, setTaskState] = useState<TaskStateResponse | null>(null)
  const [metrics, setMetrics] = useState<RuntimeMetrics | null>(null)

  const [filter, setFilter] = useState('')
  const [composerText, setComposerText] = useState('')

  const [planJson, setPlanJson] = useState(
    '{\n  "task_id": "",\n  "steps": [\n    { "task_id": "", "step_id": "s1", "op": "agent.execute", "idempotency_key": "i1" }\n  ]\n}',
  )
  const [stepId, setStepId] = useState('s1')

  const [sessions, setSessions] = useState<SessionItem[]>([])
  const [selectedSessionId, setSelectedSessionId] = useState('')

  const [message, setMessage] = useState('')
  const [error, setError] = useState('')
  const [busy, setBusy] = useState(false)

  const showError = (err: unknown) => {
    if (err instanceof ApiError) {
      setError(`${err.code}: ${err.message}${err.details ? `\n${JSON.stringify(err.details)}` : ''}`)
      return
    }
    setError('unknown_error: request failed')
  }

  const run = async (fn: () => Promise<void>) => {
    setBusy(true)
    setError('')
    setMessage('')
    try {
      await fn()
    } catch (err) {
      showError(err)
    } finally {
      setBusy(false)
    }
  }

  const refreshTasks = useCallback(
    async (status?: string) => {
      const list = await client.listTasks(status)
      setTasks(list)
    },
    [client],
  )

  const refreshTaskDetail = useCallback(
    async (id: string) => {
      const [task, state] = await Promise.all([client.getTask(id), client.taskState(id)])
      setSelectedTask(task)
      setTaskState(state)
      setTasks((prev) => {
        const next = prev.filter((t) => t.id !== task.id)
        return [task, ...next]
      })
    },
    [client],
  )

  const activeSessionId = selectedSessionId || sessions[0]?.id || ''

  const addTaskToSession = async (task: TaskItem) => {
    if (!activeSessionId) return
    const session = await client.attachTaskToSession(activeSessionId, task.id)
    setSessions((prev) => prev.map((s) => (s.id === session.id ? session : s)))
  }

  const createSession = () => {
    void run(async () => {
      const created = await client.createSession('新会话')
      setSessions((prev) => [created, ...prev])
      setSelectedSessionId(created.id)
      setMessage(`已创建会话: ${created.id}`)
    })
  }

  const onCreateTaskFromComposer = (e: FormEvent) => {
    e.preventDefault()
    const text = composerText.trim()
    if (!text) {
      setError('bad_request: 输入任务内容后再发送')
      return
    }

    void run(async () => {
      const payload = {
        title: text.length > 32 ? `${text.slice(0, 32)}...` : text,
        description: text,
        dependencies: [],
      }
      const result = await client.createTask(payload)
      setSelectedTaskId(result.id)
      setComposerText('')
      await refreshTasks(filter || undefined)
      await refreshTaskDetail(result.id)
      const task = await client.getTask(result.id)
      await addTaskToSession(task)
      setMessage(`已创建任务: ${result.id}`)
    })
  }

  const onComposerKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      onCreateTaskFromComposer(e)
    }
  }

  const onLoadSelected = () => {
    if (!selectedTaskId.trim()) return
    void run(async () => {
      await refreshTaskDetail(selectedTaskId.trim())
      const nextPlan = await client.getPlan(selectedTaskId.trim()).catch(() => null)
      if (nextPlan) setPlanJson(JSON.stringify(nextPlan, null, 2))
      const loaded = await client.getTask(selectedTaskId.trim())
      await addTaskToSession(loaded)
      setMessage(`已加载任务: ${selectedTaskId.trim()}`)
    })
  }

  const onSubmitPlan = () => {
    if (!selectedTaskId.trim()) {
      setError('bad_request: 先选择任务')
      return
    }
    void run(async () => {
      let parsed: ExecutionPlan
      try {
        parsed = JSON.parse(planJson) as ExecutionPlan
      } catch {
        setError('bad_request: plan json 格式错误')
        return
      }
      await client.submitPlan(selectedTaskId.trim(), parsed)
      setMessage('计划已提交')
      await refreshTaskDetail(selectedTaskId.trim())
      await refreshTasks(filter || undefined)
    })
  }

  const onDispatch = () => {
    if (!selectedTaskId.trim()) return
    void run(async () => {
      const dispatched = await client.dispatch(selectedTaskId.trim())
      setStepId(dispatched.step_id)
      setMessage(`已调度步骤: ${dispatched.step_id}`)
      await refreshTaskDetail(selectedTaskId.trim())
      await refreshTasks(filter || undefined)
    })
  }

  const onCallback = (kind: 'completed' | 'failed' | 'timeout') => {
    if (!selectedTaskId.trim() || !stepId.trim()) {
      setError('bad_request: task id 和 step id 不能为空')
      return
    }
    void run(async () => {
      const id = selectedTaskId.trim()
      const sid = stepId.trim()
      if (kind === 'completed') await client.completeStep(id, sid)
      if (kind === 'failed') await client.failStep(id, sid)
      if (kind === 'timeout') await client.timeoutStep(id, sid)
      setMessage(`${kind} 回调已发送: ${sid}`)
      await refreshTaskDetail(id)
      await refreshTasks(filter || undefined)
    })
  }

  const allTaskMap = useMemo(() => {
    const map = new Map<string, TaskItem>()
    tasks.forEach((t) => map.set(t.id, t))
    if (selectedTask) map.set(selectedTask.id, selectedTask)
    return map
  }, [tasks, selectedTask])

  const activeSession = useMemo(() => sessions.find((s) => s.id === activeSessionId) || sessions[0] || null, [sessions, activeSessionId])

  const activeConversationTasks = useMemo(() => {
    if (!activeSession) return []
    return activeSession.task_ids.map((id) => allTaskMap.get(id)).filter((t): t is TaskItem => Boolean(t))
  }, [activeSession, allTaskMap])

  useEffect(() => {
    void run(async () => {
      const remoteSessions = await client.listSessions()
      if (remoteSessions.length === 0) {
        const created = await client.createSession('默认会话')
        setSessions([created])
        setSelectedSessionId(created.id)
        return
      }
      setSessions(remoteSessions)
      if (!selectedSessionId) setSelectedSessionId(remoteSessions[0].id)
      await refreshTasks(filter || undefined)
      if (selectedTaskId) {
        await refreshTaskDetail(selectedTaskId)
      }
    })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  useEffect(() => {
    if (!selectedSessionId && sessions[0]) {
      setSelectedSessionId(sessions[0].id)
    }
  }, [selectedSessionId, sessions])

  return (
    <div className="app-shell">
      <aside className="left-pane pane">
        <div className="pane-head">
          <h2>会话</h2>
          <button disabled={busy} className="ghost" onClick={createSession}>+ 新会话</button>
        </div>
        <div className="session-list">
          {sessions.map((session) => (
            <button
              key={session.id}
              className={`session-item ${activeSessionId === session.id ? 'active' : ''}`}
              onClick={() => setSelectedSessionId(session.id)}
            >
              <strong>{session.title}</strong>
              <span>{session.task_ids.length} tasks</span>
            </button>
          ))}
        </div>
      </aside>

      <main className="chat-pane pane">
        <header className="chat-head">
          <h1>Zero 对话工作台</h1>
          <p>{activeSession ? `当前会话: ${activeSession.title}` : '未选择会话'}</p>
        </header>

        <section className="chat-flow">
          {activeConversationTasks.length === 0 && <div className="empty">发送第一条消息开始创建任务</div>}
          {activeConversationTasks.map((task) => (
            <article key={task.id} className="chat-group" onClick={() => setSelectedTaskId(task.id)}>
              <div className="bubble user-bubble">
                <h3>{task.title}</h3>
                <p>{task.description || '无描述'}</p>
              </div>
              <div className="bubble bot-bubble">
                <p>任务ID: {task.id}</p>
                <p>状态: {String(task.status)} {selectedTaskId === task.id ? '· 已选中' : ''}</p>
              </div>
            </article>
          ))}
        </section>

        <form className="composer" onSubmit={onCreateTaskFromComposer}>
          <textarea
            placeholder="输入你的任务目标，Enter 发送，Shift+Enter 换行"
            value={composerText}
            onChange={(e) => setComposerText(e.target.value)}
            onKeyDown={onComposerKeyDown}
          />
          <div className="composer-row">
            <button disabled={busy} type="submit">发送并创建任务</button>
          </div>
        </form>

        {(message || error) && (
          <div className="status-line">
            {message && <p className="ok">{message}</p>}
            {error && <pre className="err">{error}</pre>}
          </div>
        )}
      </main>

      <aside className="right-pane pane">
        <div className="pane-head">
          <h2>任务管理</h2>
          <div className="inline-actions">
            <button disabled={busy} className="ghost" onClick={() => void run(async () => setMessage((await client.healthz()).status))}>healthz</button>
            <button disabled={busy} className="ghost" onClick={() => void run(async () => setMetrics(await client.metrics()))}>metrics</button>
          </div>
        </div>

        <div className="control-block">
          <div className="compact-row">
            <select value={filter} onChange={(e) => setFilter(e.target.value)}>
              <option value="">all</option>
              <option value="pending">pending</option>
              <option value="running">running</option>
              <option value="completed">completed</option>
              <option value="failed">failed</option>
            </select>
            <button disabled={busy} onClick={() => void run(async () => refreshTasks(filter || undefined))}>刷新</button>
          </div>

          <div className="task-list">
            {tasks.map((t) => (
              <button
                key={t.id}
                className={`task-card ${selectedTaskId === t.id ? 'active' : ''}`}
                onClick={() => {
                  setSelectedTaskId(t.id)
                  void run(async () => {
                    await refreshTaskDetail(t.id)
                    const nextPlan = await client.getPlan(t.id).catch(() => null)
                    if (nextPlan) setPlanJson(JSON.stringify(nextPlan, null, 2))
                    await addTaskToSession(t)
                  })
                }}
              >
                <strong>{t.title}</strong>
                <span>{t.id}</span>
                <em>{String(t.status)}</em>
              </button>
            ))}
          </div>
        </div>

        <div className="control-block">
          <label>Selected Task ID</label>
          <div className="compact-row">
            <input value={selectedTaskId} onChange={(e) => setSelectedTaskId(e.target.value)} />
            <button disabled={busy} onClick={onLoadSelected}>加载</button>
          </div>

          <label>Plan JSON</label>
          <textarea className="code" value={planJson} onChange={(e) => setPlanJson(e.target.value)} />
          <div className="compact-row">
            <button disabled={busy} onClick={onSubmitPlan}>提交计划</button>
            <button disabled={busy} onClick={onDispatch}>调度下一步</button>
          </div>

          <label>Step ID</label>
          <input value={stepId} onChange={(e) => setStepId(e.target.value)} />
          <div className="compact-row triple">
            <button disabled={busy} onClick={() => onCallback('completed')}>completed</button>
            <button disabled={busy} onClick={() => onCallback('failed')}>failed</button>
            <button disabled={busy} onClick={() => onCallback('timeout')}>timeout</button>
          </div>

          <div className="compact-row">
            <button
              disabled={busy || !selectedTaskId}
              onClick={() =>
                void run(async () => {
                  const res = await client.recover(selectedTaskId, {
                    task_id: selectedTaskId,
                    failure_class: 'provider_timeout',
                    attempt: 1,
                  })
                  setMessage(`recover: ${res.decision}`)
                })
              }
            >recover/decide</button>
            <button
              disabled={busy || !selectedTaskId}
              onClick={() =>
                void run(async () => {
                  const res = await client.verify(selectedTaskId, { task_id: selectedTaskId, outcome: 'hard_fail' })
                  setMessage(`verify: ${res.decision}`)
                })
              }
            >verify/outcome</button>
          </div>
        </div>

        <div className="json-box">
          <h3>Task State</h3>
          <pre>{taskState ? JSON.stringify(taskState, null, 2) : '-'}</pre>
          <h3>Metrics</h3>
          <pre>{metrics ? JSON.stringify(metrics, null, 2) : '-'}</pre>
        </div>
      </aside>
    </div>
  )
}

export default App
